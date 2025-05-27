use std::env;

pub fn handle_pwd(rest: &[String]) {
    if rest.len() != 0 {
        println!("pwd: too many arguments");
        return
    }
    let path = env::current_dir().unwrap();
    println!("{}", path.display())
}
