// use naming::rules;
use naming::rules::report;
use naming::{process};

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    // Hỗ trợ: naming <path> [--stdout|--report] [--metric]
    if args.len() < 2 || args.len() > 4 {
        eprintln!("Usage: naming <path> [--stdout|--report] [--metric]");
        process::exit(2);
    }

    let path = &args[1];
    // Xác định chế độ xuất kết quả
    let mut stdout = true;
    let mut report = false;
    let mut metric = false;
    for arg in &args[2..] {
        match arg.as_str() {
            "--stdout" | "-s" => { report = false; },
            "--report" | "-r" => { report = true; stdout = false; },
            "--metric" | "-m" => { metric = true; },
            _ => {
                eprintln!("Tham số không hợp lệ: {}. Dùng --stdout, --report, --metric", arg);
                process::exit(2);
            }
        }
    }

    match process(path, "naming.toml") {
        Ok((metrics, details)) => {
            // Đọc whitelist từ naming.toml ở thư mục gốc
            let config = naming::rules::read("naming.toml");
            let whitelist = config.whitelist.unwrap_or_default();
            // In ra terminal nếu được chọn
            if stdout {
                // Nếu có --metric thì in metrics tổng quan
                if metric {
                    for m in &metrics {
                        println!("[METRIC] {}: line={}ms ast={}ms total={}ms violations={} peak={}KB",
                            m.file, m.line.as_millis(), m.ast.as_millis(), m.total.as_millis(), m.violations, m.peak
                        );
                    }
                    println!("\nKiểm tra hoàn tất.{}",
                        if report { " Báo cáo đã ghi." } else { "" }
                    );
                }
                // Luôn in chi tiết các vi phạm (nếu có)
                let mut found = false;
                for (file, line, name, kind) in &details {
                    // Bỏ qua nếu name nằm trong whitelist
                    if whitelist.iter().any(|w| w == name) {
                        continue;
                    }
                    // Bỏ qua các dòng không phải vi phạm (ví dụ: metrics không có lỗi)
                    if kind != "PascalCase" && kind != "camelCase" && kind != "snake_case" && kind != "Duplicate" && kind != "Blacklist" && kind != "Length" && kind != "Variant" {
                        continue;
                    }
                    found = true;
                    let line = line.map(|l| l.to_string()).unwrap_or("-".to_string());
                    println!("[VIOLATION] {}:{} {} ({})", file, line, name, kind);
                }
                if !found {
                    println!("Không có vi phạm naming nào.");
                }
            }
            // Ghi báo cáo file nếu được chọn
            if report {
                if let Err(e) = report::csv(&metrics, "naming_report.csv") { eprintln!("Lỗi CSV: {e}"); }
                if let Err(e) = report::md(&metrics, "naming_report.md") { eprintln!("Lỗi MD: {e}"); }
                if let Err(e) = report::json(&metrics, "naming_report.json") { eprintln!("Lỗi JSON: {e}"); }
                if let Err(e) = report::detail(&details, "naming_detail.csv") { eprintln!("Lỗi Detail: {e}"); }
            }
        }
        Err(e) => {
            eprintln!("Lỗi: {}", e);
            process::exit(1);
        }
    }
}
