use std::fs;
use std::path::Path;

pub fn execute(args: &[String]) {
    // Clean arguments: trim whitespace and remove empty strings
    let cleaned: Vec<&str> = args.iter().map(|s| s.as_str().trim()).filter(|s| !s.is_empty()).collect();
    if cleaned.len() < 2 {
        eprintln!("mv: missing operand");
        return;
    }
// All arguments except the last are source paths to move
    let sources = &cleaned[..cleaned.len() - 1];
    let dest = cleaned[cleaned.len() - 1];

    let dest_path = Path::new(dest);
// If there are multiple sources, the destination must be a directory
    if sources.len() > 1 && !dest_path.is_dir() {
        eprintln!("mv: target '{}' is not a directory", dest);
        return;
    }
// Process each source file/directory
    for src in sources {
        let src_path = Path::new(src);
        if !src_path.exists() {
            eprintln!("mv: cannot stat '{}': No such file or directory", src);
            continue;
        }
        // Determine the final destination path for this source
        let final_dest = if dest_path.is_dir() {
            if let Some(filename) = src_path.file_name() {
                dest_path.join(filename)
            } else {
                eprintln!("mv: failed to get filename from '{}'", src);
                continue;
            }
        } else {
            // Destination is a file path, so use it as-is
            dest_path.to_path_buf()
        };

        if let Err(e) = fs::rename(src_path, final_dest) {
            eprintln!("mv: error moving: {}", e);
        }
    }
}
