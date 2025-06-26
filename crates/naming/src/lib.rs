pub mod rules;
pub mod helper;
use helper::{warn::Warn, stat::Stat, conf::Conf};
use crate::rules::metric::{Metric, Detail};
use crate::helper::file;

// Tích hợp cảnh báo, thống kê, config động vào pipeline kiểm tra
pub fn run(files: Vec<String>, conf_path: &str) {
    let mut warn = Warn::new();
    let mut stat = Stat::new();
    let mut conf = Conf::new();
    conf.load(conf_path);
    let n = files.len() as u64;
    for file in &files {
        let t0 = std::time::Instant::now();
        // Kiểm tra ignore động
        if conf.ignore(file) { continue; }
        // ... kiểm tra file ...
        // Giả lập: nếu file chứa "slow" thì add_slow
        if file.contains("slow") {
            stat.slow(file);
        }
        // Giả lập: nếu file chứa "peak" thì add_mem
        if file.contains("peak") {
            stat.mem(1000);
        }
        // Giả lập: nếu file chứa "dup" thì cảnh báo duplicate
        if file.contains("dup") {
            warn.add("duplicate: found");
        }
        // Giả lập: nếu file chứa "long" thì cảnh báo length
        if file.contains("long") {
            warn.add("length: too long");
        }
        let dt = t0.elapsed().as_millis() as u64;
        stat.val(dt);
        if dt > 500 {
            stat.slow(file);
            warn.add("slow: file");
        }
    }
    stat.mean(n);
    stat.stop();
    stat.log();
    warn.log();
    // Cảnh báo vi phạm tăng đột biến
    if warn.msg().len() as u64 > n/2 {
        println!("[ALERT] Vi phạm tăng đột biến!");
    }
    // Log rule bật/tắt
    for r in conf.rule() {
        println!("[RULE] enable: {r}");
    }
    for r in &conf.disable {
        println!("[RULE] disable: {r}");
    }
}

// Tích hợp tự động phát hiện folder/file cho pipeline
pub fn auto(input: &str, conf_path: &str) {
    let mut files = vec![];
    if file::dir(input) {
        let mut v = vec![];
        let path = std::path::Path::new(input);
        if file::scan(path, &mut v).is_ok() {
            files = v.into_iter().filter(|f| file::ext(f).as_deref()==Some("rs")).collect();
        }
    } else if file::file(input) {
        files.push(input.to_string());
    }
    run(files, conf_path);
}

/// Processes a given file or directory path to check for naming violations.
///
/// This function will automatically detect if the input path is a file or a directory.
/// For a directory, it recursively scans for `.rs` files.
/// It then processes all found files in parallel using a Rayon thread pool.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path to the file or directory.
/// * `_conf` - A string slice for the configuration file (currently unused).
///
/// # Returns
///
/// A `Result` which is:
/// * `Ok((Vec<Metric>, Vec<Detail>))` on success.
///   - The first element is a vector of `Metric` objects for each file.
///   - The second element is a vector of `Detail` objects for each file.
/// * `Err(String)` on failure, e.g., if the path does not exist.
pub fn process(path: &str, _conf: &str) -> Result<(Vec<Metric>, Vec<Detail>), String> {
    let mut files = vec![];
    if file::dir(path) {
        let mut v = vec![];
        let p = std::path::Path::new(path);
        if file::scan(p, &mut v).is_ok() {
            files = v.into_iter().filter(|f| file::ext(f).as_deref() == Some("rs")).collect();
        }
    } else if file::file(path) {
        files.push(path.to_string());
    } else {
        return Err(format!("Input path not found: {}", path));
    }

    // Process files in parallel
    let (tx, rx) = std::sync::mpsc::channel();
    rayon::scope(move |s| {
        for file in files {
            let tx = tx.clone();
            s.spawn(move |_| {
                let (metric, details) = crate::rules::metric(&file);
                let _ = tx.send((metric, details));
            });
        }
    });

    let results: Vec<_> = rx.iter().collect();
    let mut all_metrics = Vec::with_capacity(results.len());
    let mut all_details = Vec::new();

    for (metric, details) in results {
        all_metrics.push(metric);
        all_details.extend(details);
    }
    
    Ok((all_metrics, all_details))
}
