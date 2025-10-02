pub use chrono::{ DateTime, Local };
use libc;
use std::ffi::CString;
pub use std::fs::{ self };
pub use std::io;
pub use std::os::unix::fs::{ FileTypeExt, MetadataExt, PermissionsExt };
use std::path::{ Path, PathBuf };
pub use users::{ get_group_by_gid, get_user_by_uid };

#[derive(Debug)]
struct Options {
    long: bool,
    all: bool,
    classify: bool,
    paths: Vec<String>,
}

fn parse_args(args: &[String]) -> Options {
    let mut opts = Options {
        long: false,
        all: false,
        classify: false,
        paths: vec![],
    };
    for arg in args {
        if arg.starts_with('-') {
            for ch in arg.chars().skip(1) {
                match ch {
                    'l' => {
                        opts.long = true;
                    }
                    'a' => {
                        opts.all = true;
                    }
                    'F' => {
                        opts.classify = true;
                    }
                    _ => eprintln!("ls: unknown flag -{}", ch),
                }
            }
        } else {
            opts.paths.push(arg.clone());
        }
    }
    if opts.paths.is_empty() {
        opts.paths.push(".".to_string());
    }
    opts
}

pub fn execute(args: &[String]) {
    let opts = parse_args(args);
    let multiple = opts.paths.len() > 1;

    for (i, path) in opts.paths.iter().enumerate() {
        let pathbuf = PathBuf::from(path);
        let meta = match fs::symlink_metadata(&pathbuf) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("ls: cannot access '{}': {}", path, e);
                continue;
            }
        };

        if meta.is_dir() {
            if multiple {
                println!("{}:", path);
            }
            let entries = match read_entries(&pathbuf, opts.all) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("ls: cannot read directory '{}': {}", path, e);
                    continue;
                }
            };

            let mut entries: Vec<PathBuf> = entries;
            entries.sort_by(|a, b| {
                let a_tail = Path::new(a)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let b_tail = Path::new(b)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                match (a_tail, b_tail) {
                    (".", ".") | ("..", "..") => std::cmp::Ordering::Equal,
                    (".", _) => std::cmp::Ordering::Less,
                    (_, ".") => std::cmp::Ordering::Greater,
                    ("..", _) => std::cmp::Ordering::Less,
                    (_, "..") => std::cmp::Ordering::Greater,
                    _ => a.cmp(b),
                }
            });

            if opts.long {
                let total_blocks: u64 = entries
                    .iter()
                    .map(|entry| {
                        fs::symlink_metadata(entry.as_path())
                            .map(|m| m.blocks())
                            .unwrap_or(0)
                    })
                    .sum();
                println!("total {}", total_blocks / 2);

                if let Err(e) = print_long(&entries, &opts) {
                    eprintln!("ls: error printing long format: {}", e);
                }
            } else {
                for entry in &entries {
                    let mut name = entry
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    let entry_str = entry.to_string_lossy();
                    if entry_str.ends_with("/.") {
                        name = ".".to_string();
                    } else if entry_str.ends_with("/..") {
                        name = "..".to_string();
                    }
                    if opts.classify {
                        name = format!("{}{}", name, classify_suffix(entry));
                    }
                    println!("{}", name);
                }
            }
        } else {
            if opts.long {
                if let Err(e) = print_long(&[pathbuf.clone()], &opts) {
                    eprintln!("ls: error printing long format: {}", e);
                }
            } else {
                let mut name = pathbuf
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let path_str = pathbuf.to_string_lossy();
                if path_str.ends_with("/.") || path_str == "." {
                    name = ".".to_string();
                } else if path_str.ends_with("/..") || path_str == ".." {
                    name = "..".to_string();
                }
                if opts.classify {
                    name = format!("{}{}", name, classify_suffix(&pathbuf));
                }
                println!("{}", name);
            }
        }

        if i != opts.paths.len() - 1 {
            println!();
        }
    }
}

fn read_entries(dir: &Path, show_all: bool) -> io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();

    if show_all {
        entries.push(dir.join("."));
        entries.push(dir.join(".."));
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        if !show_all && name.to_string_lossy().starts_with('.') {
            continue;
        }
        entries.push(entry.path());
    }
    Ok(entries)
}

fn print_long(entries: &[PathBuf], opts: &Options) -> io::Result<()> {
    let mut widths = Widths::default();
    let mut info = Vec::new();
    for path in entries {
        let meta = fs::symlink_metadata(&path)?;
        let nlink = meta.nlink();
        let uid = meta.uid();
        let gid = meta.gid();
        let uname = get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().to_string())
            .unwrap_or(uid.to_string());
        let gname = get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().to_string())
            .unwrap_or(gid.to_string());

        let is_device = meta.file_type().is_block_device() || meta.file_type().is_char_device();
        let size_str = if is_device {
            let rdev = meta.rdev();
            let major = (rdev >> 8) & 0xfff;
            let minor = (rdev & 0xff) | ((rdev >> 12) & 0xfff00);
            format!("{}, {}", major, minor)
        } else {
            meta.size().to_string()
        };

        widths.links = widths.links.max(nlink.to_string().len());
        widths.uname = widths.uname.max(uname.len());
        widths.gname = widths.gname.max(gname.len());
        widths.size = widths.size.max(size_str.len());

        info.push((path.clone(), meta, uname, gname, size_str));
    }

    for (path, meta, uname, gname, size_str) in info {
        let ftype = meta.file_type();
        let perm = meta.permissions().mode();
        let datetime_local: DateTime<Local> = DateTime::<Local>::from(meta.modified()?);
        let datetime = datetime_local.format("%b %e %H:%M");
        let mut name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let path_str = path.to_string_lossy();
        if path_str.ends_with("/.") {
            name = ".".to_string();
        } else if path_str.ends_with("/..") {
            name = "..".to_string();
        }

        let mut symlink = if ftype.is_symlink() {
            match fs::read_link(&path) {
                Ok(target) => format!(" -> {}", target.to_string_lossy()),
                Err(_) => "".to_string(),
            }
        } else {
            "".to_string()
        };
        if opts.classify {
            symlink = format!("{}{}", symlink, classify_suffix(&path));
        }
        let mut mode = format_mode(perm, &ftype);
        let plus_perm = has_extended_attributes(&path.to_string_lossy());
        if plus_perm {
            mode.push('+');
        } else {
            mode.push(' ');
        }
        print!(
            "{} {:>width_links$} {:<width_uname$} {:<width_gname$} {:>width_size$} {} {}{}\n",
            mode,
            meta.nlink(),
            uname,
            gname,
            size_str,
            datetime,
            name,
            symlink,
            width_links = widths.links,
            width_uname = widths.uname,
            width_gname = widths.gname,
            width_size = widths.size
        );
    }
    Ok(())
}

fn has_extended_attributes(path: &str) -> bool {
    let c_path = CString::new(path);
    match c_path {
        Ok(c_path) => unsafe {
            let size = libc::listxattr(c_path.as_ptr(), std::ptr::null_mut(), 0);
            return size > 0;
        }
        Err(_) => {
            eprintln!("ls: invalid path: {}", path);
            return false;
        }
    }
}

fn classify_suffix(path: &Path) -> &'static str {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            let ftype = meta.file_type();
            if ftype.is_symlink() {
                "@"
            } else if ftype.is_dir() {
                "/"
            } else if ftype.is_fifo() {
                "|"
            } else if ftype.is_socket() {
                "="
            } else if ftype.is_file() {
                if (meta.permissions().mode() & 0o111) != 0 { "*" } else { "" }
            } else {
                ""
            }
        }
        Err(_) => "",
    }
}

#[derive(Default)]
struct Widths {
    links: usize,
    uname: usize,
    gname: usize,
    size: usize,
}

fn format_mode(mode: u32, ftype: &fs::FileType) -> String {
    let file_type = if ftype.is_dir() {
        'd'
    } else if ftype.is_symlink() {
        'l'
    } else if ftype.is_fifo() {
        'p'
    } else if ftype.is_socket() {
        's'
    } else if ftype.is_char_device() {
        'c'
    } else if ftype.is_block_device() {
        'b'
    } else {
        '-'
    };

    let perms = [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ];

    let mut result = String::with_capacity(10);
    result.push(file_type);
    for (bit, ch) in perms.iter() {
        result.push(if (mode & bit) != 0 { *ch } else { '-' });
    }
    result
}
