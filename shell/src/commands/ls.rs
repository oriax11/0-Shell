use std::fs;
use std::os::unix::fs::MetadataExt;
use std::time::UNIX_EPOCH;
use chrono::{DateTime, Local};
use libc;
use std::path::{Path, PathBuf};
use std::os::unix::fs::FileTypeExt;

#[derive(PartialEq, Debug)]
pub enum FileType {
    Directory,
    File,
    Executable,
    Symlink(String),
    CharDevice,
    BlockDevice,
    NamedPipe,
    Socket,
    Other,
}

impl FileType {
    pub fn from_path(path: &Path) -> Self {
        let metadata = fs::symlink_metadata(path).unwrap();
        let file_type = metadata.file_type();

        if file_type.is_symlink() {
            let target = fs::read_link(path).unwrap_or_default();
            FileType::Symlink(target.to_string_lossy().to_string())
        } else if file_type.is_dir() {
            FileType::Directory
        } else if file_type.is_file() {
            let mode = metadata.mode();
            if (mode & 0o111) != 0 {
                FileType::Executable
            } else {
                FileType::File
            }
        } else if file_type.is_char_device() {
            FileType::CharDevice
        } else if file_type.is_block_device() {
            FileType::BlockDevice
        } else if file_type.is_fifo() {
            FileType::NamedPipe
        } else if file_type.is_socket() {
            FileType::Socket
        } else {
            FileType::Other
        }
    }
}

#[derive(Clone)]
enum Indicator {
    None,
    Slash,
    Star,
    Pipe,
    Eq,
    Other(char),
}

struct EntryInfo {
    name: String,
    metadata: fs::Metadata,
    file_type: FileType,
    indicator: Indicator,
}

fn file_indicator(file_type: &FileType) -> Indicator {
    match file_type {
        FileType::Directory => Indicator::Slash,
        FileType::Executable => Indicator::Star,
        FileType::NamedPipe => Indicator::Pipe,
        FileType::Socket => Indicator::Eq,
        _ => Indicator::None,
    }
}

fn indicator_char(indicator: &Indicator) -> Option<char> {
    match indicator {
        Indicator::Slash => Some('/'),
        Indicator::Star => Some('*'),
        Indicator::Pipe => Some('|'),
        Indicator::Eq => Some('='),
        Indicator::Other(c) => Some(*c),
        _ => None,
    }
}

fn permissions_string(metadata: &fs::Metadata, file_type: &FileType) -> String {
    let mut perms = String::new();
    let mode = metadata.mode();

    perms.push(match file_type {
        FileType::Directory => 'd',
        FileType::Symlink(_) => 'l',
        FileType::CharDevice => 'c',
        FileType::BlockDevice => 'b',
        FileType::NamedPipe => 'p',
        FileType::Socket => 's',
        _ => '-',
    });

    perms.push(if (mode & 0o400) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o200) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o100) != 0 { 'x' } else { '-' }); // User execute
    perms.push(if (mode & 0o040) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o020) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o010) != 0 { 'x' } else { '-' }); // Group execute
    perms.push(if (mode & 0o004) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o002) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o001) != 0 { 'x' } else { '-' }); // Other execute

    // Handle SUID, SGID, and sticky bits
    let suid = (mode & 0o4000) != 0;
    let sgid = (mode & 0o2000) != 0;
    let sticky = (mode & 0o1000) != 0;

    // SUID bit for user execute
    if suid {
        if perms.chars().nth(3) == Some('x') {
            perms.replace_range(3..4, "s");
        } else {
            perms.replace_range(3..4, "S");
        }
    }

    // SGID bit for group execute
    if sgid {
        if perms.chars().nth(6) == Some('x') {
            perms.replace_range(6..7, "s");
        } else {
            perms.replace_range(6..7, "S");
        }
    }

    // Sticky bit for other execute
    if sticky {
        if perms.chars().nth(9) == Some('x') {
            perms.replace_range(9..10, "t");
        } else {
            perms.replace_range(9..10, "T");
        }
    }

    perms
}

fn print_long_listing(info: &EntryInfo, max_nlink_len: usize, max_user_len: usize, max_group_len: usize, max_size_or_dev_len: usize) {
    let perms = permissions_string(&info.metadata, &info.file_type);
    let nlink = info.metadata.nlink();

    let user = unsafe {
        let pwuid = libc::getpwuid(info.metadata.uid());
        if pwuid.is_null() {
            info.metadata.uid().to_string() // Fallback to UID if name not found
        } else {
            std::ffi::CStr::from_ptr((*pwuid).pw_name)
                .to_string_lossy()
                .into_owned()
        }
    };

    let group = unsafe {
        let grgid = libc::getgrgid(info.metadata.gid());
        if grgid.is_null() {
            info.metadata.gid().to_string() // Fallback to GID if name not found
        } else {
            std::ffi::CStr::from_ptr((*grgid).gr_name)
                .to_string_lossy()
                .into_owned()
        }
    };

    let mtime = info.metadata.modified().unwrap_or(UNIX_EPOCH);
    let datetime: DateTime<Local> = mtime.into();
    let date = datetime.format("%b %e %H:%M");

    let size_or_rdev_string = if info.file_type == FileType::CharDevice || info.file_type == FileType::BlockDevice {
        let rdev = info.metadata.rdev();
        let major = (rdev >> 8) & 0xFF; // Common way to get major number
        let minor = rdev & 0xFF;        // Common way to get minor number
        format!("{}, {}", major, minor)
    } else {
        info.metadata.len().to_string()
    };

    let mut final_name = info.name.clone();
    // Only append indicator for non-symlinks, as symlinks get " -> target"
    if let Some(c) = indicator_char(&info.indicator) {
        if !matches!(info.file_type, FileType::Symlink(_)) {
            final_name.push(c);
        }
    }

    let formatted_line = format!(
        "{} {:>width_nlink$} {:<width_user$} {:<width_group$} {:>width_size_or_dev$} {} {}",
        perms,
        nlink,
        user,
        group,
        size_or_rdev_string,
        date,
        final_name,
        width_nlink = max_nlink_len,
        width_user = max_user_len,
        width_group = max_group_len,
        width_size_or_dev = max_size_or_dev_len,
    );

    if let FileType::Symlink(ref target) = info.file_type {
        println!("{} -> {}", formatted_line, target);
    } else {
        println!("{}", formatted_line);
    }
}

pub fn execute(args: &[&str]) {
    let (flags, mut paths) = parse_flags(args);

    if paths.is_empty() {
        paths.push(".");
    }

    for path_str in &paths {
        if paths.len() > 1 {
            println!("{}:", path_str);
        }

        let path = Path::new(path_str);
        
        let mut entries_info = Vec::new();

        // --- Handle '.' and '..' first if '-a' is present ---
        if flags.contains(&'a') {
            // Handle '.'
            if let Ok(metadata) = fs::symlink_metadata(path) {
                let file_type = FileType::from_path(path);
                let indicator = file_indicator(&file_type);
                entries_info.push(EntryInfo {
                    name: ".".to_string(),
                    metadata,
                    file_type,
                    indicator,
                });
            }

            // Handle '..'
            if let Some(parent_path) = path.parent() {
                 if let Ok(metadata) = fs::symlink_metadata(parent_path) {
                    let file_type = FileType::from_path(parent_path);
                    let indicator = file_indicator(&file_type);
                    entries_info.push(EntryInfo {
                        name: "..".to_string(),
                        metadata,
                        file_type,
                        indicator,
                    });
                }
            } else if path.as_os_str() == "/" { // For root directory, '..' is '/' itself
                if let Ok(metadata) = fs::symlink_metadata(path) {
                    let file_type = FileType::from_path(path);
                    let indicator = file_indicator(&file_type);
                    entries_info.push(EntryInfo {
                        name: "..".to_string(),
                        metadata,
                        file_type,
                        indicator,
                    });
                }
            }
        }
        // --- End handling '.' and '..' ---


        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => {
                eprintln!("ls: cannot access '{}': No such file or directory", path.display());
                continue;
            }
        };

        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            // Skip '.' and '..' here, as we already explicitly added them if '-a'
            if file_name == "." || file_name == ".." {
                continue;
            }

            // Original hidden file logic
            if !flags.contains(&'a') && file_name.starts_with('.') {
                continue;
            }

            let entry_path = entry.path();
            if let Ok(metadata) = fs::symlink_metadata(&entry_path) {
                let file_type = FileType::from_path(&entry_path);
                let indicator = file_indicator(&file_type);

                entries_info.push(EntryInfo {
                    name: file_name,
                    metadata,
                    file_type,
                    indicator,
                });
            }
        }

        // Custom sorting: '.' then '..', then alphabetical for the rest.
        entries_info.sort_by(|a, b| {
            match (a.name.as_str(), b.name.as_str()) {
                (".", "..") => std::cmp::Ordering::Less,
                ("..", ".") => std::cmp::Ordering::Greater,
                (".", _) => std::cmp::Ordering::Less,
                (_, ".") => std::cmp::Ordering::Greater,
                ("..", _) => std::cmp::Ordering::Less,
                (_, "..") => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        if flags.contains(&'l') {
            let total_blocks: u64 = entries_info.iter().map(|info| info.metadata.blocks()).sum();
            println!("total {}", total_blocks / 2); // ls typically uses 1KB blocks for 'total'

            // Calculate max widths for alignment
            let mut max_nlink_len = 0;
            let mut max_user_len = 0;
            let mut max_group_len = 0;
            let mut max_size_or_dev_len = 0;

            for info in &entries_info {
                max_nlink_len = max_nlink_len.max(info.metadata.nlink().to_string().len());

                let user = unsafe {
                    let pwuid = libc::getpwuid(info.metadata.uid());
                    if pwuid.is_null() { info.metadata.uid().to_string() } else { std::ffi::CStr::from_ptr((*pwuid).pw_name).to_string_lossy().into_owned() }
                };
                max_user_len = max_user_len.max(user.len());

                let group = unsafe {
                    let grgid = libc::getgrgid(info.metadata.gid());
                    if grgid.is_null() { info.metadata.gid().to_string() } else { std::ffi::CStr::from_ptr((*grgid).gr_name).to_string_lossy().into_owned() }
                };
                max_group_len = max_group_len.max(group.len());

                let size_or_rdev_string_len = if info.file_type == FileType::CharDevice || info.file_type == FileType::BlockDevice {
                    let rdev = info.metadata.rdev();
                    let major = (rdev >> 8) & 0xFF;
                    let minor = rdev & 0xFF;
                    format!("{}, {}", major, minor).len()
                } else {
                    info.metadata.len().to_string().len()
                };
                max_size_or_dev_len = max_size_or_dev_len.max(size_or_rdev_string_len);
            }

            // Ensure a minimum width for size/device numbers (common ls behavior)
            max_size_or_dev_len = max_size_or_dev_len.max(8); 

            for info in &entries_info {
                print_long_listing(info, max_nlink_len, max_user_len, max_group_len, max_size_or_dev_len);
            }
        } else {
            // Simplified display for non-long listing, still respecting indicators
            for info in entries_info {
                let mut display_name = info.name.clone();
                if let Some(c) = indicator_char(&info.indicator) {
                     if !matches!(info.file_type, FileType::Symlink(_)) {
                        display_name.push(c);
                    }
                }
                if let FileType::Symlink(ref target) = info.file_type {
                    display_name.push_str(" -> ");
                    display_name.push_str(target);
                }
                print!("{}  ", display_name);
            }
            println!();
        }
    }
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