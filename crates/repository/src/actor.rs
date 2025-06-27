//! Actor pattern cho Sled: tách thread lưu trữ riêng biệt, giao tiếp qua channel.

use std::thread;
use tokio::sync::{mpsc, oneshot};
use crate::Error;
use super::sled::Inner;
use async_trait::async_trait;

/// Enum đại diện cho các message gửi tới actor lưu trữ
pub enum Message {
    Insert {
        key: Vec<u8>,
        value: Vec<u8>,
        respond: oneshot::Sender<Result<(), Error>>,
    },
    Fetch {
        key: Vec<u8>,
        respond: oneshot::Sender<Result<Option<Vec<u8>>, Error>>,
    },
    Update {
        key: Vec<u8>,
        value: Vec<u8>,
        respond: oneshot::Sender<Result<Vec<u8>, Error>>,
    },
    Delete {
        key: Vec<u8>,
        respond: oneshot::Sender<Result<Vec<u8>, Error>>,
    },
    Query {
        respond: oneshot::Sender<Result<Vec<Vec<u8>>, Error>>,
    },
    Mass {
        entries: Vec<(Vec<u8>, Vec<u8>)>,
        respond: oneshot::Sender<Result<(), Error>>,
    },
    Keys {
        respond: oneshot::Sender<Result<Vec<Vec<u8>>, Error>>,
    },
}

/// Actor lưu trữ: chạy thread riêng, nhận message qua channel
pub struct Actor {
    sender: mpsc::Sender<Message>,
}

impl Actor {
    pub(crate) fn new(inner: Inner) -> Self {
        let (tx, mut rx) = mpsc::channel::<Message>(128);
        let metric = inner.metric.clone();
        thread::spawn(move || {
            while let Some(msg) = rx.blocking_recv() {
                match msg {
                    Message::Insert { key, value, respond } => {
                        let res = inner.db.insert(&key[..], &value[..]).map(|_| ()).map_err(Error::Store);
                        
                        // Ghi lại metric với tên "insert" và kết quả của thao tác
                        metric.record("insert", res.is_err());
                        
                        let _ = respond.send(res);
                    }
                    Message::Fetch { key, respond } => {
                        let res = inner.db.get(&key[..]).map(|opt| opt.map(|v| v.to_vec())).map_err(Error::Store);
                        
                        // Ghi lại metric với tên "fetch"
                        metric.record("fetch", res.is_err());

                        let _ = respond.send(res);
                    }
                    Message::Update { key, value, respond } => {
                        let res = inner.db.insert(&key[..], &value[..]).map(|_| value.clone()).map_err(Error::Store);
                        
                        // Ghi lại metric với tên "update"
                        metric.record("update", res.is_err());
                        
                        let _ = respond.send(res);
                    }
                    Message::Delete { key, respond } => {
                        let res = inner.db.remove(&key[..]).map(|opt| opt.map(|v| v.to_vec()).unwrap_or_default()).map_err(Error::Store);
                        
                        // Ghi lại metric với tên "delete"
                        metric.record("delete", res.is_err());
                        
                        let _ = respond.send(res);
                    }
                    Message::Query { respond } => {
                        let mut result = Vec::new();
                        let mut iter = inner.db.iter();
                        let mut error = None;
                        for kv in &mut iter {
                            match kv {
                                Ok((_, v)) => result.push(v.to_vec()),
                                Err(e) => { error = Some(e); break; }
                            }
                        }
                        let res = if let Some(e) = error {
                            Err(Error::Store(e))
                        } else {
                            Ok(result)
                        };
                        
                        // Ghi lại metric với tên "query"
                        metric.record("query", res.is_err());
                        
                        let _ = respond.send(res);
                    }
                    Message::Mass { entries, respond } => {
                        let mut ok = true;
                        for (k, v) in entries.iter() {
                            if inner.db.insert(&k[..], &v[..]).is_err() {
                                ok = false;
                                break;
                            }
                        }
                        let res = if ok { Ok(()) } else { Err(Error::Aborted) };
                        
                        // Ghi lại metric với tên "mass"
                        metric.record("mass", res.is_err());
                        
                        let _ = respond.send(res);
                    }
                    Message::Keys { respond } => {
                        let mut result = Vec::new();
                        let mut iter = inner.db.iter();
                        let mut error = None;
                        for kv in &mut iter {
                            match kv {
                                Ok((k, _)) => result.push(k.to_vec()),
                                Err(e) => { error = Some(e); break; }
                            }
                        }
                        let res = if let Some(e) = error {
                            Err(Error::Store(e))
                        } else {
                            Ok(result)
                        };
                        
                        // Ghi lại metric với tên "keys"
                        metric.record("keys", res.is_err());
                        
                        let _ = respond.send(res);
                    }
                }
            }
        });
        Self { sender: tx }
    }
    pub fn handle(&self) -> Handle {
        Handle { sender: self.sender.clone() }
    }
}

/// Handle gửi request tới actor, cloneable
#[derive(Clone)]
pub struct Handle {
    sender: mpsc::Sender<Message>,
}

#[async_trait]
pub trait Actorable: Send + Sync + Clone + 'static {
    async fn insert(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error>;
    async fn fetch(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, Error>;
    async fn update(&self, key: Vec<u8>, value: Vec<u8>) -> Result<Vec<u8>, Error>;
    async fn delete(&self, key: Vec<u8>) -> Result<Vec<u8>, Error>;
    async fn query(&self) -> Result<Vec<Vec<u8>>, Error>;
    async fn mass(&self, entries: Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), Error>;
    async fn keys(&self) -> Result<Vec<Vec<u8>>, Error>;
}

#[async_trait]
impl Actorable for Handle {
    async fn insert(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Insert { key, value, respond: tx };
        self.sender.send(msg).await.map_err(|_| Error::Aborted)?;
        rx.await.map_err(|_| Error::Aborted)?
    }
    async fn fetch(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Fetch { key, respond: tx };
        self.sender.send(msg).await.map_err(|_| Error::Aborted)?;
        rx.await.map_err(|_| Error::Aborted)?
    }
    async fn update(&self, key: Vec<u8>, value: Vec<u8>) -> Result<Vec<u8>, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Update { key, value, respond: tx };
        self.sender.send(msg).await.map_err(|_| Error::Aborted)?;
        rx.await.map_err(|_| Error::Aborted)?
    }
    async fn delete(&self, key: Vec<u8>) -> Result<Vec<u8>, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Delete { key, respond: tx };
        self.sender.send(msg).await.map_err(|_| Error::Aborted)?;
        rx.await.map_err(|_| Error::Aborted)?
    }
    async fn query(&self) -> Result<Vec<Vec<u8>>, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Query { respond: tx };
        self.sender.send(msg).await.map_err(|_| Error::Aborted)?;
        rx.await.map_err(|_| Error::Aborted)?
    }
    async fn mass(&self, entries: Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Mass { entries, respond: tx };
        self.sender.send(msg).await.map_err(|_| Error::Aborted)?;
        rx.await.map_err(|_| Error::Aborted)?
    }
    async fn keys(&self) -> Result<Vec<Vec<u8>>, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Keys { respond: tx };
        self.sender.send(msg).await.map_err(|_| Error::Aborted)?;
        rx.await.map_err(|_| Error::Aborted)?
    }
}

// TODO: Triển khai các hàm gửi message bất đồng bộ cho Handle 