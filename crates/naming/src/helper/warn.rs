use std::collections::HashSet;

pub struct Warn {
    set: HashSet<String>,
}

impl Warn {
    pub fn new() -> Self {
        Self { set: HashSet::new() }
    }
    pub fn add(&mut self, msg: &str) {
        self.set.insert(msg.to_string());
    }
    pub fn show(&self) -> String {
        self.set.iter().cloned().collect::<Vec<_>>().join("|")
    }
    pub fn log(&self) {
        for w in &self.set {
            println!("[WARN] {w}");
        }
    }
    pub fn flag(&self, pat: &str) -> bool {
        self.set.iter().any(|w| w.contains(pat))
    }
    pub fn msg(&self) -> Vec<String> {
        self.set.iter().cloned().collect()
    }
    pub fn typ(&self) -> Vec<&str> {
        self.set.iter().map(|w| w.split(':').next().unwrap_or("")).collect()
    }
}

impl Default for Warn {
    fn default() -> Self {
        Self::new()
    }
} 