use std::time::{Duration, Instant};

pub struct Stat {
    pub time: Duration,
    pub mem: u64,
    pub peak: u64,
    pub slow: Vec<String>,
    pub sum: u64,
    pub max: u64,
    pub min: u64,
    pub mean: f64,
    pub start: Instant,
}

impl Stat {
    pub fn new() -> Self {
        Self {
            time: Duration::ZERO,
            mem: 0,
            peak: 0,
            slow: vec![],
            sum: 0,
            max: 0,
            min: u64::MAX,
            mean: 0.0,
            start: Instant::now(),
        }
    }
    pub fn log(&self) {
        println!("[STAT] time={:?} mem={} peak={} sum={} mean={:.2} max={} min={}", self.time, self.mem, self.peak, self.sum, self.mean, self.max, self.min);
        if !self.slow.is_empty() {
            println!("[SLOW] {}", self.slow.join(", "));
        }
    }
    pub fn reset(&mut self) {
        self.time = Duration::ZERO;
        self.mem = 0;
        self.peak = 0;
        self.slow.clear();
        self.sum = 0;
        self.max = 0;
        self.min = u64::MAX;
        self.mean = 0.0;
        self.start = Instant::now();
    }
    pub fn stop(&mut self) {
        self.time = self.start.elapsed();
    }
    pub fn mem(&mut self, mem: u64) {
        self.mem += mem;
        if mem > self.peak {
            self.peak = mem;
        }
    }
    pub fn slow(&mut self, file: &str) {
        self.slow.push(file.to_string());
    }
    pub fn val(&mut self, val: u64) {
        self.sum += val;
        if val > self.max {
            self.max = val;
        }
        if val < self.min {
            self.min = val;
        }
    }
    pub fn mean(&mut self, n: u64) {
        if n > 0 {
            self.mean = self.sum as f64 / n as f64;
        }
    }
}

impl Default for Stat {
    fn default() -> Self {
        Self::new()
    }
} 