use std::fs;
use std::path::Path;

pub fn execute(args: &[String]) {
    let cleaned: Vec<&str> = args.iter()
        .map(|w| w.trim())
        .filter(|w| !w.is_empty())
        .collect();

    // Check for unsupported flags
    for arg in &cleaned {
        if arg.starts_with('-') {
            eprintln!("cp: invalid option -- '{}'", arg.trim_start_matches('-'));
            eprintln!("cp does not support flags");
            return;
        }
    }

    if cleaned.len() < 2 {
        eprintln!("cp: missing operand");
        return;
    }

    // All arguments except the last are sources
    let sources = &cleaned[..cleaned.len() - 1];
    let dest = cleaned[cleaned.len() - 1];
    let dest_path = Path::new(dest);

    // If multiple sources, destination must be a directory
    if sources.len() > 1 && !dest_path.is_dir() {
        eprintln!("cp: target '{}' is not a directory", dest);
        return;
    }

    // Copy each source file
    for src in sources {
        let src_path = Path::new(src);

        if !src_path.exists() {
            eprintln!("cp: cannot stat '{}': No such file or directory", src);
            continue;
        }

        if src_path.is_dir() {
            eprintln!("cp: omitting directory '{}'", src);
            continue;
        }

        let final_dest = if dest_path.is_dir() {
            if let Some(filename) = src_path.file_name() {
                dest_path.join(filename)
            } else {
                eprintln!("cp: failed to get filename from '{}'", src);
                continue;
            }
        } else {
            dest_path.to_path_buf()
        };

        if let Err(e) = fs::copy(src_path, &final_dest) {
            eprintln!("cp: error copying '{}': {}", src, e);
        }
    }
}
