// use std::fs;
// use std::path::Path;
// pub fn execute(args: &[&str]) {
//     // if args.len() != 2 {
//     //     eprintln!("mv: missing operand");
//     // } else {
//     //     let src = args[0];
//     //     let dest = args[1];
//     //     if let Err(e) = fs::rename(src, dest) {
//     //         eprintln!("mv: {}", e);
//     //     }
//     // }

//     let cleaned : Vec<&str>= args.iter().map(|w| w.trim()).filter(|w|!w.is_empty()).collect();

//     if args.len() < 2 {
//         eprintln!("mv: missing operand");
//         return;
//     }

//     let src = cleaned[0];
//     let dist = cleaned[1];

//     let path_src =  Path::new(src);
//     let path_dest = Path::new(dist);

//     if !path_src.exists()  {
//          eprintln!("mv: cannot stat '{}': No such file or directory", src);
//         return;
//     }

//     let final_des = if path_dest.is_dir() {
//         if let Some(filename) = path_src.file_name() {
//             path_dest.join(filename)
//         } else {
//             eprintln!("mv: failed to get filename from '{}'", src);
//             return;
//         }
//     } else {
//         path_dest.to_path_buf()
//     };


//     match fs::rename(path_src, final_des) {
//         Ok(_) => {},
//         Err(e) => eprintln!("mv: error moving: {}", e)
//     }
// }
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub fn execute(args: &[&str]) {
    let cleaned: Vec<&str> = args.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

    if cleaned.is_empty() {
        eprintln!("mv: missing operand");
        return;
    } else if cleaned.len() == 1 {
        eprintln!("mv: missing destination operand after '{}'", cleaned[0]);
        return;
    } else if cleaned.len() > 2 {
        eprintln!("mv: extra operand '{}'", cleaned[2]);
        return;
    }

    let src = PathBuf::from(cleaned[0]);
    let dst = PathBuf::from(cleaned[1]);

    if !src.exists() {
        eprintln!("mv: cannot stat '{}': No such file or directory", src.display());
        return;
    }

    let src_abs = safe_canonicalize(&src);
    let dst_abs = safe_canonicalize(&dst);

    if dst_abs.starts_with(&src_abs) && src_abs != dst_abs {
        eprintln!(
            "mv: cannot move '{}' into a subdirectory of itself, '{}'",
            src.display(),
            dst.display()
        );
        return;
    }

    if src_abs == dst_abs {
        eprintln!(
            "mv: '{}' and '{}' are the same file",
            src.display(),
            dst.display()
        );
        return;
    }

    let final_dst = if dst.is_dir() {
        match src.file_name() {
            Some(name) => dst.join(name),
            None => {
                eprintln!("mv: failed to get source file name");
                return;
            }
        }
    } else {
        let dst_str = dst.to_string_lossy();
        if dst_str.ends_with('/') && !dst.exists() {
            eprintln!("mv: target '{}' is not a directory", dst.display());
            return;
        }
        dst
    };

    if src.is_dir() && final_dst.exists() && final_dst.is_file() {
        eprintln!(
            "mv: cannot overwrite non-directory '{}' with directory '{}'",
            final_dst.display(),
            src.display()
        );
        return;
    }


    if src.is_file() && final_dst.exists() && final_dst.is_dir() {
        let dest_in_dir = final_dst.join(src.file_name().unwrap());
        if let Err(e) = fs::rename(&src, dest_in_dir) {
            eprintln!("mv: error moving '{}': {}", src.display(), e);
        }
        return;
    }


    if let Err(e) = fs::rename(&src, &final_dst) {
        if let Some(18) = e.raw_os_error() {
            if let Err(copy_err) = copy_recursively(&src, &final_dst) {
                eprintln!("mv: cannot move '{}': {}", src.display(), copy_err);
                return;
            }
            if src.is_dir() {
                if let Err(remove_err) = fs::remove_dir_all(&src) {
                    eprintln!("mv: failed to remove '{}': {}", src.display(), remove_err);
                }
            } else if let Err(remove_err) = fs::remove_file(&src) {
                eprintln!("mv: failed to remove '{}': {}", src.display(), remove_err);
            }
        } else {
            eprintln!("mv: cannot move '{}': {}", src.display(), e);
        }
    }
}

fn safe_canonicalize(p: &Path) -> PathBuf {
    fs::canonicalize(p).unwrap_or_else(|_e: std::io::Error| p.to_path_buf())
}


fn copy_recursively(src: &Path, dst: &Path) -> io::Result<()> {
    if src.is_file() {
        copy_file(src, dst)
    } else {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if file_type.is_dir() {
                copy_recursively(&src_path, &dst_path)?;
            } else {
                copy_file(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }
}

fn copy_file(src: &Path, dst: &Path) -> io::Result<()> {
    let mut src_file = fs::File::open(src)?;
    let mut dst_file = fs::File::create(dst)?;
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = src_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        dst_file.write_all(&buffer[..bytes_read])?;
    }
    Ok(())
}