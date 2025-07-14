use std::fs;
use std::os::unix::fs::{ MetadataExt, FileTypeExt };
use std::time::UNIX_EPOCH;
use chrono::{ DateTime, Local };
use std::path::Path;
use std::ffi::CStr;
use libc::{ getpwuid, getgrgid };

fn parse_flags(args: &[String]) -> (Vec<char>, Vec<String>) {
    let mut flags = Vec::new();
    let mut paths = Vec::new();
    for arg in args {
        if arg.starts_with('-') {
            for c in arg.chars().skip(1) {
                flags.push(c);
            }
        } else {
            paths.push(arg.clone());
        }
    }
    (flags, paths)
}

pub fn execute(rest: &[String]) {
    let (flags, mut paths) = parse_flags(rest);
    if paths.is_empty() {
        paths.push(".".to_string());
    }

    for path in &paths {
        if paths.len() > 1 {
            println!("{}:", path);
        }

        let mut names = Vec::new();
        let entries = match fs::read_dir(&path) {
            Ok(e) => e,
            Err(_) => {
                eprintln!("ls: cannot access '{}': No such file or directory", path);
                continue;
            }
        };

        if flags.contains(&'a') && flags.contains(&'F') {
            names.push("./".to_string());
            names.push("../".to_string());
        } else if flags.contains(&'a') {
            names.push(".".to_string());
            names.push("..".to_string());
        }

        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if !flags.contains(&'a') && file_name.starts_with('.') {
                    continue;
                }

                let mut display_name = file_name.clone();
                let full_path = Path::new(&path).join(&file_name);

                if flags.contains(&'F') {
                    if let Ok(meta) = fs::symlink_metadata(&full_path) {
                        let ftype = meta.file_type();
                        if ftype.is_dir() {
                            display_name.push('/');
                        } else if ftype.is_symlink() {
                            display_name.push('@');
                        } else if (meta.mode() & 0o111) != 0 {
                            display_name.push('*');
                        }
                    }
                }

                names.push(display_name);
            }
        }

        names.sort();

        if flags.contains(&'l') {
            let mut total_blocks = 0;
            for name in &names {
                let full_path = Path::new(&path).join(name);
                if let Ok(metadata) = fs::symlink_metadata(&full_path) {
                    total_blocks += metadata.blocks();
                }
            }

            println!("total {}", total_blocks);

            for  name in &names {
                let mut tmp_name = name.clone();
                if flags.contains(&'F') {
                    if let Some(last) = name.chars().last() {
                        if last == '@' || last == '*' || last == '/' {
                            tmp_name.pop();
                        }
                    }
                }

                let full_path = Path::new(&path).join(tmp_name);
                let metadata = match fs::symlink_metadata(&full_path) {
                    Ok(m) => m,
                    Err(e) => {
                        println!("{:?} {}",e, name);
                        continue;
                    }
                };

                let file_type = metadata.file_type();

                let perms = if file_type.is_symlink() {
                    "l".to_string() + &permissions_string(&metadata)[1..]
                } else if file_type.is_dir() {
                    "d".to_string() + &permissions_string(&metadata)[1..]
                } else {
                    permissions_string(&metadata)
                };

                let nlink = metadata.nlink();
                let user = get_username(metadata.uid());
                let group = get_groupname(metadata.gid());

                let size_display = if file_type.is_char_device() || file_type.is_block_device() {
                    let rdev = metadata.rdev();
                    let major = ((rdev >> 8) & 0xfff) as u32;
                    let minor = ((rdev & 0xff) | ((rdev >> 12) & 0xfff00)) as u32;
                    format!("{}, {}", major, minor)
                } else {
                    format!("{}", metadata.len())
                };


                let mtime = metadata.modified().unwrap_or(UNIX_EPOCH);
                let datetime: DateTime<Local> = mtime.into();
                let date = datetime.format("%b %d %H:%M");

                let mut display_name = name.clone();
                if file_type.is_symlink() {
                    if let Ok(target) = fs::read_link(&full_path) {
                        display_name = format!("{} -> {}", name, target.to_string_lossy());
                    }
                }

                println!(
                    "{} {:>2} {:<8} {:<8} {:>8} {} {}",
                    perms,
                    nlink,
                    user,
                    group,
                    size_display,
                    date,
                    display_name
                );
            }
        } else {
            println!("{}", names.join("  "));
        }
    }
}

fn permissions_string(metadata: &fs::Metadata) -> String {
    let mut perms = String::new();
    let mode = metadata.mode();
    perms.push('-');
    perms.push(if (mode & 0o400) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o200) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o100) != 0 { 'x' } else { '-' });
    perms.push(if (mode & 0o040) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o020) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o010) != 0 { 'x' } else { '-' });
    perms.push(if (mode & 0o004) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o002) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o001) != 0 { 'x' } else { '-' });
    perms
}

fn get_username(uid: u32) -> String {
    unsafe {
        let pw = getpwuid(uid);
        if pw.is_null() {
            return "unknown".into();
        }
        let name_ptr = (*pw).pw_name;
        if name_ptr.is_null() {
            return "unknown".into();
        }
        CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
    }
}

fn get_groupname(gid: u32) -> String {
    unsafe {
        let gr = getgrgid(gid);
        if gr.is_null() {
            return "unknown".into();
        }
        let name_ptr = (*gr).gr_name;
        if name_ptr.is_null() {
            return "unknown".into();
        }
        CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
    }
}
