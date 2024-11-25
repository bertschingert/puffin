use puffin::test_libs::*;

#[test]
fn empty_program() {
    let name = "empty_program";
    let state = TestState::setup(name).unwrap();

    let path = state.create_file("hey", None).unwrap();

    let dir = state.test_subdir();
    let prog = "{ }";

    let mut buf = Buffer::new();
    puffin::driver(&dir, prog, &mut buf);

    buf.trim_newline();
    assert_eq!(buf, &path);

    assert!(state.cleanup().is_ok());
}

#[test]
fn print_statements_1() {
    let name = "print_statements_1";
    let state = TestState::setup(name).unwrap();

    let _ = state.create_file("hey", None).unwrap();

    let dir = state.test_subdir();
    let prog = "{ print 1 } end {print 2 }";

    let mut buf = Buffer::new();
    puffin::driver(&dir, prog, &mut buf);

    assert_eq!(buf, "1\n2\n");

    assert!(state.cleanup().is_ok());
}

#[test]
fn print_statements_2() {
    let name = "print_statements_2";
    let state = TestState::setup(name).unwrap();

    let _ = state
        .create_file("hey", Some(Metadata { size: 42 }))
        .unwrap();

    let dir = state.test_subdir();
    let prog = "{ tot = tot + .size } end {print tot }";

    let mut buf = Buffer::new();
    puffin::driver(&dir, prog, &mut buf);

    buf.trim_newline();
    assert_eq!(buf, "42");

    assert!(state.cleanup().is_ok());
}
