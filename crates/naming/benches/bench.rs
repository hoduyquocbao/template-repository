use criterion::{criterion_group, criterion_main, Criterion};
use naming::process;
use std::path::Path;
use std::fs;

fn setup_test_files() {
    let test_dir = "benches/src";
    if Path::new(test_dir).exists() {
        fs::remove_dir_all(test_dir).unwrap();
    }
    fs::create_dir_all(test_dir).unwrap();
    
    // Create a few dummy files with some content to process
    for i in 0..5 {
        let file_path = Path::new(test_dir).join(format!("test_file_{}.rs", i));
        let content = r#"
            // This is a test file for benchmarking.
            struct MyStruct {
                field_one: i32,
                another_field: String,
            }

            fn my_function(arg_one: &str) -> bool {
                println!("Hello, {}!", arg_one);
                true
            }

            fn anotherFunction() {
                let snake_case = 1;
                let PascalCase = 2;
                let camelCase = 3;
            }
        "#;
        fs::write(file_path, content).unwrap();
    }
}

fn bench_process_folder(c: &mut Criterion) {
    setup_test_files();
    c.bench_function("process_folder", |b| {
        b.iter(|| {
            if let Err(e) = process("benches/src", "naming.toml") {
                eprintln!("Error during benchmark: {}", e);
            }
        })
    });
}

criterion_group!(benches, bench_process_folder);
criterion_main!(benches); 