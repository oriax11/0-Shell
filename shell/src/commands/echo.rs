pub fn execute(args: &[&str]) {
    let output = args.join(" ").replace(['\'', '\"'], "");
    println!("{}", output);
}
