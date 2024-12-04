use puffin::test_libs::*;
use puffin::Args;

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
    prog: &str,
    expected_output: ExpectedOutput,
) {
    let state = TestState::setup(test_name).unwrap();

    let path = state.create_file(&format!("{test_name}-file"), md).unwrap();

    let args = Args {
        path: path.clone(),
        prog: prog.to_string(),
        n_threads: 1,
    };

    let mut buf = Buffer::new();
    puffin::driver(&args, &mut buf).unwrap();

    match expected_output {
        ExpectedOutput::String(out) => assert_eq!(buf, out, "program: {}", prog),
        ExpectedOutput::Filename => {
            buf.trim_newline();
            assert_eq!(buf, &path, "program: {}", prog);
        }
    };

    state.cleanup();
}

#[test]
fn empty_program() {
    let name = "empty_program";
    let state = TestState::setup(name).unwrap();

    let path = state.create_file("hey", None).unwrap();

    let dir = state.test_subdir();
    let prog = "{ }";

    let args = Args {
        path: dir,
        prog: prog.to_string(),
        n_threads: 1,
    };

    let mut buf = Buffer::new();
    puffin::driver(&args, &mut buf).unwrap();

    assert_eq!(&buf.last_line(), &path);

    state.cleanup();
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

    let args = Args {
        path: path.clone(),
        prog: "{ print .name }".to_string(),
        n_threads: 1,
    };

    let mut buf = Buffer::new();
    puffin::driver(&args, &mut buf).unwrap();
    buf.trim_newline();
    assert_eq!(buf, "testfile");

    let args = Args {
        path: path.clone(),
        prog: "{ print .path }".to_string(),
        n_threads: 1,
    };

    let mut buf = Buffer::new();
    puffin::driver(&args, &mut buf).unwrap();
    buf.trim_newline();
    assert_eq!(buf, &path);

    state.cleanup();
}

#[test]
fn print_statements() {
    test_one_file_with_program(
        "print_statements",
        None,
        "{ print \"hey\"; print \"there\" }",
        ExpectedOutput::String("hey\nthere\n"),
    );

    test_one_file_with_program(
        "print_statements",
        Some(Metadata { size: 69 }),
        "{ print \"hey\", \"there\"; print \"size:\", .size }",
        ExpectedOutput::String("hey there\nsize: 69\n"),
    );
}

#[test]
fn variables() {
    test_one_file_with_program(
        "variables",
        None,
        "{ var1 = 1; var2 = 2} end {print var1, var2}",
        ExpectedOutput::String("1 2\n"),
    );

    test_one_file_with_program(
        "variables",
        None,
        "{ var1 = 1; var2 = 2; var1 = var2} end {print var1, var2}",
        ExpectedOutput::String("2 2\n"),
    );

    test_one_file_with_program(
        "variables",
        None,
        "begin {var = 1} var { print var + 2 }",
        ExpectedOutput::String("3\n"),
    );

    test_one_file_with_program(
        "variables",
        None,
        "begin {var = 0} var { print var + 2 }",
        ExpectedOutput::String(""),
    );

    test_one_file_with_program(
        "variables",
        None,
        "begin {var1 = 1; var2 = 2} var2 - var1 { print var1 + var2 }",
        ExpectedOutput::String("3\n"),
    );
}

#[test]
fn arrays() {
    fn expect_output(prog: &str, output: &str) {
        let output = match output {
            "" => "",
            _ => &format!("{}\n", output),
        };
        test_one_file_with_program("arrays", None, prog, ExpectedOutput::String(output));
    }

    expect_output(
        "{ arr[1] = 1; arr2[2] = 2} end {print arr[1], arr2[2]}",
        "1 2",
    );

    expect_output(
        "{ arr[1] = 1; arr[\"key\"] = 2; arr[1] = arr[\"key\"]} end {print arr[1], arr[\"key\"]}",
        "2 2",
    );

    expect_output("{ arr[1] = 1} arr {print arr[1]}", "1");

    expect_output("{ arr[1] = 1} arr[\"key\"] {print \"output\"}", "");

    expect_output(
        "{ arr[1] = 1} arr[\"key\"] + arr[1] {print \"output\"}",
        "output",
    );

    expect_output(
        "{arr[1] = 1; arr2[arr[1]] = 2; arr[1] += arr2[arr[1]]} end {print arr2[arr[1]], arr2[1], arr[1]}",
        "0 2 3");

    expect_output(
        "{arr[1] = \"hey\"; arr[2] = \"there\"; print arr[1], arr[2]}",
        "hey there",
    );
}

#[test]
fn expressions() {
    fn expect_output(prog: &str, output: &str) {
        let output = match output {
            "" => "",
            _ => &format!("{}\n", output),
        };
        test_one_file_with_program("expressions", None, prog, ExpectedOutput::String(output));
    }

    expect_output("{ print 1 + 2 * 3 }", "7");
    expect_output("{ print (1 + 2) * 3}", "9");
    expect_output("{ print 2 + 3 * 2 / 3 }", "4");
    expect_output("0 or 1 { print 9 }", "9");
    expect_output("1 and 1 { print 9 }", "9");
    expect_output("0 and 1 { print 9 }", "");
    expect_output("1 and 0 { print 9 }", "");
    expect_output("0 and 0 { print 9 }", "");
    expect_output("0 or 0 { print 9 }", "");
    expect_output("0 or 3 - 3 { print 9 }", "");
    expect_output("0 or 3 + 3 { print 9 }", "9");
    expect_output("0 + 1 - 1 or 3 + 3 * 2 / 3 - 5 { print 9 }", "");
    expect_output("{print (1 > 2) * 3}", "0");
    expect_output("{print (1 < 2) * 3}", "3");
    expect_output("{print (((1 + 2) * 3) - 1) * 5 }", "40");
}
