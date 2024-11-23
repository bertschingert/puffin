use puffin::test_libs::{cleanup, setup, Buffer};

#[test]
fn empty_program() {
    assert!(setup("empty_program").is_ok());

    let path = std::path::PathBuf::from("hey");

    std::fs::File::create(path);

    let path = ".";
    let prog = "{ }";

    let mut buf = Buffer::new();
    puffin::driver(path, prog, &mut buf);

    assert_eq!(buf, "./hey\n");

    cleanup();
}
