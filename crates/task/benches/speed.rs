// benches/speed.rs

use once_cell::sync::Lazy; // Sử dụng once_cell để khởi tạo runtime một lần
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, BatchSize, PlotConfiguration, AxisScale, Bencher};
use repository::{self, Sled, Id, Error,  Storage}; 
use tempfile::tempdir; 
use task::{Entry, Patch, Status, Priority, Summary};
use tokio::runtime::{Runtime, Builder};
use shared::query;


// Tạo một Tokio runtime toàn cục để sử dụng trong các benchmark
static RT: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});

// Hàm tiện ích để lấy tham chiếu đến runtime
fn rt() -> &'static Runtime {
    &RT
}

struct BenchStore {
    store: Sled,
    _dir: tempfile::TempDir, // Giữ TempDir để nó không bị xóa sớm
    // Không cần runtime ở đây nữa vì chúng ta dùng RT toàn cục
}

/// Truy vấn và trả về các đối tượng Task đầy đủ.
fn fetch(store: &BenchStore, status: bool, limit: usize) -> Result<Vec<Entry>, Error> {
    let prefix = vec![(&if status { Status::Done } else { Status::Open }).into()];
    let query_obj = query(prefix, None::<Vec<u8>>, limit);
    let summaries: Vec<_> = rt().block_on(async { store.store.query::<Entry>(query_obj).await })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut tasks = Vec::with_capacity(summaries.len());
    for summary in summaries {
        let task = rt().block_on(async { task::find(&store.store, summary.id).await })?;
        tasks.push(task);
    }
    Ok(tasks)
}

/// Truy vấn và chỉ trả về các bản tóm tắt (Summary).
fn list(store: &BenchStore, status: bool, limit: usize) -> Result<Vec<Summary>, Error> {
    let prefix = vec![(&if status { Status::Done } else { Status::Open }).into()];
    let query_obj = query(prefix, None::<Vec<u8>>, limit);
    let results = rt().block_on(async { store.store.query::<Entry>(query_obj).await })?;
    let summaries: Vec<_> = results.collect::<Result<Vec<_>, _>>()?;
    Ok(summaries)
}

/// Thiết lập cơ sở dữ liệu với một số lượng bản ghi cụ thể.
fn prepare(count: usize) -> BenchStore {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap().to_string();
    let store = Sled::new(&path).unwrap();
    let entries = (0..count).map(|i| Entry {
        id: Id::new_v4(),
        context: "bench".to_string(),
        module: "mod".to_string(),
        task: format!("Công việc mẫu {}", i),
        priority: if i % 2 == 0 { Priority::High } else { Priority::Medium },
        status: if i % 2 == 0 { Status::Open } else { Status::Done },
        assignee: "bench".to_string(),
        due: "2025-01-01".to_string(),
        notes: "benchmark".to_string(),
        created: repository::now(),
    });
    rt().block_on(async {
        task::bulk(&store, entries).await.unwrap();
    });
    BenchStore { store, _dir: dir }
}

fn bench(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>, size: usize) {
    let store = prepare(size);
    let limit = std::cmp::min(size, 100);
    group.bench_function(BenchmarkId::new("add", size), |b: &mut Bencher| {
        b.iter_batched(
            || Entry {
                id: Id::new_v4(),
                context: "bench".to_string(),
                module: "mod".to_string(),
                task: "Benchmark add".to_string(),
                priority: Priority::High,
                status: Status::Open,
                assignee: "bench".to_string(),
                due: "2025-01-01".to_string(),
                notes: "benchmark".to_string(),
                created: repository::now(),
            },
            |entry| rt().block_on(task::add(
                &store.store,
                entry.context,
                entry.module,
                entry.task,
                entry.priority,
                entry.status,
                entry.assignee,
                entry.due,
                entry.notes,
            )),
            BatchSize::SmallInput,
        );
    });
    if size > 0 {
        let summaries = list(&store, false, 1).expect("Không thể lấy summary để test");
        let id = if !summaries.is_empty() {
            summaries[0].id
        } else {
            let entry = Entry {
                id: Id::new_v4(),
                context: "bench".to_string(),
                module: "mod".to_string(),
                task: "temp".to_string(),
                priority: Priority::High,
                status: Status::Open,
                assignee: "bench".to_string(),
                due: "2025-01-01".to_string(),
                notes: "benchmark".to_string(),
                created: repository::now(),
            };
            let task = rt().block_on(task::add(
                &store.store,
                entry.context,
                entry.module,
                entry.task,
                entry.priority,
                entry.status,
                entry.assignee,
                entry.due,
                entry.notes,
            )).unwrap();
            task.id
        };
        group.bench_function(BenchmarkId::new("find", size), |b: &mut Bencher| {
            b.iter(|| rt().block_on(task::find(&store.store, id)));
        });
        group.bench_function(BenchmarkId::new("change", size), |b: &mut Bencher| {
            let patch = Patch { status: Some(Status::Done), ..Default::default() };
            b.iter(|| rt().block_on(task::change(&store.store, id, patch.clone())));
        });
    }
    group.bench_function(BenchmarkId::new("query_summary", size), |b| {
        b.iter(|| {
            match list(&store, false, limit) {
                Ok(_) => (),
                Err(e) => eprintln!("Lỗi khi query_summary: {:?}", e),
            }
        });
    });
    group.bench_function(BenchmarkId::new("query_full", size), |b| {
        b.iter(|| {
            match fetch(&store, false, limit) {
                Ok(_) => (),
                Err(e) => eprintln!("Lỗi khi query_full: {:?}", e),
            }
        });
    });
}

fn compare(c: &mut Criterion) {
    let mut group = c.benchmark_group("QueryComparison");
    // So sánh query_summary
    let store = prepare(100);
    group.bench_function("summary_small", |b| b.iter(|| list(&store, false, 50)));
    let store = prepare(1_000);
    group.bench_function("summary_medium", |b| b.iter(|| list(&store, false, 50)));
    let store = prepare(10_000);
    group.bench_function("summary_large", |b| b.iter(|| list(&store, false, 50)));
    // So sánh query_full
    let store = prepare(100);
    group.bench_function("full_small", |b| b.iter(|| fetch(&store, false, 50)));
    let store = prepare(1_000);
    group.bench_function("full_medium", |b| b.iter(|| fetch(&store, false, 50)));
    let store = prepare(10_000);
    group.bench_function("full_large", |b| b.iter(|| fetch(&store, false, 50)));
    group.finish();
}

// Thay đổi: criterion_benchmark -> benchmarks
pub fn benchmarks(c: &mut Criterion) {
    // Các kích thước cơ sở dữ liệu nhỏ để chạy nhanh
    let large = std::env::var("BENCH_LARGE").is_ok();
    // Thay đổi: run_extreme -> extreme
    let extreme = std::env::var("BENCH_EXTREME").is_ok();

    let mut group = c.benchmark_group("CRUD");
    // Thay đổi: run_benches_for_size -> bench
    bench(&mut group, 10);
    // Thay đổi: run_benches_for_size -> bench
    bench(&mut group, 100);
    // Thay đổi: run_benches_for_size -> bench
    bench(&mut group, 1_000);
    // Thay đổi: run_large -> large
    if large {
        // Thay đổi: run_benches_for_size -> bench
        bench(&mut group, 10_000);
        // Thay đổi: run_benches_for_size -> bench
        bench(&mut group, 100_000);
    }
    // Thay đổi: run_extreme -> extreme
    if extreme {
        // Thay đổi: run_benches_for_size -> bench
        bench(&mut group, 1_000_000);
    }
    group.finish();
    
    // Thay đổi: bench_query_comparison -> compare
    compare(c); 
}

fn scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("Scalability");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic)); // Đã import
    let large = std::env::var("BENCH_LARGE").is_ok();
    let extreme = std::env::var("BENCH_EXTREME").is_ok();

    let sizes = if extreme {
        vec![10, 100, 1_000, 10_000, 100_000, 1_000_000]
    } else if large {
        vec![10, 100, 1_000, 10_000, 100_000]
    } else {
        vec![10, 100, 1_000]
    };

    for size_val in sizes {
        let store = prepare(size_val);
        let limit = std::cmp::min(size_val, 100);

        group.bench_with_input(BenchmarkId::new("add_scale", size_val), &size_val, |b: &mut Bencher, &_s| {
            b.iter_batched(
                || Entry {
                    id: Id::new_v4(),
                    context: "bench".to_string(),
                    module: "mod".to_string(),
                    task: format!("Công việc benchmark {}", rand::random::<u32>()),
                    priority: Priority::High,
                    status: Status::Open,
                    assignee: "bench".to_string(),
                    due: "2025-01-01".to_string(),
                    notes: "benchmark".to_string(),
                    created: repository::now(),
                },
                |entry| rt().block_on(task::add(
                    &store.store,
                    entry.context,
                    entry.module,
                    entry.task,
                    entry.priority,
                    entry.status,
                    entry.assignee,
                    entry.due,
                    entry.notes,
                )),
                BatchSize::SmallInput,
            );
        });
        
        if size_val > 0 {
            // Thay đổi: existing_summaries -> summaries
            let summaries = list(&store, false, 1).unwrap_or_default();
            // Thay đổi: id_to_use -> id
            let id = if !summaries.is_empty() {
                summaries[0].id
            } else { Id::new_v4() }; // ID giả nếu không có gì

            group.bench_with_input(BenchmarkId::new("find_scale", size_val), &id, |b: &mut Bencher, &local_id| { // đổi tên id ở đây để tránh xung đột với id bên ngoài nếu có
                // Sửa cách gọi benchmark bất đồng bộ
                b.iter(|| rt().block_on(task::find(&store.store, local_id)));
            });
        }

        group.bench_with_input(BenchmarkId::new("query_summary_scale", size_val), &limit, |b, &l| {
            b.iter(|| list(&store, false, l));
        });
    }
    group.finish();
}

// Thay đổi: criterion_benchmark -> benchmarks
criterion_group!(benches, benchmarks, scale); // Thêm nhóm scale vào đây
criterion_main!(benches);