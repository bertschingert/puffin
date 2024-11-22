fn setup() {
    std::fs::create_dir("puffin_tests");
}

fn cleanup() {
    std::fs::remove_dir("puffin_tests");
}

#[test]
fn integration_test() {
    setup();

    std::fs::File::create("puffin_tests/hey");

    let path = "puffin_tests";
    let prog = "{ }";

    let mut buf: Vec<u8> = Vec::new();
    puffin::driver(path, prog, &mut buf);

    assert_eq!(buf, "puffin_tests/hey\n".as_bytes());

    cleanup();
}
