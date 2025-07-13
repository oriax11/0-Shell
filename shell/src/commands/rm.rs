use std::fs;
use std::path::{Path, PathBuf};

pub fn execute(args: &[String]) {
    let mut recursive = false;
    let mut force = false;
    let mut paths = Vec::new();

    let cleaned: Vec<&str> = args
        .iter()
        .map(|s| s.as_str())
        .map(|w| w.trim())
        .filter(|w| !w.is_empty())
        .collect();

    for arg in cleaned.iter() {
        if *arg == "-r" || *arg == "-R" {
            recursive = true;
        } else if *arg == "-f" {
            force = true;
        } else {
            paths.push(arg);
        }
    }

    for path_str in paths {
        let path = Path::new(path_str);

        // Normalize path for safety checks
        let normalized = normalize_path(path);

        if is_dangerous_path(&normalized) {
            eprintln!(
                "rm: refusing to remove '.' or '..' directory: skipping '{}'",
                path_str
            );
            continue;
        }

        if normalized.to_string_lossy() == "/" {
            eprintln!("rm: it is dangerous to operate recursively on '/'");
            continue;
        }

        match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    // Delete the symlink itself, not the target
                    if let Err(e) = fs::remove_file(path) {
                        if !force {
                            eprintln!("rm: {}: {}", path_str, e);
                        }
                    }
                } else if metadata.is_dir() {
                    if recursive {
                        if let Err(e) = fs::remove_dir_all(path) {
                            if !force {
                                eprintln!("rm: {}: {}", path_str, e);
                            }
                        }
                    } else {
                        eprintln!("rm: cannot remove '{}': Is a directory", path_str);
                    }
                } else {
                    // regular file
                    if let Err(e) = fs::remove_file(path) {
                        if !force {
                            eprintln!("rm: {}: {}", path_str, e);
                        }
                    }
                }
            }
            Err(_) => {
                if !force {
                    eprintln!("rm: cannot remove '{}': No such file or directory", path_str);
                }
            }
        }
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut clean = PathBuf::new();
    for component in path.components() {
        clean.push(component);
    }
    clean
}

fn is_dangerous_path(path: &Path) -> bool {
    match path.to_string_lossy().as_ref() {
        "." | ".." => true,
        s if s.ends_with("/.") || s.ends_with("/..") => true,
        _ => false,
    }
}
