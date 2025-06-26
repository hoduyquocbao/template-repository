// use naming::rules;
use naming::rules::report;
use naming::{process};

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: naming <path>");
        process::exit(2);
    }

    let path = &args[1];

    match process(path, "naming.toml") {
        Ok((metrics, details)) => {
            for m in &metrics {
                 println!("[METRIC] {}: line={}ms ast={}ms total={}ms violations={} peak={}KB",
                    m.file, m.line.as_millis(), m.ast.as_millis(), m.total.as_millis(), m.violations, m.peak
                );
            }
            if let Err(e) = report::csv(&metrics, "naming_report.csv") { eprintln!("Lỗi CSV: {e}"); }
            if let Err(e) = report::md(&metrics, "naming_report.md") { eprintln!("Lỗi MD: {e}"); }
            if let Err(e) = report::json(&metrics, "naming_report.json") { eprintln!("Lỗi JSON: {e}"); }
            if let Err(e) = report::detail(&details, "naming_detail.csv") { eprintln!("Lỗi Detail: {e}"); }
            println!("\nKiểm tra hoàn tất. Báo cáo đã ghi.");
        }
        Err(e) => {
            eprintln!("Lỗi: {}", e);
            process::exit(1);
        }
    }
}
