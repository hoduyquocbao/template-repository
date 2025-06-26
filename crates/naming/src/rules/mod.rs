// ========================
// CẢNH BÁO VÀ GIỚI HẠN TOOL NAMING
//
// 1. Không kiểm tra định danh sinh ra bởi macro (macro_rules!, derive, procedural macro).
// 2. Không kiểm tra định danh trong doc comment, string literal.
// 3. Không kiểm tra định danh trong mod/use.
// 4. Không kiểm tra định danh trong tuple struct field, unnamed field.
// 5. Hỗ trợ whitelist/blacklist qua file naming.toml (ở cùng thư mục file kiểm tra).
//    - whitelist = ["Id", "API", ...]  // các định danh hợp lệ, luôn bỏ qua
//    - blacklist = ["DataManager", ...] // các định danh luôn báo lỗi
// 6. Không kiểm tra định danh trong code sinh động (build.rs, codegen).
// 7. Không kiểm tra định danh trong macro expansion (chỉ code gốc).
// 8. Nếu gặp macro, sẽ cảnh báo nhưng không kiểm tra bên trong.
// 9. Nếu muốn enforce toàn bộ codebase, cần tích hợp với rustc hoặc build script.
// 10. Số dòng báo lỗi chỉ chính xác khi kiểm tra từng dòng, không dựa vào AST.
// ========================

use std::fs;
use std::path::Path;
use serde::Deserialize;
pub mod line;
pub mod ast;
pub mod metric;
use metric::{Metric, Detail};
pub mod report;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub whitelist: Option<Vec<String>>,
    pub blacklist: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>,
    pub pascal: Option<bool>,
    pub snake: Option<bool>,
    pub camel: Option<bool>,
    pub length: Option<bool>,
    pub min: Option<usize>,
    pub max: Option<usize>,
}

fn ignore(file: &str, config: &Config) -> bool {
    if let Some(ignore) = &config.ignore {
        for pat in ignore {
            if file.contains(pat) {
                return true;
            }
        }
    }
    false
}

pub fn metric(file: &str) -> (Metric, Vec<Detail>) {
    use metric::measure;
    let config = read(file);
    if ignore(file, &config) {
        let mut m = Metric::new(file);
        m.error = Some("Ignored by config".to_string());
        return (m, vec![]);
    }
    
    measure(
        file,
        || {
            let mut found = false;
            let mut lines = Vec::new();
            line::scan(file, &config, &mut found, &mut lines)?;
            Ok(lines)
        },
        || {
            ast::scan(file, &config)
        },
    )
}

pub fn read(file: &str) -> Config {
    let path = Path::new(file).parent().unwrap_or_else(|| Path::new("."));
    let path = path.join("naming.toml");
    if let Ok(cfg_str) = fs::read_to_string(&path) {
        toml::from_str(&cfg_str).unwrap_or_default()
    } else {
        Config::default()
    }
}
