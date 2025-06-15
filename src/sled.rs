//! Triển khai cụ thể của `Storage` trait sử dụng cơ sở dữ liệu Sled.
//!
//! Module này cung cấp một triển khai hiệu năng cao của giao diện lưu trữ
//! sử dụng Sled - một cơ sở dữ liệu embedded, nhúng dạng key-value với các giao dịch
//! ACID. Module này triển khai các chiến lược quan trọng để tối ưu hiệu năng:
//!
//! 1. Sử dụng chỉ mục bao phủ (covering index) để truy vấn hiệu quả
//! 2. Thực hiện các thao tác cập nhật trong các giao dịch đảm bảo tính toàn vẹn dữ liệu
//! 3. Xử lý bất đồng bộ thông qua tokio để đạt được khả năng mở rộng đồng thời cao

use crate::{Error, entity::{Entity, Query}};
use crate::storage::Storage;
use crate::pool::Pool;
use crate::cache::Cache;
use crate::metric::{Registry, Metric};
use sled::{Db, Tree, Transactional, transaction::ConflictableTransactionError};
use tokio::task::spawn_blocking;
use tracing::{debug, instrument, trace, warn};
use std::time::{Duration, Instant};
use std::future::Future;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use async_trait::async_trait;

// Kích thước khối cho xử lý hàng loạt
const CHUNK: usize = 1000;

/// Wrapper xung quanh `sled::Db` với các tính năng nâng cao
#[derive(Clone)]
pub struct Sled {
    /// Cơ sở dữ liệu Sled gốc
    pub(crate) db: Db,
    /// Pool kết nối
    #[allow(dead_code)]
    pool: Pool<Db>,
    /// Cache cho các thực thể
    #[allow(dead_code)]
    cache: Cache<Vec<u8>, Vec<u8>>,
    /// Registry cho metrics
    #[allow(dead_code)]
    metric: Registry,
}

impl Sled {
    /// Tạo instance Sled mới với các tính năng nâng cao
    pub fn new(path: &str) -> Result<Self, Error> {
        let db = sled::Config::new()
            .path(path)
            .temporary(path.is_empty())
            .open()?;
            
        let pool = Pool::new(10, || Ok(db.clone()))?;
        let cache = Cache::new(Duration::from_secs(300)); // 5 phút TTL
        let metric = Registry::new();
        
        Ok(Self { db, pool, cache, metric })
    }
    
    /// Lấy metric cho một thao tác
    #[allow(dead_code)]
    async fn metric(&self, name: &str) -> Metric {
        self.metric.get(name).await
    }
    
    /// Thực hiện thao tác với metric
    #[allow(dead_code)]
    async fn with_metric<F, T>(&self, name: &str, f: F) -> Result<T, Error>
    where
        F: Future<Output = Result<T, Error>>,
    {
        let start = Instant::now();
        let metric = self.metric(name).await;
        let result = f.await;
        metric.record(start, result.is_err());
        result
    }
    
    /// Lấy dữ liệu từ cache hoặc storage
    #[allow(dead_code)]
    async fn get<E: Entity>(&self, id: &E::Key) -> Result<Option<E>, Error>
    where 
        E::Key: Debug + AsRef<[u8]> + Sync,
        E: DeserializeOwned,
    {
        // Thử lấy từ cache
        let key = id.as_ref().to_vec();
        if let Some(data) = self.cache.get(&key).await {
            return Ok(Some(bincode::deserialize(&data)?));
        }
        let key2 = key.clone();
        let this = self.clone();
        let result = spawn_blocking(move || {
            let data = this.data::<E>()?;
            data.get(&key2).map_err(Error::Store)
        }).await??;
        
        if let Some(data) = result {
            self.cache.set(key, data.to_vec()).await;
            Ok(Some(bincode::deserialize(&data)?))
        } else {
            Ok(None)
        }
    }
    
    /// Lấy (hoặc tạo) cây dữ liệu chính cho một loại thực thể.
    fn data<E: Entity>(&self) -> Result<Tree, Error> {
        Ok(self.db.open_tree(E::NAME)?)
    }
    
    /// Lấy (hoặc tạo) cây chỉ mục cho một loại thực thể.
    fn index<E: Entity>(&self) -> Result<Tree, Error> {
        Ok(self.db.open_tree(format!("index_{}", E::NAME))?)
    }
    
    #[instrument(skip(self, entity), fields(r#type = std::any::type_name::<E>()))]
    fn insert<E: Entity>(&self, entity: &E) -> Result<(), Error> 
    where E::Key: Debug, E::Index: Debug
    {
        let data = self.data::<E>()?;
        let index = self.index::<E>()?;
        
        debug!("Đang tuần tự hóa thực thể để chèn");
        let bytes = bincode::serialize(entity)?;
        let summary = bincode::serialize(&entity.summary())?;
        
        let key = entity.key();
        let idx = entity.index();
        
        trace!("Bắt đầu giao dịch để chèn");
        
        let outcome = (&data, &index).transaction(|(d, i)| {
            d.insert::<&[u8], &[u8]>(key.as_ref(), bytes.as_ref())?;
            i.insert::<&[u8], &[u8]>(idx.as_ref(), summary.as_ref())?;
            Ok(())
        });

        match &outcome {
            Ok(_) => {
                debug!("Giao dịch chèn hoàn thành thành công");
                Ok(())
            },
            Err(e) => {
                warn!(error = ?e, "Giao dịch chèn thất bại");
                outcome.map_err(|e| match e {
                    sled::transaction::TransactionError::Storage(error) => Error::Store(error),
                    sled::transaction::TransactionError::Abort(error) => error,
                })
            }
        }
    }
    
    #[instrument(skip(self), fields(r#type = std::any::type_name::<E>()))]
    fn fetch<E: Entity>(&self, key: &E::Key) -> Result<Option<E>, Error> 
    where E::Key: Debug
    {
        debug!("Đang truy xuất thực thể từ kho lưu trữ");
        
        let data = self.data::<E>()?;
        let result = data.get(key.as_ref())?;
        
        match result {
            Some(ivec) => {
                debug!("Đã tìm thấy thực thể, đang giải tuần tự hóa");
                let entity = bincode::deserialize(&ivec)?;
                Ok(Some(entity))
            }
            None => {
                debug!("Không tìm thấy thực thể");
                Ok(None)
            }
        }
    }
    
    #[instrument(skip(self, transform), fields(r#type = std::any::type_name::<E>()))]
    fn update<E: Entity, F>(&self, key: &E::Key, transform: F) -> Result<E, Error>
    where
        F: FnOnce(E) -> E,
        E::Key: Debug,
        E::Index: Debug
    {
        debug!("Đang cập nhật thực thể");
        
        let data = self.data::<E>()?;
        let index = self.index::<E>()?;
        
        let buffer = data.get(key.as_ref())?.ok_or(Error::Missing)?;
        
        let before: E = bincode::deserialize(&buffer)?;
        
        let after = transform(before.clone());
        
        let outcome = (&data, &index).transaction(|(d, i)| {
            let _guard = d.get(key.as_ref())?
                .ok_or(ConflictableTransactionError::Abort(Error::Missing))?;
            
            let stale = before.index();
            let fresh = after.index();
            
            let changed = stale.as_ref() != fresh.as_ref();
            
            if changed {
                i.remove::<&[u8]>(stale.as_ref())?;
                let summary = bincode::serialize(&after.summary())
                    .map_err(|e| ConflictableTransactionError::Abort(Error::Format(e)))?;
                i.insert::<&[u8], &[u8]>(fresh.as_ref(), summary.as_ref())?;
            }

            let bytes = bincode::serialize(&after)
                .map_err(|e| ConflictableTransactionError::Abort(Error::Format(e)))?;
            d.insert::<&[u8], &[u8]>(key.as_ref(), bytes.as_ref())?;

            Ok(after.clone())
        });

        match &outcome {
            Ok(entity) => {
                debug!("Cập nhật thực thể thành công");
                Ok(entity.clone())
            },
            Err(e) => {
                warn!(error = ?e, "Cập nhật thực thể thất bại");
                outcome.map_err(|e| match e {
                    sled::transaction::TransactionError::Storage(error) => Error::Store(error),
                    sled::transaction::TransactionError::Abort(error) => error,
                })
            }
        }
    }
    
    /// Xóa một thực thể dựa trên khóa của nó, trả về thực thể đã bị xóa nếu thành công.
    #[instrument(skip(self), fields(r#type = std::any::type_name::<E>()))]
    fn delete<E: Entity>(&self, key: &E::Key) -> Result<E, Error>
    where E::Key: Debug, E::Index: Debug
    {
        debug!("Đang xóa thực thể");
        
        // Cập nhật từ data_tree thành data và index_tree thành index
        let data = self.data::<E>()?;
        let index = self.index::<E>()?;
        
        let outcome = (&data, &index).transaction(|(d, i)| {
            let buffer = d.get(key.as_ref())?
                .ok_or(ConflictableTransactionError::Abort(Error::Missing))?;
            
            // Giải tuần tự hóa
            let entity: E = bincode::deserialize(&buffer)
                .map_err(|e| ConflictableTransactionError::Abort(Error::Format(e)))?;
            
            // Xóa cả bản ghi từ cây dữ liệu và cây chỉ mục
            d.remove(key.as_ref())?;
            i.remove(entity.index().as_ref())?;
            
            Ok(entity)
        });

        match &outcome {
            Ok(entity) => {
                debug!("Xóa thực thể thành công");
                Ok(entity.clone())
            },
            Err(e) => {
                warn!(error = ?e, "Xóa thực thể thất bại");
                outcome.map_err(|e| match e {
                    sled::transaction::TransactionError::Storage(error) => Error::Store(error),
                    sled::transaction::TransactionError::Abort(error) => error,
                })
            }
        }
    }
    
    /// Truy vấn thực thể sử dụng chỉ mục bao phủ.
    ///
    /// Phương thức này tận dụng chỉ mục bao phủ để trả về một Stream các bản tóm tắt thực thể
    /// mà không cần truy cập vào dữ liệu đầy đủ, làm tăng hiệu suất đáng kể.
    #[instrument(skip(self, query), fields(r#type = std::any::type_name::<E>()))]
    fn query<E: Entity>(&self, query: Query<E::Index>) -> Result<impl Iterator<Item=Result<E::Summary, Error>>, Error>
    where E::Key: Debug, E::Index: Debug
    {
        debug!("Thực hiện truy vấn dựa trên chỉ mục");
        
        // Cập nhật từ index_tree thành index
        let index = self.index::<E>()?;
        
        // Thiết lập giới hạn trên cho phạm vi tìm kiếm
        let lower = query.prefix.clone();
        let mut upper = query.prefix.clone();
        
        // Nếu có giá trị prefix, tạo giới hạn trên bằng cách tăng byte cuối lên 1
        if !upper.is_empty() {
            let last = upper.len() - 1;
            upper[last] = upper[last].saturating_add(1);
        }
        
        debug!("Thiết lập phạm vi truy vấn");
        let mut iter = if upper.is_empty() {
            // Nếu không có giới hạn trên, lấy tất cả
            index.iter()
        } else {
            // Nếu có giới hạn, thực hiện truy vấn phạm vi
            index.range(lower..upper)
        };
        
        // Nếu có 'after', bỏ qua các mục trước nó
        if let Some(after) = query.after {
            while let Some(Ok((k, _))) = iter.next() {
                if k.as_ref() == after.as_ref() {
                    break;
                }
            }
        }
        
        // Tạo một iterator và ánh xạ các giá trị để giải mã thành Summary
        let result = iter
            .take(query.limit)
            .map(|res| -> Result<E::Summary, Error> {
                let (_, v) = res?;
                let summary = bincode::deserialize(&v)?;
                Ok(summary)
            });
        
        debug!("Truy vấn thực hiện thành công");
        Ok(result)
    }
    
    /// Chèn nhiều thực thể cùng một lúc.
    #[instrument(skip(self, iterator), fields(r#type = std::any::type_name::<E>()))]
    fn mass<E>(&self, mut iterator: Box<dyn Iterator<Item=E> + Send>) -> Result<(), Error>
    where 
        E: Entity,
        E::Key: Debug, 
        E::Index: Debug
    {
        debug!("Bắt đầu chèn hàng loạt");
        
        // Xử lý theo chunk để giảm áp lực bộ nhớ
        let mut count = 0;
        loop {
            // Lấy chunk kế tiếp của dữ liệu
            let chunk: Vec<_> = iterator.by_ref().take(CHUNK).collect();
            // Thay đổi: chunk_size -> size
            let size = chunk.len();
            
            if size == 0 {
                break;
            }
            
            // Thay đổi: chunk_size -> size
            debug!(size = size, "Đang xử lý chunk dữ liệu");
            
            // Chèn từng thực thể trong chunk
            for entity in chunk {
                self.insert(&entity)?;
            }
            
            // Thay đổi: chunk_size -> size
            count += size;
            debug!(processed = count, "Đã xử lý chunk dữ liệu");
            
            // Thay đổi: chunk_size -> size
            if size < CHUNK {
                break; // Đã xử lý hết
            }
        }
        
        debug!(total = count, "Hoàn thành chèn hàng loạt");
        Ok(())
    }
    
    /// Lấy các thông tin về cơ sở dữ liệu.
    ///
    /// Sửa lỗi: Thay vì sử dụng kiểu Stats không tồn tại, chúng ta trả về một cấu trúc mô tả CSDL.
    pub fn stats(&self) -> Result<String, Error> {
        // Trả về thông tin dưới dạng chuỗi mô tả thay vì kiểu Stats không tồn tại
        Ok(format!("Database size: {} bytes", self.db.size_on_disk()?))
    }
}

#[async_trait]
impl Storage for Sled {
    #[instrument(skip(self, entity), fields(entity_type = std::any::type_name::<E>()))]
    async fn insert<E: Entity>(&self, entity: E) -> Result<(), Error> 
    where E::Key: Debug, E::Index: Debug
    {
        debug!("Đang tạo tác vụ blocking cho thao tác chèn");
        let db = self.clone();
        let result = spawn_blocking(move || db.insert(&entity)).await??;
        debug!("Tác vụ chèn hoàn thành");
        Ok(result)
    }

    #[instrument(skip(self), fields(entity_type = std::any::type_name::<E>()))]
    async fn fetch<E: Entity>(&self, key: E::Key) -> Result<Option<E>, Error> 
    where E::Key: Debug
    {
        debug!("Đang tạo tác vụ blocking cho thao tác truy xuất");
        let db = self.clone();
        let result = spawn_blocking(move || db.fetch::<E>(&key)).await??;
        debug!(found = result.is_some(), "Tác vụ truy xuất hoàn thành");
        Ok(result)
    }

    #[instrument(skip(self, transform), fields(entity_type = std::any::type_name::<E>()))]
    async fn update<E: Entity, F>(&self, key: E::Key, transform: F) -> Result<E, Error>
    where
        F: FnOnce(E) -> E + Send + 'static,
        E::Key: Debug
    {
        debug!("Đang tạo tác vụ blocking cho thao tác cập nhật");
        let db = self.clone();
        let result = spawn_blocking(move || db.update::<E, _>(&key, transform)).await??;
        debug!("Tác vụ cập nhật hoàn thành");
        Ok(result)
    }

    #[instrument(skip(self), fields(entity_type = std::any::type_name::<E>()))]
    async fn delete<E: Entity>(&self, key: E::Key) -> Result<E, Error> 
    where E::Key: Debug
    {
        debug!("Đang tạo tác vụ blocking cho thao tác xóa");
        let db = self.clone();
        let result = spawn_blocking(move || db.delete::<E>(&key)).await??;
        debug!("Tác vụ xóa hoàn thành");
        Ok(result)
    }

    #[instrument(skip(self, query), fields(entity_type = std::any::type_name::<E>()))]
    async fn query<E: Entity>(&self, query: Query<E::Index>) 
        -> Result<Box<dyn Iterator<Item = Result<E::Summary, Error>> + Send>, Error> 
    where E::Key: Debug, E::Index: Debug
    {
        debug!("Đang tạo tác vụ blocking cho thao tác truy vấn");
        
        // Lưu trữ tham chiếu
        let this = self.clone();
        
        // Thực hiện truy vấn trong một tác vụ blocking
        let result = spawn_blocking(move || {
            this.query::<E>(query)
        }).await??;
        
        // Bọc kết quả trong Box để khớp với signature trả về
        Ok(Box::new(result))
    }

    #[cfg(any(test, feature = "testing"))]
    #[instrument(skip(self, query), fields(r#type = std::any::type_name::<E>()))]
    async fn keys<E: Entity>(&self, query: Query<E::Index>) 
        -> Result<Box<dyn Iterator<Item = Result<Vec<u8>, Error>> + Send>, Error> 
    where E::Index: Debug
    {
        debug!("Đang tạo tác vụ blocking cho thao tác truy vấn khóa");
        let db = self.clone();
        
        let result = spawn_blocking(move || {
            let index = db.index::<E>()?;
            
            let lower = query.prefix.clone();
            let mut upper = query.prefix.clone();
            
            if !upper.is_empty() {
                let last = upper.len() - 1;
                upper[last] = upper[last].saturating_add(1);
            }
            
            let mut iter = if upper.is_empty() {
                index.iter()
            } else {
                index.range(lower..upper)
            };
            
            if let Some(after) = query.after {
                while let Some(Ok((k, _))) = iter.next() {
                    if k.as_ref() == after.as_ref() {
                        break;
                    }
                }
            }
            
            let result = iter
                .take(query.limit)
                .map(|res| -> Result<Vec<u8>, crate::Error> {
                    let (k, _) = res?;
                    Ok(k.to_vec())
                });
            
            let boxed: Box<dyn Iterator<Item = Result<Vec<u8>, crate::Error>> + Send> = Box::new(result);
            Ok::<Box<dyn Iterator<Item = Result<Vec<u8>, crate::Error>> + Send>, crate::Error>(boxed)
        }).await??;
        
        debug!("Tác vụ truy vấn khóa hoàn thành"); 
        Ok(result)
    }

    #[instrument(skip(self, iter), fields(r#type = std::any::type_name::<E>()))]
    async fn mass<E: Entity>(&self, iter: Box<dyn Iterator<Item = E> + Send>) -> Result<(), Error> 
    where E::Key: Debug, E::Index: Debug
    {
        debug!("Đang tạo tác vụ blocking cho thao tác chèn hàng loạt");
        let db = self.clone();
        
        // Sửa: Thêm thao tác `.await?` để đợi Future hoàn thành và giải nén kết quả
        // Cần dùng `?` hai lần - một lần cho kết quả của spawn_blocking và một lần cho kết quả của mass
        spawn_blocking(move || db.mass::<E>(iter)).await??;
        
        debug!("Tác vụ chèn hàng loạt hoàn thành");
        Ok(())
    }
}

mod tests {
    use super::*;
    use crate::Id;
    use serde::{Serialize, Deserialize};
    
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
    

    #[allow(dead_code)]
    fn memory() -> Sled {
        // Sử dụng uuid để đảm bảo mỗi test có đường dẫn riêng
        // Thay đổi: unique_path -> path
        let path = format!("db/{}", uuid::Uuid::new_v4());
        Sled::new(&path).unwrap()
    }

    #[test]
    fn crud() {
        let store = memory();
        // Tạo một đối tượng duy nhất cho test
        let item = Thing { id: Id::new_v4(), name: "Test".to_string(), value: 42 };
        
        // Insert
        store.insert(&item).unwrap();
        
        // Fetch
        let fetched = store.fetch::<Thing>(&item.id).unwrap().unwrap();
        assert_eq!(item, fetched);
        
        // Update
        let updated = Thing { value: 100, ..item.clone() };
        store.insert(&updated).unwrap();
        let fetched = store.fetch::<Thing>(&item.id).unwrap().unwrap();
        assert_eq!(updated, fetched);
        
        // Delete
        let deleted = store.delete::<Thing>(&item.id).unwrap();
        assert_eq!(updated, deleted);
        
        // Verify deletion
        assert!(store.fetch::<Thing>(&item.id).unwrap().is_none());
    }
    
    #[test]
    fn bulk() {
        // Sử dụng runtime Tokio cho các phương thức async
        let rt = tokio::runtime::Runtime::new().unwrap();
        let store = memory();
        let things: Vec<_> = (0..100).map(|i| Thing {
            id: Id::new_v4(), 
            name: format!("Thing {}", i), 
            value: i,
        }).collect();
        
        // Chèn hàng loạt sử dụng phương thức async
        rt.block_on(async {
            // Sử dụng phiên bản async của mass từ trait Storage
            <Sled as Storage>::mass(&store, Box::new(things.clone().into_iter())).await.unwrap();
        });
        
        // Kiểm tra tất cả đã được chèn
        for item in &things {
            let fetched = store.fetch::<Thing>(&item.id).unwrap().unwrap();
            assert_eq!(*item, fetched);
        }
        
        // Kiểm tra truy vấn 
        let query: Query<Vec<u8>> = Query {
            prefix: vec![],
            after: None,
            limit: 1000,
        };
        
        // Có 2 phiên bản của hàm query:
        // 1. fn query(...) -> Result<impl Iterator<...>> (đồng bộ)
        // 2. async fn query(...) -> Result<Box<dyn Iterator<...>>> (bất đồng bộ, tuân theo trait Storage)
        // Để sử dụng phiên bản async, chúng ta cần chỉ định kiểu để tránh nhầm lẫn
        // Thay đổi: store_ref -> store (vì nó vẫn là Sled)
        let store = &store;
        let result = rt.block_on(async {
            // Tạo một query mới để không sử dụng lại query đã tiêu thụ
            // Thay đổi: query -> _query (để đánh dấu là không sử dụng)
            let _query: Query<Vec<u8>> = Query {
                prefix: vec![],
                after: None,
                limit: 1000,
            };
            
            // Gọi phiên bản async từ trait Storage
            // Sử dụng query đã được truyền vào hàm bulk, không phải _query mới tạo
            let outcome = <Sled as Storage>::query::<Thing>(store, query).await.unwrap(); // Sửa ở đây, dùng query từ tham số
            
            // Debug: Liệt kê từng kết quả và đếm
            let mut count = 0;
            // Thay đổi: query_result -> outcome
            for item in outcome {
                // Chỉ đếm các mục thành công, bỏ qua các lỗi nếu có
                if item.is_ok() {
                    count += 1;
                }
            }
            
            count
        });
        
        assert_eq!(result, 100);
    }
}
