use crate::storage::actor::message::Message;
use crate::storage::sled::Inner;
use crate::metric::Registry;
use crate::error::Error;

pub(crate) fn handle(msg: Message, inner: &Inner, metric: &Registry) {
    match msg {
        Message::Insert { key, value, respond } => {
            let res = inner.db.insert(&key[..], &value[..])
                .map(|_| ())
                .map_err(Error::Store);
            if let Err(ref e) = res {
                tracing::error!(?e, "Lỗi khi insert vào db");
            }
            metric.record("insert", res.is_err());
            if respond.send(res).is_err() {
                tracing::error!("Lỗi gửi kết quả insert qua channel oneshot");
            }
        }
        Message::Fetch { key, respond } => {
            let res = inner.db.get(&key[..])
                .map(|opt| opt.map(|v| v.to_vec()))
                .map_err(Error::Store);
            if let Err(ref e) = res {
                tracing::error!(?e, "Lỗi khi fetch từ db");
            }
            metric.record("fetch", res.is_err());
            if respond.send(res).is_err() {
                tracing::error!("Lỗi gửi kết quả fetch qua channel oneshot");
            }
        }
        Message::Update { key, value, respond } => {
            let res = inner.db.insert(&key[..], &value[..])
                .map(|_| value.clone())
                .map_err(Error::Store);
            if let Err(ref e) = res {
                tracing::error!(?e, "Lỗi khi update vào db");
            }
            metric.record("update", res.is_err());
            if respond.send(res).is_err() {
                tracing::error!("Lỗi gửi kết quả update qua channel oneshot");
            }
        }
        Message::Delete { key, respond } => {
            let res = inner.db.remove(&key[..])
                .map(|opt| opt.map(|v| v.to_vec()).unwrap_or_default())
                .map_err(Error::Store);
            if let Err(ref e) = res {
                tracing::error!(?e, "Lỗi khi delete từ db");
            }
            metric.record("delete", res.is_err());
            if respond.send(res).is_err() {
                tracing::error!("Lỗi gửi kết quả delete qua channel oneshot");
            }
        }
        Message::Query { respond } => {
            let mut result = Vec::new();
            let mut iter = inner.db.iter();
            let mut error = None;
            tracing::debug!("Bắt đầu query database");
            for kv in &mut iter {
                match kv {
                    Ok((k, v)) => {
                        if !v.is_empty() {
                            if k.len() >= 16 {
                                result.push(v.to_vec());
                            } else {
                                tracing::warn!("Bỏ qua key quá ngắn: {} bytes", k.len());
                            }
                        } else {
                            tracing::warn!("Bỏ qua value rỗng trong query");
                        }
                    },
                    Err(e) => {
                        error = Some(e.clone());
                        tracing::error!(?e, "Lỗi khi query database");
                        break;
                    }
                }
            }
            let res = if let Some(e) = error {
                tracing::error!(?e, "Query thất bại");
                Err(Error::Store(e))
            } else {
                tracing::debug!("Query thành công, trả về {} items", result.len());
                Ok(result)
            };
            metric.record("query", res.is_err());
            if respond.send(res).is_err() {
                tracing::error!("Lỗi gửi kết quả query qua channel oneshot");
            }
        }
        Message::Mass { entries, respond } => {
            let mut ok = true;
            for (k, v) in entries.iter() {
                if inner.db.insert(&k[..], &v[..]).is_err() {
                    tracing::error!("Lỗi khi insert trong mass");
                    ok = false;
                    break;
                }
            }
            let res = if ok { Ok(()) } else { Err(Error::Aborted) };
            metric.record("mass", res.is_err());
            if respond.send(res).is_err() {
                tracing::error!("Lỗi gửi kết quả mass qua channel oneshot");
            }
        }
        Message::Keys { respond } => {
            let mut result = Vec::new();
            let mut iter = inner.db.iter();
            let mut error = None;
            for kv in &mut iter {
                match kv {
                    Ok((k, _)) => result.push(k.to_vec()),
                    Err(e) => { let err = e.clone(); error = Some(e); tracing::error!(?err, "Lỗi khi lấy keys"); break; }
                }
            }
            let res = if let Some(e) = error {
                Err(Error::Store(e))
            } else {
                Ok(result)
            };
            metric.record("keys", res.is_err());
            if respond.send(res).is_err() {
                tracing::error!("Lỗi gửi kết quả keys qua channel oneshot");
            }
        }
    }
} 