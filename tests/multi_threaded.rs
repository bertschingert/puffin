use puffin::test_libs::*;
use puffin::Args;

#[test]
fn count_files_1() {
    let state = TestState::setup("count_files_1").unwrap();

    state.make_tree(3, 3, 0).unwrap();

    let dir = state.test_subdir();

    let args = Args {
        path: dir,
        prog: "{ numfiles += 1 } end { print numfiles }".to_string(),
        n_threads: 8,
    };

    let mut buf = Buffer::new();
    puffin::driver(&args, &mut buf);

    buf.trim_newline();
    assert_eq!(buf, "40");

    state.cleanup().unwrap();
}

#[test]
fn count_files_2() {
    let state = TestState::setup("count_files_2").unwrap();

    state.make_tree(3, 3, 0).unwrap();

    let dir = state.test_subdir();

    let args = Args {
        path: dir,
        prog: "{ numfiles = numfiles + 1 } end { print numfiles }".to_string(),
        n_threads: 8,
    };

    let mut buf = Buffer::new();
    puffin::driver(&args, &mut buf);

    buf.trim_newline();
    assert_eq!(buf, "40");

    state.cleanup().unwrap();
}
