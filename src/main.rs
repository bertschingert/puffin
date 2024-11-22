fn main() {
    let mut args = std::env::args();
    let path = args.nth(1).unwrap();
    let prog = args.nth(0).unwrap();

    puffin::driver(&path, &prog, &mut std::io::stdout());
}
