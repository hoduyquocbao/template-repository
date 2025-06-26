use std::fs::File;
use std::io::{Write, BufWriter};
use crate::rules::metric::Metric;
use serde::Serialize;

fn warning(metric: &Metric) -> String {
    let mut warns = Vec::new();
    if metric.total.as_millis() > 500 {
        warns.push("Slow");
    }
    if metric.peak > 10240 {
        warns.push("HighMem");
    }
    if let Some(e) = &metric.error {
        if e.contains("Permission denied") {
            warns.push("Denied");
        } else if e.contains("No such file") {
            warns.push("NotFound");
        } else {
            warns.push("IOError");
        }
    }
    warns.join("|")
}

#[derive(Serialize)]
struct Json<'a> {
    file: &'a str,
    line: u128,
    ast: u128,
    total: u128,
    peak: u64,
    violations: usize,
    error: &'a str,
    warning: String,
}

pub fn json(metrics: &[Metric], path: &str) -> std::io::Result<()> {
    let mut out = Vec::new();
    for m in metrics {
        out.push(Json {
            file: &m.file,
            line: m.line.as_millis(),
            ast: m.ast.as_millis(),
            total: m.total.as_millis(),
            peak: m.peak,
            violations: m.violations,
            error: m.error.as_deref().unwrap_or(""),
            warning: warning(m),
        });
    }
    let file = BufWriter::new(File::create(path)?);
    serde_json::to_writer_pretty(file, &out)?;
    Ok(())
}

pub fn csv(metrics: &[Metric], path: &str) -> std::io::Result<()> {
    let mut file = BufWriter::new(File::create(path)?);
    writeln!(file, "file,line,ast,total,peak,violations,error,warning")?;
    for m in metrics {
        writeln!(file, "{}",
            [
                &m.file,
                &m.line.as_millis().to_string(),
                &m.ast.as_millis().to_string(),
                &m.total.as_millis().to_string(),
                &m.peak.to_string(),
                &m.violations.to_string(),
                m.error.as_deref().unwrap_or(""),
                &warning(m)
            ].join(",")
        )?;
    }
    Ok(())
}

pub fn md(metrics: &[Metric], path: &str) -> std::io::Result<()> {
    let mut file = BufWriter::new(File::create(path)?);
    writeln!(file, "| File | Line (ms) | AST (ms) | Total (ms) | Peak (KB) | Violations | Error | Warning |")?;
    writeln!(file, "|------|-----------|----------|-----------|----------|------------|-------|---------|")?;
    for m in metrics {
        writeln!(file, "| {} | {} | {} | {} | {} | {} | {} | {} |",
            m.file,
            m.line.as_millis(),
            m.ast.as_millis(),
            m.total.as_millis(),
            m.peak,
            m.violations,
            m.error.as_deref().unwrap_or(""),
            warning(m)
        )?;
    }
    Ok(())
}

pub fn detail(details: &[(String, Option<usize>, String, String)], path: &str) -> std::io::Result<()> {
    let mut file = BufWriter::new(File::create(path)?);
    writeln!(file, "file,line,name,kind")?;
    for (f, l, n, k) in details {
        let line = l.map(|x| x.to_string()).unwrap_or_default();
        writeln!(file, "{},{},{},{}", f, line, n, k)?;
    }
    Ok(())
} 