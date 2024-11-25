fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (path, prog) = match args.len() {
        // If no arguments are provided, run the default program on the CWD:
        1 => (".", ""),
        // If only one argument is provided, it's ambiguous if it's a path or a program.
        // TODO: Find a good way to "intelligently" guess whether the argument should be
        // interpreted as a path or program?
        2 => usage(),
        // If two arguments are provided, they are the path and the program:
        3 => (args[1].as_str(), args[2].as_str()),
        _ => usage(),
    };

    let path = std::path::PathBuf::from(path);

    puffin::driver(&path, &prog, &mut std::io::stdout());
}

fn usage() -> ! {
    eprintln!("Usage: puffin [path] [program]");
    std::process::exit(1);
}
