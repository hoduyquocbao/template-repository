use std::collections::{HashMap, HashSet};
use std::fs;

pub struct Conf {
    pub map: HashMap<String, String>,
    pub enable: HashSet<String>,
    pub disable: HashSet<String>,
    pub ignore: HashSet<String>,
}

impl Conf {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            enable: HashSet::new(),
            disable: HashSet::new(),
            ignore: HashSet::new(),
        }
    }
    pub fn load(&mut self, path: &str) {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                if let Some(stripped) = line.strip_prefix("enable=") {
                    self.enable.insert(stripped.to_string());
                } else if let Some(stripped) = line.strip_prefix("disable=") {
                    self.disable.insert(stripped.to_string());
                } else if let Some(stripped) = line.strip_prefix("ignore=") {
                    self.ignore.insert(stripped.to_string());
                } else if let Some((k, v)) = line.split_once('=') {
                    self.map.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
    }
    pub fn save(&self, path: &str) {
        let mut s = String::new();
        for k in &self.enable {
            s.push_str(&format!("enable={}\n", k));
        }
        for k in &self.disable {
            s.push_str(&format!("disable={}\n", k));
        }
        for k in &self.ignore {
            s.push_str(&format!("ignore={}\n", k));
        }
        for (k, v) in &self.map {
            s.push_str(&format!("{}={}\n", k, v));
        }
        let _ = fs::write(path, s);
    }
    pub fn get(&self, k: &str) -> Option<&String> {
        self.map.get(k)
    }
    pub fn set(&mut self, k: &str, v: &str) {
        self.map.insert(k.to_string(), v.to_string());
    }
    pub fn rule(&self) -> Vec<&String> {
        self.enable.iter().collect()
    }
    pub fn list(&self) -> Vec<&String> {
        self.map.keys().collect()
    }
    pub fn enable(&self, k: &str) -> bool {
        self.enable.contains(k)
    }
    pub fn disable(&self, k: &str) -> bool {
        self.disable.contains(k)
    }
    pub fn ignore(&self, k: &str) -> bool {
        self.ignore.contains(k)
    }
}

impl Default for Conf {
    fn default() -> Self {
        Self::new()
    }
} 