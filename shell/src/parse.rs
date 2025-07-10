use crate::pwd;
use crate::cd;
use crate::ls;
use crate::command;
use crate::cat;
use crate::echo;
use std::process;
use regex::Regex;


fn remove_quotes(arg: &str) -> &str {
    let re = Regex::new(r#"^(['"])(.*)['"]$"#).unwrap();
    if let Some(caps) = re.captures(arg) {
        let open = caps.get(1).unwrap().as_str();
        let close = &arg[arg.len()-1..];
        // println!("open: {:?}, close: {:?}", open, close);
        if open == close {
            return caps.get(2).unwrap().as_str();
        }
    }
    arg
}

pub fn parse_data(line: &str) {
    let input = line.trim();
    let raw_args: Vec<&str> = input.split_whitespace().collect();
    let args : Vec<String> = raw_args.iter().map(|a| remove_quotes(a).to_string()).collect();
    if let Some((cmd, rest)) = args.split_first() {
        match cmd.as_str() {
            "echo" => echo::handle_echo(rest),
            "pwd" => pwd::handle_pwd(rest),
            "cd" =>  cd::handle_cd(rest),
            "ls" =>  ls::handle_ls(rest),
            "cat" =>  cat::handle_cat(rest),
            "exit" =>  process::exit(0),
            _ => command::handle_command(cmd, rest),
        }
    }

}