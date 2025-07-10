pub fn execute(args: &[&str]) {
    println!("{:?}", args);
    let output = args.join(" ").replace(['\'', '\"'], "");
    println!("{}", output);
}
