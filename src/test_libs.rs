use std::path::PathBuf;

const TEST_DIR: &str = "puffin_tests";

pub fn setup(test_subdir: &str) -> std::io::Result<()> {
    let subdir = PathBuf::from(TEST_DIR).join(test_subdir);

    let _ = std::fs::create_dir(TEST_DIR);

    match std::fs::create_dir(&subdir) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        e => return e,
    };

    std::env::set_current_dir(&subdir).inspect_err(|e| println!("{e}"))
}

pub fn cleanup() {
    match std::fs::remove_dir(TEST_DIR) {
        Ok(_) => {}
        Err(e) => eprintln!("Cleanup failed: {e}"),
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
