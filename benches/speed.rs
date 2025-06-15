// benches/speed.rs

use once_cell::sync::Lazy; // Sử dụng once_cell để khởi tạo runtime một lần
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, BatchSize, PlotConfiguration, AxisScale, Bencher};
use bedrock::{self, Sled, Id, Error, todo::{self, Summary, Todo}, Patch, Storage}; 
use tempfile::tempdir; 

use tokio::runtime::{Runtime, Builder};


// Tạo một Tokio runtime toàn cục để sử dụng trong các benchmark
static RT: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});

// Hàm tiện ích để lấy tham chiếu đến runtime
fn rt() -> &'static Runtime {
    &*RT
}

struct BenchStore {
    store: Sled,
    _dir: tempfile::TempDir, // Giữ TempDir để nó không bị xóa sớm
    // Không cần runtime ở đây nữa vì chúng ta dùng RT toàn cục
}

/// Truy vấn và trả về các đối tượng Todo đầy đủ.
fn fetch(store: &BenchStore, status: bool, limit: usize) -> Result<Vec<Todo>, Error> {
    let query = todo::filter(status, None, limit);
    // Sử dụng rt() đã định nghĩa
    // Lời gọi store.store.query sẽ hoạt động sau khi import Storage
    let summaries: Vec<_> = rt().block_on(async { store.store.query::<Todo>(query).await })?
        .collect::<Result<Vec<_>, _>>()?;
    
    let mut todos = Vec::with_capacity(summaries.len());
    for summary in summaries {
        // Sử dụng rt()
        let todo = rt().block_on(async { bedrock::todo::find(&store.store, summary.id).await })?;
        todos.push(todo);
    }
    Ok(todos)
}

/// Truy vấn và chỉ trả về các bản tóm tắt (Summary).
fn list(store: &BenchStore, done: bool, limit: usize) -> Result<Vec<Summary>, Error> {
    let query = todo::filter(done, None, limit);
    // Sử dụng rt()
    // Lời gọi store.store.query sẽ hoạt động sau khi import Storage
    let results = rt().block_on(async { store.store.query::<Todo>(query).await })?;
    let summaries: Vec<_> = results.collect::<Result<Vec<_>, _>>()?;
    Ok(summaries)
}

/// Thiết lập cơ sở dữ liệu với một số lượng bản ghi cụ thể.
fn prepare(count: usize) -> BenchStore {
    let dir = tempdir().unwrap(); // Đã import
    let path = dir.path().to_str().unwrap().to_string();
    let store = Sled::new(&path).unwrap();

    let todos = (0..count).map(|i| Todo {
        id: Id::new_v4(),
        text: format!("Công việc mẫu {}", i),
        done: i % 2 == 0,
        created: todo::now() + i as u128,
    });

    // Sử dụng rt()
    rt().block_on(async {
        bedrock::todo::bulk(&store, todos).await.unwrap();
    });
    
    BenchStore { store, _dir: dir } // _dir để giữ tempdir
}

fn bench(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>, size: usize) {
    let store = prepare(size);
    let limit = std::cmp::min(size, 100); 

    group.bench_function(BenchmarkId::new("add", size), |b: &mut Bencher| {
        // Sửa cách gọi benchmark bất đồng bộ
        b.iter_batched(
            || format!("Công việc benchmark {}", rand::random::<u32>()),
            |text| rt().block_on(bedrock::todo::add(&store.store, text)),
            BatchSize::SmallInput,
        );
    });

    if size > 0 {
        // Thay đổi: existing_summaries -> summaries
        let summaries = list(&store, false, 1).expect("Không thể lấy summary để test");
        // Thay đổi: id_to_use -> id
        let id = if !summaries.is_empty() {
            summaries[0].id
        } else {
            // Thay đổi: done_summaries -> summaries
            let summaries = list(&store, true, 1).expect("Không thể lấy summary (done) để test");
            if !summaries.is_empty() {
                summaries[0].id
            } else {
                // Sử dụng rt()
                // Thay đổi: temp_todo -> todo
                let todo = rt().block_on(bedrock::todo::add(&store.store, "temp".to_string())).unwrap();
                // Thay đổi: temp_todo -> todo
                todo.id
            }
        };

        group.bench_function(BenchmarkId::new("find", size), |b: &mut Bencher| {
            // Sửa cách gọi benchmark bất đồng bộ
            // Thay đổi: id_to_use -> id
            b.iter(|| rt().block_on(bedrock::todo::find(&store.store, id)));
        });

        group.bench_function(BenchmarkId::new("change", size), |b: &mut Bencher| {
            // Sửa cách gọi benchmark bất đồng bộ
            let patch = Patch { text: Some("đã cập nhật".to_string()), done: Some(true) }; // Patch đã được import
            // Thay đổi: id_to_use -> id
            b.iter(|| rt().block_on(bedrock::todo::change(&store.store, id, patch.clone())));
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
    // Thay đổi: store_small -> store (sử dụng shadowing)
    let store = prepare(100);
    // So sánh query_summary
    group.bench_function("summary_small", |b| b.iter(|| list(&store, false, 50)));
    // Thay đổi: store_medium -> store (sử dụng shadowing)
    let store = prepare(1_000);
    group.bench_function("summary_medium", |b| b.iter(|| list(&store, false, 50)));
    // Thay đổi: store_large_comp -> store (sử dụng shadowing)
    let store = prepare(10_000);
    group.bench_function("summary_large", |b| b.iter(|| list(&store, false, 50)));

    // So sánh query_full
    // Sử dụng lại shadowing cho các store ở đây
    let store = prepare(100); // store_small
    group.bench_function("full_small", |b| b.iter(|| fetch(&store, false, 50)));
    let store = prepare(1_000); // store_medium
    group.bench_function("full_medium", |b| b.iter(|| fetch(&store, false, 50)));
    let store = prepare(10_000); // store_large_comp
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
            // Sửa cách gọi benchmark bất đồng bộ
            b.iter_batched(
                || format!("Công việc benchmark {}", rand::random::<u32>()),
                |text| rt().block_on(bedrock::todo::add(&store.store, text)),
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
                b.iter(|| rt().block_on(bedrock::todo::find(&store.store, local_id)));
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