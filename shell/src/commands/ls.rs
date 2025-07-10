use std::fs;
use std::os::unix::fs::MetadataExt;
use std::time::UNIX_EPOCH;
use chrono::{DateTime, Local};
use libc;
use std::path::Path;

// Helper struct to hold file information
struct EntryInfo {
    name: String,
    display_name: String,
    metadata: fs::Metadata,
}

fn parse_flags<'a>(args: &'a [&str]) -> (Vec<char>, Vec<&'a str>) {
    let mut flags = Vec::new();
    let mut paths = Vec::new();

    for arg in args.iter().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        if arg.starts_with('-') {
            for c in arg.chars().skip(1) {
                if !flags.contains(&c) {
                    flags.push(c);
                }
            }
        } else {
            paths.push(arg);
        }
    }

    (flags, paths)
}

pub fn execute(rest: &[&str]) {
    let (flags, mut paths) = parse_flags(rest);

    // Check for invalid flags
    let valid_flags = ['a', 'l', 'F'];
    if let Some(invalid_flag) = flags.iter().find(|f| !valid_flags.contains(f)) {
        println!("Command '-{}' not found", invalid_flag);
        return;
    }

    if paths.is_empty() {
        paths.push(".");
    }

    for path_str in &paths {
        if paths.len() > 1 {
            println!("{}:", path_str);
        }

        let path = Path::new(path_str);
        let mut entries_info = Vec::new();

        // Handle . and .. for -a flag
        if flags.contains(&'a') {
            if let Ok(meta) = fs::symlink_metadata(path) {
                let mut display_name = ".".to_string();
                if flags.contains(&'F') {
                    display_name.push('/');
                }
                entries_info.push(EntryInfo {
                    name: ".".to_string(),
                    display_name,
                    metadata: meta,
                });
            }

            let parent_path = path.join("..");
            if let Ok(meta) = fs::symlink_metadata(&parent_path) {
                let mut display_name = "..".to_string();
                if flags.contains(&'F') {
                    display_name.push('/');
                }
                entries_info.push(EntryInfo {
                    name: "..".to_string(),
                    display_name,
                    metadata: meta,
                });
            }
        }

        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => {
                eprintln!("ls: cannot access '{}': No such file or directory", path.display());
                continue;
            }
        };

        let mut dir_entries: Vec<fs::DirEntry> = entries.filter_map(Result::ok).collect();
        dir_entries.sort_by_key(|e| e.file_name());

        for entry in dir_entries {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !flags.contains(&'a') && file_name.starts_with('.') {
                continue;
            }

            let path = entry.path();
            if let Ok(metadata) = fs::symlink_metadata(&path) {
                let mut display_name = file_name.clone();

                if flags.contains(&'F') {
                    let file_type = metadata.file_type();
                    if file_type.is_dir() {
                        display_name.push('/');
                    } else if file_type.is_symlink() {
                        display_name.push('@');
                    } else if (metadata.mode() & 0o111) != 0 {
                        display_name.push('*');
                    }
                }

                entries_info.push(EntryInfo {
                    name: file_name.clone(),
                    display_name,
                    metadata,
                });
            }
        }

        entries_info.sort_by(|a, b| a.name.cmp(&b.name));

        if flags.contains(&'l') {
            let total_blocks: u64 = entries_info
                .iter()
                .map(|info| info.metadata.blocks())
                .sum();
            println!("total {}", total_blocks / 2); // Convert 512B blocks to 1K

            for info in &entries_info {
                print_long_listing(&info.name, &info.metadata, flags.contains(&'F'));
            }
        } else {
            let display_names: Vec<String> =
                entries_info.iter().map(|info| info.display_name.clone()).collect();
            println!("{}", display_names.join("  "));
        }
    }
}

fn print_long_listing(name: &str, metadata: &fs::Metadata, show_indicator: bool) {
    let perms = permissions_string(metadata);
    let nlink = metadata.nlink();

    let user = unsafe {
        let pwuid = libc::getpwuid(metadata.uid());
        if pwuid.is_null() {
            "unknown".to_string()
        } else {
            std::ffi::CStr::from_ptr((*pwuid).pw_name)
                .to_string_lossy()
                .into_owned()
        }
    };

    let group = unsafe {
        let grgid = libc::getgrgid(metadata.gid());
        if grgid.is_null() {
            "unknown".to_string()
        } else {
            std::ffi::CStr::from_ptr((*grgid).gr_name)
                .to_string_lossy()
                .into_owned()
        }
    };

    let size = metadata.len();
    let mtime = metadata.modified().unwrap_or(UNIX_EPOCH);
    let datetime: DateTime<Local> = mtime.into();
    let date = datetime.format("%b %d %H:%M");

    let mut display_name = name.to_string();

    if metadata.file_type().is_symlink() {
        if let Ok(target) = fs::read_link(name) {
            display_name.push_str(" -> ");
            display_name.push_str(&target.to_string_lossy());
        }
    } else if show_indicator {
        let file_type = metadata.file_type();
        if file_type.is_dir() {
            display_name.push('/');
        } else if file_type.is_symlink() {
            display_name.push('@');
        }else if (metadata.mode() & 0o111) != 0 {
            display_name.push('*');
        }
    }

    println!(
        "{} {:>2} {:<8} {:<8} {:>6} {} {}",
        perms, nlink, user, group, size, date, display_name
    );
}

fn permissions_string(metadata: &fs::Metadata) -> String {
    let mut perms = String::new();
    let mode = metadata.mode();

    perms.push(
        if metadata.file_type().is_symlink() {
            'l'
        } else if metadata.is_dir() {
            'd'
        } else {
            '-'
        },
    );

    // Owner
    perms.push(if (mode & 0o400) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o200) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o100) != 0 { 'x' } else { '-' });

    // Group
    perms.push(if (mode & 0o040) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o020) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o010) != 0 { 'x' } else { '-' });

    // Others
    perms.push(if (mode & 0o004) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o002) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o001) != 0 { 'x' } else { '-' });

    perms
}
