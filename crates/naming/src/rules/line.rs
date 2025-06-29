use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::rules::Config;
use std::collections::HashSet;
use once_cell::sync::Lazy;
use crate::helper::text;

static PATTERNS: Lazy<Vec<(regex::Regex, &'static str)>> = Lazy::new(|| vec![
    // Alias patterns (ưu tiên cao nhất)
    (regex::Regex::new(r"^use\s+.*\s+as\s+([A-Z][A-Za-z0-9]*)").unwrap(), "AliasPascalCase"),
    (regex::Regex::new(r"^pub\s+use\s+.*\s+as\s+([A-Z][A-Za-z0-9]*)").unwrap(), "AliasPascalCase"),
    (regex::Regex::new(r"^use\s+.*\s+as\s+([a-z]+[A-Z][a-zA-Z0-9]*)").unwrap(), "AliasCamelCase"),
    (regex::Regex::new(r"^pub\s+use\s+.*\s+as\s+([a-z]+[A-Z][a-zA-Z0-9]*)").unwrap(), "AliasCamelCase"),
    (regex::Regex::new(r"^use\s+.*\s+as\s+([a-z0-9]+_[a-z0-9_]+)").unwrap(), "AliasSnakeCase"),
    (regex::Regex::new(r"^pub\s+use\s+.*\s+as\s+([a-z0-9]+_[a-z0-9_]+)").unwrap(), "AliasSnakeCase"),
    // Regular patterns
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
        let trimmed = text::trim(&line);
        // Xử lý group alias: use foo::{Bar as Baz, Qux as Quux};
        if trimmed.starts_with("use ") && trimmed.contains("{") && trimmed.contains("}") {
            if let Some(start) = trimmed.find('{') {
                if let Some(end) = trimmed.find('}') {
                    let group = &trimmed[start+1..end];
                    for part in group.split(',') {
                        let part = part.trim();
                        if let Some(as_pos) = part.find(" as ") {
                            let alias = part[as_pos+4..].trim();
                            // Kiểm tra alias như các pattern khác
                            if let Some((name, kind)) = check(alias) {
                                out.push((Some(i+1), name, kind));
                                *found = true;
                            }
                        }
                    }
                }
            }
        }
        if let Some((name, kind)) = extract(&line, config) {
            // Kiểm tra enable rule
            if (kind == "PascalCase" && config.pascal == Some(false))
                || (kind == "snake_case" && config.snake == Some(false))
                || (kind == "camelCase" && config.camel == Some(false))
                || (kind == "AliasPascalCase" && config.alias == Some(false))
                || (kind == "AliasCamelCase" && config.alias == Some(false))
                || (kind == "AliasSnakeCase" && config.alias == Some(false)) {
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
            // Chỉ báo lỗi AliasPascalCase nếu nhiều hub (>=2)
            if *kind == "AliasPascalCase" {
                if text::hub(&name) > 1 {
                    // Vi phạm: AliasPascalCase nhiều hub
                    return Some((name, *kind));
                } else {
                    // Hợp lệ: AliasPascalCase một hub
                    return None;
                }
            }
            // Báo lỗi AliasCamelCase (luôn vi phạm vì không phải một từ)
            if *kind == "AliasCamelCase" {
                return Some((name, *kind));
            }
            // Báo lỗi AliasSnakeCase (luôn vi phạm vì không phải một từ)
            if *kind == "AliasSnakeCase" {
                return Some((name, *kind));
            }
            // Các pattern khác giữ nguyên
            return Some((name, *kind));
        }
    }
    None
}

// Hàm kiểm tra alias trong group
fn check(alias: &str) -> Option<(String, &'static str)> {
    // Kiểm tra các pattern alias
    let name = alias.to_string();
    if name.contains('_') {
        return Some((name, "AliasSnakeCase"));
    }
    if name.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
        if name.chars().any(|c| c.is_uppercase()) {
            return Some((name, "AliasCamelCase"));
        }
    }
    if text::hub(&name) > 1 {
        return Some((name, "AliasPascalCase"));
    }
    None
} 