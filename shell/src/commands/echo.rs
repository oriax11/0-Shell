pub fn execute(args: &[String]) {
    for arg in args {
        if arg.ends_with('!') {
            println!("!: event not found");
            return;
        }
    }
    let mut text = args.join(" "); 
    let mut trailing_newline = true;

    if args.len() == 1 && args[0] == "-n" {
        trailing_newline = false;
        text = "".to_string();
    } else if args.len() > 1 && args[0] == "-n" {
        trailing_newline = false;
        text = args[1..].join(" ");
    }

    print!("{}", text);
    if trailing_newline {
        println!();
    }
}