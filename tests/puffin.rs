use puffin::test_libs::*;

/// Create a test state for `test_name` that creates a single file with optionally supplied
/// Metadata`md`.
///
/// Then, runs `program` given that single file.
///
/// If `expected_output` is supplied, then it is compared to the program output. Otherwise, if it
/// is `None`, the expected output is the filename, so compare against that.
fn test_one_file_with_program(
    test_name: &str,
    md: Option<Metadata>,
    program: &str,
    expected_output: Option<&str>,
) {
    let state = TestState::setup(test_name).unwrap();

    let path = state.create_file(&format!("{test_name}-file"), md).unwrap();

    let mut buf = Buffer::new();
    puffin::driver(&path, program, &mut buf);

    match expected_output {
        Some(out) => assert_eq!(buf, out),
        None => {
            buf.trim_newline();
            assert_eq!(buf, &path);
        }
    };

    assert!(state.cleanup().is_ok());
}

#[test]
fn empty_program() {
    let name = "empty_program";
    let state = TestState::setup(name).unwrap();

    let path = state.create_file("hey", None).unwrap();

    let dir = state.test_subdir();
    let prog = "{ }";

    let mut buf = Buffer::new();
    puffin::driver(&dir, prog, &mut buf);

    assert_eq!(&buf.last_line(), &path);

    assert!(state.cleanup().is_ok());
}

#[test]
fn size_equals() {
    test_one_file_with_program(
        "size_equals",
        Some(Metadata { size: 42 }),
        ".size == 42",
        None,
    );
}

#[test]
fn print_statements_1() {
    test_one_file_with_program(
        "print_statements_1",
        None,
        "{ print 1 } end {print 2 }",
        Some("1\n2\n"),
    );
}

#[test]
fn print_statements_2() {
    test_one_file_with_program(
        "print_statements_2",
        Some(Metadata { size: 42 }),
        "{ tot = tot + .size } end {print tot }",
        Some("42\n"),
    );
}
