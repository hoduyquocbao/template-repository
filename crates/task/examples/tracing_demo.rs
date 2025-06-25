use repository::{self, Sled, Id, Error, Extension};
use tracing::{debug, info, trace_span, warn, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use task::{Status, Priority};

/// Ví dụ minh họa cách sử dụng tracing trong ứng dụng repository
///
/// Ví dụ này chứng minh:
/// 1. Cách cấu hình tracing với các mức độ chi tiết khác nhau
/// 2. Cách sử dụng span để nhóm các hoạt động liên quan
/// 3. Cách ghi nhật ký có cấu trúc với các trường dữ liệu
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Thiết lập tracing với bộ lọc nâng cao
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            EnvFilter::from_default_env()
                .add_directive(Level::INFO.into()) // Mức mặc định
                .add_directive("repository=debug".parse().map_err(Error::parse)?) // Chi tiết hơn cho ứng dụng
                .add_directive("sled=warn".parse().map_err(Error::parse)?) // Chỉ cảnh báo cho sled
        )
        .init();

    info!("Bắt đầu demo tracing");

    // Tạo cơ sở dữ liệu tạm thời cho ví dụ này
    let temp = tempfile::tempdir().map_err(Error::Io)?; // temp thay cho temp_dir
    let path = temp.path().to_str().unwrap(); // path thay cho db_path
    let store = Sled::new(path)?;

    // Tạo một span tùy chỉnh cho chuỗi hoạt động này
    let span = trace_span!("demo", path = %path); // span thay cho demo_span
    let _guard = span.enter();

    // Thêm một số công việc
    let entry1 = task::add(&store, "Học về tracing".to_string(), "module1".to_string(), "task1".to_string(), Priority::High, Status::Open, "assignee1".to_string(), "2024-05-01".to_string(), "notes1".to_string()).await?;
    let entry2 = task::add(&store, "Triển khai khả năng quan sát".to_string(), "module2".to_string(), "task2".to_string(), Priority::High, Status::Open, "assignee2".to_string(), "2024-05-02".to_string(), "notes2".to_string()).await?;
    let entry3 = task::add(&store, "Giám sát trong môi trường sản xuất".to_string(), "module3".to_string(), "task3".to_string(), Priority::High, Status::Open, "assignee3".to_string(), "2024-05-03".to_string(), "notes3".to_string()).await?;

    info!("Đã thêm 3 công việc, giờ lấy lại");

    // Truy vấn các công việc
    let prefix = vec![(&Status::Open).into()];
    let query_obj = shared::query(prefix, None::<Vec<u8>>, 10);
    let summaries = task::query(&store, query_obj).await?;

    let tasks: Vec<_> = summaries.collect::<Result<Vec<_>, _>>()?;
    debug!(count = tasks.len(), "Truy xuất công việc thành công");

    // Đánh dấu một công việc là hoàn thành
    let patch = task::Patch {
        status: Some(Status::Done),
        ..Default::default()
    };

    info!(id = %entry2.id, "Đánh dấu công việc là hoàn thành");
    task::change(&store, entry2.id, patch).await?;

    // Thử tìm một công việc không tồn tại
    let uuid = Id::new_v4(); // uuid thay cho non_existent_id
    match task::find(&store, uuid).await {
        Ok(_) => unreachable!("Điều này không nên thành công"),
        Err(e) => warn!(id = %uuid, error = ?e, "Lỗi dự kiến khi tìm kiếm công việc không tồn tại"),
    }

    // Dọn dẹp
    info!("Dọn dẹp các công việc demo");
    task::remove(&store, entry1.id).await?;
    task::remove(&store, entry2.id).await?;
    task::remove(&store, entry3.id).await?;

    info!("Demo tracing hoàn thành thành công");
    Ok(())
}
