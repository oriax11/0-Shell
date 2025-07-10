use std::fs;
use std::os::unix::fs::MetadataExt;
use std::time::UNIX_EPOCH;
use chrono::{DateTime, Local};
// use users::{get_user_by_uid, get_group_by_gid}; // REMOVE THIS
use libc; // ADD THIS

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

pub fn handle_ls(rest: &[String]) {
    if rest.len() == 0 {
        let mut names = Vec::new();
        match fs::read_dir(".") {
            Ok(entries) => {
                for entry_result in entries {
                    match entry_result {
                        Ok(entry) => {
                            let name = entry.file_name().to_string_lossy().into_owned();
                            if name.starts_with('.') {
                                continue;
                            }
                            names.push(name);
                        }
                        Err(e) => eprintln!("Erreur lors de la lecture d'une entrée: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("Erreur lors de l'ouverture du répertoire: {}", e),
        }
        names.sort();
        println!("{}", names.join("  "));
    } else {
        let (flags, mut paths) = parse_flags(rest);
        if paths.len() == 0 {
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
                    return;
                }
            };

            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if !flags.contains(&'a') && file_name.starts_with('.') {
                        continue;
                    }

                    let mut display_name = file_name;

                    if flags.contains(&'F') {
                        if let Ok(meta) = entry.metadata() {
                            if meta.is_dir() {
                                display_name.push('/');
                            }
                        }
                    }

                    names.push(display_name);
                }
            }
            names.sort();

            if flags.contains(&'l') {
                for name in &names {
                    let metadata = match fs::metadata(&name) {
                        Ok(m) => m,
                        Err(_) => {
                            continue;
                        }
                    };
                    let perms = permissions_string(&metadata);
                    let nlink = metadata.nlink();

                    // --- Replacement for 'users' crate ---
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
                    // --- End Replacement ---

                    let size = metadata.len();
                    let mtime = metadata.modified().unwrap_or(UNIX_EPOCH);
                    let datetime: DateTime<Local> = mtime.into();
                    let date = datetime.format("%b %d %H:%M");

                    println!(
                        "{} {:>2} {:<8} {:<8} {:>6} {} {}",
                        perms,
                        nlink,
                        user,
                        group,
                        size,
                        date,
                        name
                    );
                }
            } else {
                println!("{}", names.join("  "));
            }
        }
    }
}

fn permissions_string(metadata: &fs::Metadata) -> String {
    let mut perms = String::new();
    let mode = metadata.mode();
    //Type;
    perms.push(if metadata.is_dir() { 'd' } else { '-' });
    //Owner;
    perms.push(if (mode & 0o400) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o200) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o100) != 0 { 'x' } else { '-' });
    //Group;
    perms.push(if (mode & 0o040) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o020) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o010) != 0 { 'x' } else { '-' });
    //Others;
    perms.push(if (mode & 0o004) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o002) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o001) != 0 { 'x' } else { '-' });

    perms
}