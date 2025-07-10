use std::fs;
use std::path::PathBuf;

pub fn execute(args: &[&str]) {
    let mut recursive = false;
    let mut paths = Vec::new();

    for arg in args {
        if *arg == "-r" {
            recursive = true;
        } else {
            paths.push(arg);
        }
    }

    for path in paths {
        let path_buf = PathBuf::from(path);
        if path_buf.is_dir() {
            if recursive {
                if let Err(e) = fs::remove_dir_all(&path_buf) {
                    eprintln!("rm: {}: {}", path, e);
                }
            } else {
                eprintln!("rm: cannot remove '{}': Is a directory", path);
            }
        } else {
            if let Err(e) = fs::remove_file(&path_buf) {
                eprintln!("rm: {}: {}", path, e);
            }
        }
    }
}
