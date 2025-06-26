use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::rules::Config;
use std::collections::HashSet;
use once_cell::sync::Lazy;
use crate::helper::text;

static PATTERNS: Lazy<Vec<(regex::Regex, &'static str)>> = Lazy::new(|| vec![
    (regex::Regex::new(r"^(struct|trait|enum|union|type)\s+([A-Z][A-Za-z0-9]*)").unwrap(), "PascalCase"),
    (regex::Regex::new(r"^(pub\s+)?(let|fn|const|static)\s+(mut\s+)?([a-z]+[A-Z][a-zA-Z0-9]*)").unwrap(), "camelCase"),
    (regex::Regex::new(r"^(pub\s+)?(let|fn|const|static)\s+(mut\s+)?([a-z0-9]+_[a-z0-9_]+)").unwrap(), "snake_case"),
]);

/// Kiểm tra từng dòng, chỉ giữ hash các dòng vi phạm, không nạp toàn bộ file vào RAM
pub fn scan(file: &str, config: &crate::rules::Config, found: &mut bool, out: &mut Vec<(Option<usize>, String, &'static str)>) -> Result<(), String> {
    let f = File::open(file).map_err(|e| format!("Không mở được file {file}: {e}"))?;
    let reader = BufReader::new(f);
    let mut seen: HashSet<u64> = HashSet::new();
    let mut count = std::collections::HashMap::new();
    let mut lines = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(e) => return Err(format!("Lỗi đọc dòng {i} file {file}: {e}")),
        };
        if let Some((name, kind)) = extract(&line, config) {
            // Kiểm tra enable rule
            if (kind == "PascalCase" && config.pascal == Some(false))
                || (kind == "snake_case" && config.snake == Some(false))
                || (kind == "camelCase" && config.camel == Some(false)) {
                continue;
            }
            // Kiểm tra độ dài định danh
            if config.length.unwrap_or(true) {
                if let Some(min) = config.min {
                    if text::len(&name) < min {
                        out.push((Some(i+1), name.clone(), "Length"));
                        *found = true;
                    }
                }
                if let Some(max) = config.max {
                    if text::len(&name) > max {
                        out.push((Some(i+1), name.clone(), "Length"));
                        *found = true;
                    }
                }
            }
            let hash = text::hash(&line);
            if !seen.contains(&hash) {
                out.push((Some(i+1), name.clone(), kind));
                seen.insert(hash);
                *found = true;
            }
            // Đếm số lần xuất hiện định danh
            *count.entry(name.clone()).or_insert(0) += 1;
            lines.push((i+1, name));
        }
    }
    // Cảnh báo định danh trùng lặp
    for (name, c) in count.iter() {
        if *c > 1 {
            for (line, n) in &lines {
                if n == name {
                    out.push((Some(*line), name.clone(), "Duplicate"));
                }
            }
            *found = true;
        }
    }
    Ok(())
}

/// Trích xuất định danh vi phạm trên dòng, trả về (tên, loại vi phạm)
fn extract(line: &str, config: &Config) -> Option<(String, &'static str)> {
    let trimmed = text::trim(line);
    // Bỏ qua comment
    if trimmed.starts_with("//") { return None; }
    for (re, kind) in PATTERNS.iter() {
        if let Some(cap) = re.captures(trimmed) {
            let name = cap.get(cap.len()-1).unwrap().as_str().to_string();
            // Whitelist luôn bỏ qua (ưu tiên tuyệt đối)
            if let Some(white) = &config.whitelist {
                if white.iter().any(|w| w == &name) {
                    return None;
                }
            }
            // Blacklist luôn báo lỗi
            if let Some(black) = &config.blacklist {
                if black.iter().any(|b| b == &name) {
                    return Some((name, "Blacklist"));
                }
            }
            // Chỉ báo lỗi PascalCase nếu nhiều hub (>=2)
            if *kind == "PascalCase" {
                if text::hub(&name) > 1 {
                    // Vi phạm: PascalCase nhiều hub
                    return Some((name, *kind));
                } else {
                    // Hợp lệ: PascalCase một hub
                    return None;
                }
            }
            // Các pattern khác giữ nguyên
            return Some((name, *kind));
        }
    }
    None
} 