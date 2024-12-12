use std::io::{BufRead, Write};
use std::os::unix::ffi::OsStrExt;
use std::sync::Mutex;

use std::path::{Path, PathBuf};

const TEST_DIR: &str = "puffin_tests";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// A struct representing a test's state.
///
/// Since tests run concurrently, each test gets its own subdirectory within the testing directory.
/// All file creations done by a test should be done via methods on `TestState` so that they are
/// created in the appropriate subdirectory.
pub struct TestState<'a> {
    /// The subdirectory that the test is to be performed in, based on the name of the test.
    test_subdir: &'a str,
}

impl<'a> TestState<'a> {
    pub fn setup(test_name: &'a str) -> std::io::Result<TestState<'a>> {
        let subdir = PathBuf::from(TEST_DIR).join(test_name);

        let _ = std::fs::create_dir(TEST_DIR);

        match std::fs::create_dir(&subdir) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            e => e?,
        };

        Ok(TestState {
            test_subdir: test_name,
        })
    }

    /// Clean up a test's state by removing all files/dirs created by the test.
    ///
    /// This is not implemented in Drop because we want the test's state to be left around if the
    /// test fails so that it can be examined.
    pub fn cleanup(&self) {
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

        remove_recursive(&subdir).unwrap();
    }

    /// Get a PathBuf for a file within the test subdirectory.
    pub fn get_path<P: AsRef<Path>>(&self, filename: P) -> PathBuf {
        std::path::PathBuf::from(TEST_DIR)
            .join(self.test_subdir)
            .join(filename)
    }

    /// Get a PathBuf for the subdirectory that a test should use.
    pub fn test_subdir(&self) -> PathBuf {
        std::path::PathBuf::from(TEST_DIR).join(self.test_subdir)
    }

    /// Create a file in the test's subdirectory named `filename`, and use `metadata` if it is
    /// provided.
    pub fn create_file<P: AsRef<Path>>(
        &self,
        filename: P,
        metadata: Option<Metadata>,
    ) -> Result<PathBuf> {
        let path = self.get_path(filename);
        self.create_file_fullpath(&path, metadata)?;

        Ok(path)
    }

    /// Create a file at `path` with optional `metadata`.
    fn create_file_fullpath(&self, path: &Path, metadata: Option<Metadata>) -> Result<()> {
        match metadata {
            Some(md) => {
                self.create_file_md(path, md)?;
            }
            None => {
                std::fs::File::create(path)?;
            }
        };

        Ok(())
    }

    fn create_file_md(&self, path: &Path, metadata: Metadata) -> Result<()> {
        create_file_ignore_eexist(path)?;
        nix::unistd::truncate(path, metadata.size)?;

        Ok(())
    }

    fn create_n_files(&self, n: usize, parent: &Path, metadata: Option<Metadata>) -> Result<()> {
        for i in 0..n {
            let path = parent.join(format!("file_{i}"));
            self.create_file_fullpath(&path, metadata)?;
        }

        Ok(())
    }

    /// Create a tree named `root` under the test subdirectory with the specified parameters.
    pub fn make_tree(
        &self,
        root_name: &str,
        depth: usize,
        branching_factor: usize,
        files_per_dir: usize,
        file_metadata: Option<Metadata>,
    ) -> Result<()> {
        let root = self.get_path(root_name);
        create_dir_ignore_eexist(&root)?;
        self.make_tree_inner(&root, depth, branching_factor, files_per_dir, file_metadata)
    }

    fn make_tree_inner(
        &self,
        root_path: &Path,
        depth: usize,
        branching_factor: usize,
        files_per_dir: usize,
        file_metadata: Option<Metadata>,
    ) -> Result<()> {
        if depth > 0 {
            for i in 0..branching_factor {
                let subdir = root_path.join(format!("subdir_{i}"));
                create_dir_ignore_eexist(&subdir)?;

                self.create_n_files(files_per_dir, &subdir, file_metadata)?;

                self.make_tree_inner(
                    &subdir,
                    depth - 1,
                    branching_factor,
                    files_per_dir,
                    file_metadata,
                )?;
            }
        }

        Ok(())
    }
}

fn create_file_ignore_eexist<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    match std::fs::File::create(path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(e),
    }
}

fn create_dir_ignore_eexist(path: &Path) -> std::io::Result<()> {
    match std::fs::create_dir(path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(e),
    }
}

/// Metadata to set when creating test files.
#[derive(Copy, Clone)]
pub struct Metadata {
    pub size: i64,
}

pub struct Buffer {
    data: Mutex<Vec<u8>>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
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

    /// Consumes the buffer, returning a vector of the lines as Strings in sorted order.
    pub fn sorted_lines(self) -> Vec<String> {
        let mut buf = self
            .data
            .into_inner()
            .unwrap()
            .lines()
            .map(|s| s.unwrap())
            .collect::<Vec<String>>();
        buf.sort();
        buf
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
