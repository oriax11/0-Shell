pub fn execute(args: &[&str]) {
    let output = args.join(" ").trim_matches('\"').trim_matches('\'').to_string();
    println!("{}", output);
}
