use std::fs;

pub fn handle_ls(rest: &[String]) {
    if rest.len() == 0 {
        let mut names = Vec::new();
        match fs::read_dir(".") {
            Ok(entries) => {
                for entry_result in entries {
                    match entry_result {
                        Ok(entry) => {
                            let name = entry.file_name().to_string_lossy().into_owned();
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
    } 
}
