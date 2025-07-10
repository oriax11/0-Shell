use std::io::{self, Write};
use std::env;
use std::process::Command;

mod commands;

fn main() {
    loop {
        let current_dir = match env::current_dir() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Error getting current directory: {}", e);
                continue;
            }
        };
        let home_dir = home::home_dir().unwrap_or_default(); // home_dir() can return None, but unwrap_or_default() is fine here
        let display_path = if current_dir.starts_with(&home_dir) {
            format!("~/{}", current_dir.strip_prefix(&home_dir).unwrap_or(&current_dir).display())
        } else {
            format!("{}", current_dir.display())
        };

        if let Err(e) = write!(io::stdout(), "{} $ ", display_path) {
            eprintln!("Error writing to stdout: {}", e);
            continue;
        }
        if let Err(e) = io::stdout().flush() {
            eprintln!("Error flushing stdout: {}", e);
            continue;
        }

        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            eprintln!("Error reading from stdin: {}", e);
            continue;
        }
        if input.is_empty() { // Ctrl+D was pressed
            println!();
            break;
        }

        let mut parts = input.trim().split_whitespace();
        let command = parts.next().unwrap_or("").trim_matches('"').trim_matches('\'');
        let args: Vec<&str> = parts.collect();

        match command {
            "exit" => break,
            "echo" => commands::echo::execute(&args),
            "cd" => commands::cd::execute(&args),
            "ls" => commands::ls::execute(&args),
            "pwd" => commands::pwd::execute(),
            "cat" => commands::cat::execute(&args),
            "cp" => commands::cp::execute(&args),
            "mv" => commands::mv::execute(&args),
            "mkdir" => commands::mkdir::execute(&args),
            "rm" => commands::rm::execute(&args),
            _ => {
                if !command.is_empty() {
                    let mut cmd = Command::new(command);
                    for arg in args {
                        cmd.arg(arg);
                    }
                    match cmd.status() {
                        Ok(status) => {
                            if !status.success() {
                                eprintln!("Command '{}' exited with status: {}", command, status);
                            }
                        },
                        Err(e) => {
                            if e.kind() == io::ErrorKind::NotFound {
                                eprintln!("Command '{}' not found", command);
                            } else {
                                eprintln!("Error executing command '{}': {}", command, e);
                            }
                        }
                    }
                }
            }
        }
    }
}