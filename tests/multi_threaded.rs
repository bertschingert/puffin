use puffin::test_libs::*;
use puffin::Args;

#[test]
fn count_files_1() {
    let state = TestState::setup("count_files_1").unwrap();

    state.make_tree("tree", 3, 3, 0, None).unwrap();

    let dir = state.get_path("tree");

    let args = Args {
        path: dir,
        prog: "{ numfiles += 1 } end { print numfiles }".to_string(),
        n_threads: 8,
    };

    let mut buf = Buffer::new();
    puffin::driver(&args, &mut buf);

    buf.trim_newline();
    assert_eq!(buf, "40");

    state.cleanup();
}

#[test]
fn count_files_2() {
    let state = TestState::setup("count_files_2").unwrap();

    state.make_tree("tree", 3, 3, 0, None).unwrap();

    let dir = state.get_path("tree");

    let args = Args {
        path: dir,
        prog: "{ numfiles = numfiles + 1 } end { print numfiles }".to_string(),
        n_threads: 8,
    };

    let mut buf = Buffer::new();
    puffin::driver(&args, &mut buf);

    buf.trim_newline();
    assert_eq!(buf, "40");

    state.cleanup();
}

#[test]
fn count_files_by_size() {
    let state = TestState::setup("count_files_by_size").unwrap();

    state
        .make_tree("size_2", 2, 2, 2, Some(Metadata { size: 2 }))
        .unwrap();

    state
        .make_tree("size_3", 2, 2, 2, Some(Metadata { size: 3 }))
        .unwrap();

    let dir = state.test_subdir();

    let args = Args {
        path: dir,
        prog: ".size <= 3 { numfiles += 1 } end {print numfiles }".to_string(),
        n_threads: 8,
    };

    let mut buf = Buffer::new();
    puffin::driver(&args, &mut buf);

    buf.trim_newline();
    assert_eq!(buf, "24");

    state.cleanup();
}
