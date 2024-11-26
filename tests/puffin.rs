use puffin::test_libs::*;

enum ExpectedOutput<'a> {
    String(&'a str),
    Filename,
}

/// Create a test state for `test_name` that creates a single file with optionally supplied
/// Metadata`md`.
///
/// Then, runs `program` given that single file.
///
/// If `expected_output` is a string, then it is compared to the program output. Otherwise,
/// compare against the name of the created file.
fn test_one_file_with_program(
    test_name: &str,
    md: Option<Metadata>,
    program: &str,
    expected_output: ExpectedOutput,
) {
    let state = TestState::setup(test_name).unwrap();

    let path = state.create_file(&format!("{test_name}-file"), md).unwrap();

    let mut buf = Buffer::new();
    puffin::driver(&path, program, &mut buf);

    match expected_output {
        ExpectedOutput::String(out) => assert_eq!(buf, out),
        ExpectedOutput::Filename => {
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
        ExpectedOutput::Filename,
    );
}

#[test]
fn size_greater() {
    test_one_file_with_program(
        "size_greater",
        Some(Metadata { size: 42 }),
        ".size > 42",
        ExpectedOutput::String(""),
    );

    test_one_file_with_program(
        "size_greater",
        Some(Metadata { size: 42 }),
        ".size > 41",
        ExpectedOutput::Filename,
    );

    test_one_file_with_program(
        "size_greater",
        Some(Metadata { size: 42 }),
        ".size >= 43",
        ExpectedOutput::String(""),
    );

    test_one_file_with_program(
        "size_greater",
        Some(Metadata { size: 42 }),
        ".size >= 42",
        ExpectedOutput::Filename,
    );
}

#[test]
fn size_less() {
    test_one_file_with_program(
        "size_less",
        Some(Metadata { size: 42 }),
        ".size < 42",
        ExpectedOutput::String(""),
    );

    test_one_file_with_program(
        "size_less",
        Some(Metadata { size: 42 }),
        ".size < 43",
        ExpectedOutput::Filename,
    );

    test_one_file_with_program(
        "size_less",
        Some(Metadata { size: 42 }),
        ".size <= 41",
        ExpectedOutput::String(""),
    );

    test_one_file_with_program(
        "size_less",
        Some(Metadata { size: 42 }),
        ".size <= 42",
        ExpectedOutput::Filename,
    );
}

#[test]
fn print_statements_1() {
    test_one_file_with_program(
        "print_statements_1",
        None,
        "{ print 1 } end {print 2 }",
        ExpectedOutput::String("1\n2\n"),
    );
}

#[test]
fn print_statements_2() {
    test_one_file_with_program(
        "print_statements_2",
        Some(Metadata { size: 42 }),
        "{ tot = tot + .size } end {print tot }",
        ExpectedOutput::String("42\n"),
    );
}

#[test]
fn plus_equal() {
    test_one_file_with_program(
        "plus_equal",
        Some(Metadata { size: 42 }),
        "{ tot += .size } end {print tot }",
        ExpectedOutput::String("42\n"),
    );
}

#[test]
fn minus_equal() {
    test_one_file_with_program(
        "minus_equal",
        Some(Metadata { size: 42 }),
        "{ tot -= .size } end {print tot }",
        ExpectedOutput::String("-42\n"),
    );
}

#[test]
fn strings() {
    // Empty string should evaluate False and not match the file:
    test_one_file_with_program("strings", None, "\"\"", ExpectedOutput::String(""));

    // String with contents should evaluate True and match the file:
    test_one_file_with_program("strings", None, "\"a\"", ExpectedOutput::Filename);

    test_one_file_with_program(
        "strings",
        None,
        "\"a\" { print \"hey\" }",
        ExpectedOutput::String("hey\n"),
    );
}

#[test]
fn print_paths() {
    let state = TestState::setup("print_paths").unwrap();

    let path = state.create_file(&format!("testfile"), None).unwrap();

    let mut buf = Buffer::new();
    puffin::driver(&path, "{ print .name }", &mut buf);
    buf.trim_newline();
    assert_eq!(buf, "testfile");

    let mut buf = Buffer::new();
    puffin::driver(&path, "{ print .path }", &mut buf);
    buf.trim_newline();
    assert_eq!(buf, &path);

    assert!(state.cleanup().is_ok());
}
