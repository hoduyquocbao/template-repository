//! Enum đại diện cho các message gửi tới actor lưu trữ
use crate::error::Error;
use tokio::sync::oneshot;

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