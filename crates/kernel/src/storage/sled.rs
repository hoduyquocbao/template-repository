//! Triển khai cụ thể của `Storage` trait sử dụng cơ sở dữ liệu Sled.
//!
//! Module này cung cấp một triển khai hiệu năng cao của giao diện lưu trữ
//! sử dụng Sled - một cơ sở dữ liệu embedded, nhúng dạng key-value với các giao dịch
//! ACID. Module này triển khai các chiến lược quan trọng để tối ưu hiệu năng:
//!
//! 1. Sử dụng chỉ mục bao phủ (covering index) để truy vấn hiệu quả
//! 2. Thực hiện các thao tác cập nhật trong các giao dịch đảm bảo tính toàn vẹn dữ liệu
//! 3. Xử lý bất đồng bộ thông qua tokio để đạt được khả năng mở rộng đồng thời cao

// ---
// Import các module, trait, struct cần thiết cho lưu trữ, đồng bộ hóa, cache, metric, tracing, v.v.
use crate::storage::actor::{Handle, Actor, Actorable};
use crate::Error;
use async_trait::async_trait;
use crate::storage::entity::{Entity, Query};

/// Wrapper xung quanh actor lưu trữ
/// Mục đích: Gom nhóm các thành phần lưu trữ qua actor để tối ưu hóa hiệu năng và khả năng mở rộng
#[derive(Clone)]
pub struct Sled {
    pub handle: Handle,
}

impl Sled {
    pub fn new(path: &str) -> Result<Self, Error> {
        let inner = Inner::new(path)?;
        let actor = Actor::new(inner);
        Ok(Self { handle: actor.handle() })
    }
}

/// Đổi tên struct SledInner thành Inner
pub(crate) struct Inner {
    pub db: sled::Db,
    #[allow(dead_code)]
    pub pool: crate::storage::pool::Pool<sled::Db>,
    #[allow(dead_code)]
    pub cache: crate::storage::cache::Cache<Vec<u8>, Vec<u8>>,
    #[allow(dead_code)]
    pub metric: crate::metric::Registry,
}

impl Inner {
    pub fn new(path: &str) -> Result<Self, Error> {
        let db = sled::Config::new()
            .path(path)
            .temporary(path.is_empty())
            .open()?;
        let pool = crate::storage::pool::Pool::new(10, || Ok(db.clone()))?;
        let cache = crate::storage::cache::Cache::new(std::time::Duration::from_secs(300));
        let metric = crate::metric::Registry::new();
        Ok(Self { db, pool, cache, metric })
    }
}

#[async_trait]
impl crate::storage::Storage for Sled {
    async fn insert<E: Entity>(&self, entity: E) -> Result<(), Error>
    where E::Key: std::fmt::Debug + serde::Serialize, E::Index: std::fmt::Debug {
        let key = bincode::serialize(&entity.key())?;
        let value = bincode::serialize(&entity)?;
        self.handle.insert(key, value).await
    }

    async fn fetch<E: Entity>(&self, key: E::Key) -> Result<Option<E>, Error>
    where E::Key: std::fmt::Debug + serde::Serialize {
        let key = bincode::serialize(&key)?;
        let res = self.handle.fetch(key).await?;
        match res {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }

    async fn update<E: Entity, F>(&self, key: E::Key, transform: F) -> Result<E, Error>
    where
        F: FnOnce(E) -> E + Send + 'static,
        E::Key: std::fmt::Debug + serde::Serialize {
        let old = self.fetch::<E>(key.clone()).await?.ok_or(Error::Missing)?;
        let new = transform(old);
        let key = bincode::serialize(&key)?;
        let value = bincode::serialize(&new)?;
        let res = self.handle.update(key, value).await?;
        Ok(bincode::deserialize(&res)?)
    }

    async fn delete<E: Entity>(&self, key: E::Key) -> Result<E, Error>
    where E::Key: std::fmt::Debug + serde::Serialize {
        let key = bincode::serialize(&key)?;
        let res = self.handle.delete(key).await?;
        Ok(bincode::deserialize(&res)?)
    }

    async fn query<E: Entity>(&self, query: Query<E::Index>) -> Result<Box<dyn Iterator<Item = Result<E::Summary, Error>> + Send>, Error>
    where E::Index: std::fmt::Debug {
        tracing::debug!("Sled query với prefix: {:?}, after: {:?}, limit: {}", query.prefix, query.after, query.limit);
        
        let res = self.handle.query().await?;
        let mut items: Vec<E::Summary> = Vec::new();
        
        for (i, bytes) in res.into_iter().enumerate() {
            if i >= query.limit {
                break;
            }
            
            match bincode::deserialize::<E>(&bytes) {
                Ok(entry) => {
                    items.push(entry.summary());
                },
                Err(e) => {
                    tracing::warn!("Lỗi deserialize item {}: {:?}", i, e);
                    // Bỏ qua item lỗi thay vì fail toàn bộ query
                    continue;
                }
            }
        }
        
        tracing::debug!("Query trả về {} items thành công", items.len());
        Ok(Box::new(items.into_iter().map(Ok)))
    }

    async fn mass<E: Entity>(&self, iter: Box<dyn Iterator<Item = E> + Send>) -> Result<(), Error>
    where E::Key: std::fmt::Debug + serde::Serialize, E::Index: std::fmt::Debug {
        let entries: Vec<(Vec<u8>, Vec<u8>)> = iter.map(|e| {
            let k = bincode::serialize(&e.key()).unwrap();
            let v = bincode::serialize(&e).unwrap();
            (k, v)
        }).collect();
        self.handle.mass(entries).await
    }

    #[cfg(any(test, feature = "testing"))]
    async fn keys<E: Entity>(&self, _query: Query<E::Index>) -> Result<Box<dyn Iterator<Item = Result<Vec<u8>, Error>> + Send>, Error>
    where E::Index: std::fmt::Debug {
        let res = self.handle.keys().await?;
        Ok(Box::new(res.into_iter().map(Ok)))
    }
}

mod tests {
    #[allow(unused_imports)]
    use crate::storage::Storage;
    use crate::{Entity, Id, Sled};
    use serde::{Serialize, Deserialize};
    use tempfile::tempdir;

    #[allow(dead_code)]
    fn memory() -> Sled {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        Sled::new(&path).unwrap()
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct Thing {
        id: Id,
        name: String,
        value: u32,
    }
    
    #[derive(Serialize, Deserialize)]
    struct Brief {
        id: Id,
        name: String,
    }
    
    impl Entity for Thing {
        // Thay đổi: "test_things" -> "things"
        const NAME: &'static str = "things";
        type Key = Id;
        type Index = Vec<u8>;
        type Summary = Brief;
        
        fn key(&self) -> Self::Key { self.id }
        fn index(&self) -> Self::Index { 
            // Sử dụng giá trị để tạo các khóa chỉ mục khác nhau cho mỗi thực thể
            // Điều này giúp đảm bảo mỗi thực thể có một chỉ mục riêng biệt
            format!("idx_{}", self.value).into_bytes()
        }
        fn summary(&self) -> Self::Summary {
            Brief { id: self.id, name: self.name.clone() }
        }
    }

    #[tokio::test]
    async fn crud() {
        let store = memory();
        let item = Thing { id: Id::new_v4(), name: "Test".to_string(), value: 42 };
        // Insert
        store.insert(item.clone()).await.unwrap();
        // Fetch
        let fetched = store.fetch::<Thing>(item.id).await.unwrap().unwrap();
        assert_eq!(item, fetched);
        // Update
        let updated = Thing { value: 100, ..item.clone() };
        store.insert(updated.clone()).await.unwrap();
        let fetched = store.fetch::<Thing>(item.id).await.unwrap().unwrap();
        assert_eq!(updated, fetched);
        // Delete
        let deleted = store.delete::<Thing>(item.id).await.unwrap();
        assert_eq!(updated, deleted);
        // Verify deletion
        assert!(store.fetch::<Thing>(item.id).await.unwrap().is_none());
    }
    
    #[tokio::test]
    async fn bulk() {
        let store = memory();
        let things: Vec<_> = (0..100).map(|i| Thing {
            id: Id::new_v4(),
            name: format!("Thing {}", i),
            value: i,
        }).collect();
        store.mass(Box::new(things.clone().into_iter())).await.unwrap();
        for item in &things {
            let fetched = store.fetch::<Thing>(item.id).await.unwrap().unwrap();
            assert_eq!(*item, fetched);
        }
    }
}
