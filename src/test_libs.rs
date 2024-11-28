use std::io::{BufRead, Write};
use std::os::unix::ffi::OsStrExt;
use std::sync::Mutex;

use std::path::{Path, PathBuf};

const TEST_DIR: &str = "puffin_tests";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct TestState<'a> {
    /// The subdirectory that the test is to be performed in, based on the name of the test.
    test_subdir: &'a str,
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

        Ok(TestState { test_subdir })
    }

    pub fn cleanup(&self) -> std::io::Result<()> {
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

    /// Get a PathBuf for a file within the test subdirectory.
    pub fn get_path(&self, filename: &str) -> PathBuf {
        std::path::PathBuf::from(TEST_DIR)
            .join(self.test_subdir)
            .join(filename)
    }

    /// Get a PathBuf for the subdirectory that a test should use.
    pub fn test_subdir(&self) -> PathBuf {
        std::path::PathBuf::from(TEST_DIR).join(self.test_subdir)
    }

    pub fn create_file(&self, filename: &str, metadata: Option<Metadata>) -> Result<PathBuf> {
        let path = self.get_path(filename);

        match metadata {
            Some(md) => {
                self.create_file_md(&path, md)?;
            }
            None => {
                std::fs::File::create(&path)?;
            }
        };

        Ok(path)
    }

    pub fn make_tree(
        &self,
        depth: usize,
        branching_factor: usize,
        files_per_dir: usize,
    ) -> Result<()> {
        fn make_tree_inner(
            root: &Path,
            depth: usize,
            branching_factor: usize,
            files_per_dir: usize,
        ) -> Result<()> {
            if depth > 0 {
                for i in 0..branching_factor {
                    let subdir = root.join(format!("subdir_{i}"));
                    std::fs::create_dir(&subdir)?;

                    make_tree_inner(&subdir, depth - 1, branching_factor, files_per_dir)?;
                }
            }

            Ok(())
        }

        make_tree_inner(&self.test_subdir(), depth, branching_factor, files_per_dir)
    }

    fn create_file_md(&self, path: &Path, metadata: Metadata) -> Result<()> {
        std::fs::File::create(&path)?;
        Ok(nix::unistd::truncate(path, metadata.size)?)
    }
}

/// Metadata to set when creating test files.
pub struct Metadata {
    pub size: i64,
}

pub struct Buffer {
    data: Mutex<Vec<u8>>,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            data: Mutex::new(Vec::new()),
        }
    }

    pub fn trim_newline(&mut self) {
        match self.data.lock().unwrap().pop() {
            Some(b'\n') => {}
            Some(ch) => panic!("Expected newline, got {:?}", ch),
            None => {}
        }
    }

    /// Get the last line of a buffer.
    ///
    /// Returns OsString rather than String as this is typically compared to a PathBuf, which holds
    /// an OsString.
    pub fn last_line(&self) -> std::ffi::OsString {
        match self.data.lock().unwrap().lines().last() {
            Some(line) => line.unwrap().into(),
            None => panic!("Expected at least one line in string."),
        }
    }
}

impl crate::SyncWrite for Buffer {
    fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.data.lock().unwrap().write(buf)
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match std::str::from_utf8(&self.data.lock().unwrap()) {
            Ok(s) => write!(f, "\"{s}\""),
            Err(_) => write!(f, "\"{:?}\"", self.data),
        }
    }
}

impl PartialEq<&str> for Buffer {
    fn eq(&self, other: &&str) -> bool {
        *self.data.lock().unwrap() == other.as_bytes()
    }
}

impl PartialEq<&PathBuf> for Buffer {
    fn eq(&self, other: &&PathBuf) -> bool {
        *self.data.lock().unwrap() == other.as_os_str().as_bytes()
    }
}
