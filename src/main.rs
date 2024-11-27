use clap::Parser;

use puffin::Args;

#[derive(Parser)]
struct RawArgs {
    /// The root to start traversing from.
    path: Option<String>,

    /// The program to run.
    prog: Option<String>,

    /// Number of threads.
    #[arg(short = 'j', long, default_value_t = 4)]
    n_threads: usize,
}

fn main() {
    let raw_args = RawArgs::parse();

    let (path, prog) = match raw_args.path {
        Some(ref path) => match raw_args.prog {
            Some(ref prog) => (path.as_str(), prog.as_str()),
            None => usage(),
        },
        None => (".", ""),
    };

    let path = std::path::PathBuf::from(path);

    let args = crate::Args {
        path,
        prog: prog.to_string(),
        n_threads: raw_args.n_threads,
    };

    puffin::driver(&args, &mut std::io::stdout());
}

fn usage() -> ! {
    eprintln!("Usage: puffin [path] [program]");
    std::process::exit(1);
}
