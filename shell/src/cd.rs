use std::env;
use std::fs;

pub fn handle_cd(rest: &[String]) {
    let path = rest.join("/");
      match fs::metadata(&path) {
        Ok(meta) => {
            if meta.is_dir() {
                if let Err(e) = env::set_current_dir(&path) {
                    eprintln!("cd: erreur lors du changement de dossier : {}", e);
                }
            } else {
                eprintln!("cd: ce n'est pas un dossier : {}", path);
            }
        }
        Err(_) => {
            eprintln!("cd: no such file or directory: {}", path);
        }
    }
}
