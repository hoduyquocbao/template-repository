use std::time::{Duration, Instant};
use sysinfo::System;
use crate::rules::ast::Violation as AstViolation;

#[derive(Debug, Default)]
pub struct Metric {
    pub file: String,
    pub line: Duration,
    pub ast: Duration,
    pub total: Duration,
    pub error: Option<String>,
    pub violations: usize,
    pub peak: u64,
}

impl Metric {
    pub fn new(file: &str) -> Self {
        Self {
            file: file.to_string(),
            ..Default::default()
        }
    }
}

pub type Detail = (String, Option<usize>, String, String);

pub fn measure<F1, F2>(
    file: &str,
    line: F1,
    ast: F2,
) -> (Metric, Vec<Detail>)
where
    F1: FnOnce() -> Result<Vec<(Option<usize>, String, &'static str)>, String>,
    F2: FnOnce() -> Result<Vec<AstViolation>, String>,
{
    let mut metric = Metric::new(file);
    let mut details: Vec<Detail> = Vec::new();
    
    let mut sys = System::new_all();
    sys.refresh_memory();
    let before = sys.used_memory();
    let start = Instant::now();
    
    let time = Instant::now();
    let lres = line();
    metric.line = time.elapsed();
    
    let now = Instant::now();
    let ares = ast();
    metric.ast = now.elapsed();
    
    metric.total = start.elapsed();
    sys.refresh_memory();
    let after = sys.used_memory();
    metric.peak = after.saturating_sub(before);

    match (lres, ares) {
        (Ok(line_violations), Ok(ast_violations)) => {
            metric.violations = line_violations.len() + ast_violations.len();
            for (l, n, k) in line_violations {
                details.push((file.to_string(), l, n, k.to_string()));
            }
            for v in ast_violations {
                details.push((file.to_string(), v.line, v.name, v.kind.to_string()));
            }
        }
        (Err(e), _) | (_, Err(e)) => {
            metric.error = Some(e);
        }
    }

    (metric, details)
} 