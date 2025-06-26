pub fn split(s: &str, pat: char) -> Vec<&str> {
    s.split(pat).collect()
}

pub fn join(v: &[&str], sep: &str) -> String {
    v.join(sep)
}

pub fn trim(s: &str) -> &str {
    s.trim()
}

pub fn case(s: &str) -> String {
    s.to_ascii_lowercase()
}

pub fn find(s: &str, pat: &str) -> Option<usize> {
    s.find(pat)
}

pub fn hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

pub fn count(s: &str, c: char) -> usize {
    s.chars().filter(|&x| x == c).count()
}

pub fn len(s: &str) -> usize {
    s.chars().count()
}

pub fn min(a: usize, b: usize) -> usize {
    std::cmp::min(a, b)
}

pub fn max(a: usize, b: usize) -> usize {
    std::cmp::max(a, b)
}

pub fn cmp(a: &str, b: &str) -> std::cmp::Ordering {
    a.cmp(b)
}

pub fn dup(s: &str, n: usize) -> String {
    s.repeat(n)
}

/// Đếm số hub (ký tự viết hoa) trong định danh PascalCase
/// Theo quy tắc: PascalCase nhiều hub (>=2) là vi phạm, một hub thì hợp lệ
/// Ví dụ: UserProfile (2 hub, vi phạm), User (1 hub, hợp lệ)
pub fn hub(name: &str) -> usize {
    name.chars().filter(|c| c.is_uppercase()).count()
} 