use puffin::test_libs::*;
use puffin::Args;

/// Create a test state for `test_name` that runs the program `prog` and expects it to error.
/// Metadata`md`.
fn should_runtime_error(test_name: &str, prog: &str) {
    let state = TestState::setup(test_name).unwrap();

    let path = state
        .create_file(&format!("{test_name}-file"), None)
        .unwrap();

    let args = Args {
        path: path.clone(),
        prog: prog.to_string(),
        n_threads: 1,
    };

    let mut buf = Buffer::new();
    let res = puffin::driver(&args, &mut buf);
    assert!(res.is_err(), "Program should runtime error: '{}'", prog);

    state.cleanup();
}

#[test]
fn special_values_invalid() {
    should_runtime_error("special_values_invalid", "{ print .owner + 1 }");
    should_runtime_error("special_values_invalid", "{ print .owner + .ino }");
    should_runtime_error("special_values_invalid", "{ arr[1] = .ino; arr[1] += 1 }");
}
