use std::process::Command;

pub fn handle_command(cmd: &str, rest: &[String]) {
    let output = Command::new(cmd)
        .args(rest)
        .output();

    match output {
        Ok(output) => {
            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                print!("Command '{}' not found", cmd)
            }
        }
        Err(_) => {
            println!("Command '{}' not found", cmd)
        }
    }
}
