use puffin::test_libs::{Buffer, TestState};

#[test]
fn empty_program() {
    let name = "empty_program";
    let state = TestState::setup(name).unwrap();

    let path = state.get_path("hey");

    std::fs::File::create(&path);

    let dir = state.test_subdir();
    let prog = "{ }";

    let mut buf = Buffer::new();
    puffin::driver(&dir, prog, &mut buf);

    buf.trim_newline();
    assert_eq!(buf, &path);

    assert!(state.cleanup().is_ok());
}

fn print_statements() {
    let name = "print_statements";
    let state = TestState::setup(name).unwrap();

    let path = state.get_path("hey");

    std::fs::File::create(&path);

    let dir = state.test_subdir();
    let prog = "{ print 1 } end {print 2 }";

    let mut buf = Buffer::new();
    puffin::driver(&path, prog, &mut buf);

    assert_eq!(buf, "1\n2\n");

    assert!(state.cleanup().is_ok());
}
