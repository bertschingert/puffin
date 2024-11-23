use std::path::{Path, PathBuf};

const TEST_DIR: &str = "puffin_tests";

pub struct TestState<'a> {
    /// The subdirectory that the test is to be performed in, based on the name of the test.
    test_subdir: &'a str,

    /// The original CWD of the process, so that it can be restored after the test finishes.
    original_cwd: PathBuf,
}

impl<'a> TestState<'a> {
    pub fn setup(test_subdir: &'a str) -> std::io::Result<TestState> {
        let subdir = PathBuf::from(TEST_DIR).join(test_subdir);

        let _ = std::fs::create_dir(TEST_DIR);

        match std::fs::create_dir(&subdir) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            e => e?,
        };

        let original_cwd = std::env::current_dir()?;

        std::env::set_current_dir(&subdir)?;

        Ok(TestState {
            test_subdir,
            original_cwd,
        })
    }

    pub fn cleanup(&self) -> std::io::Result<()> {
        std::env::set_current_dir(&self.original_cwd)?;

        let subdir = PathBuf::from(TEST_DIR).join(self.test_subdir);

        fn remove_recursive(path: &Path) -> std::io::Result<()> {
            for ent in std::fs::read_dir(path).unwrap() {
                let ent = ent?;
                let ty = ent.file_type()?;
                if ty.is_dir() {
                    remove_recursive(&ent.path())?;
                } else {
                    std::fs::remove_file(ent.path())?;
                }
            }

            std::fs::remove_dir(path)
        }

        remove_recursive(&subdir)
    }
}

pub struct Buffer {
    data: Vec<u8>,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer { data: Vec::new() }
    }
}

impl std::io::Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.data.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.data.flush()
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match std::str::from_utf8(&self.data) {
            Ok(s) => write!(f, "{s}"),
            Err(_) => write!(f, "{:?}", self.data),
        }
    }
}

impl PartialEq<&str> for Buffer {
    fn eq(&self, other: &&str) -> bool {
        self.data == other.as_bytes()
    }
}
