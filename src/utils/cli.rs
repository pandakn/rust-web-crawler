use std::env;

pub fn print_usage() {
    println!("\n\x1b[1;34mUsage:\x1b[0m");
    println!("  \x1b[36mcargo run -- https://www.example.com\x1b[0m\n");
}

pub fn parse_arguments() -> Option<String> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => None,
        len if len >= 2 => Some(args[1].clone()),
        _ => None,
    }
}
