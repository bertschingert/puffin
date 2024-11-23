use puffin::test_libs::{Buffer, TestState};

#[test]
fn empty_program() {
    let name = "empty_program";
    let state = TestState::setup(name).unwrap();

    let path = std::path::PathBuf::from("hey");

    std::fs::File::create(path);

    let path = ".";
    let prog = "{ }";

    let mut buf = Buffer::new();
    puffin::driver(path, prog, &mut buf);

    assert_eq!(buf, "./hey\n");

    assert!(state.cleanup().is_ok());
}
