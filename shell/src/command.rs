use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub fn handle_command(cmd: &str, rest: &[String]) {
    let child = Command::new(cmd)
        .args(rest)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match child {
        Ok(mut child) => {
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                for line in reader.lines().flatten() { // prevent infinite output
                    println!("{}", line);
                }
            }

        }
        Err(_) => {
            println!("Command '{}' not found", cmd);
        }
    }
}
