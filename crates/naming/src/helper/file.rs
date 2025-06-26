use std::fs::{self, File};
use std::io::{self};
use std::path::Path;

pub fn read(path: &str) -> io::Result<String> {
    fs::read_to_string(path)
}

pub fn write(path: &str, data: &str) -> io::Result<()> {
    fs::write(path, data)
}

pub fn open(path: &str) -> io::Result<File> {
    File::open(path)
}

pub fn close(_file: File) {}

pub fn lock(path: &str) -> io::Result<()> {
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o000))
    }
    #[cfg(not(unix))]
    { Ok(()) }
}

pub fn perm(path: &str, mode: u32) -> io::Result<()> {
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
    }
    #[cfg(not(unix))]
    { Ok(()) }
}

pub fn scan(path: &Path, v: &mut Vec<String>) -> io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                scan(&p, v)?;
            } else if p.is_file() {
                v.push(p.to_string_lossy().to_string());
            }
        }
    }
    Ok(())
}

pub fn ext(file: &str) -> Option<String> {
    Path::new(file).extension().map(|e| e.to_string_lossy().to_string())
}

pub fn dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

pub fn file(path: &str) -> bool {
    Path::new(path).is_file()
}

pub fn list(dir: &str) -> io::Result<Vec<String>> {
    let mut v = vec![];
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            v.push(path.to_string_lossy().to_string());
        }
    }
    Ok(v)
}

pub fn find(dir: &str, ext: &str) -> io::Result<Vec<String>> {
    let mut v = vec![];
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map(|e| e==ext).unwrap_or(false) {
            v.push(path.to_string_lossy().to_string());
        }
    }
    Ok(v)
}

pub fn del(path: &str) -> io::Result<()> {
    fs::remove_file(path)
} 