#![forbid(unsafe_code)]

//! Durable local append helpers shared by JSONL stores.

use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use std::path::Path;

/// Open a local JSONL file for appending at its current end.
///
/// The caller must hold the store's writer lock, so seeking to the end is
/// race-free among cooperating writers. The handle is opened with write access
/// (not append-only access) so a failed write can be rolled back with
/// `set_len`. On Windows an append-only handle cannot truncate the file, which
/// would leave a partial line behind after a write error.
pub(crate) fn open_for_append(path: &Path) -> Result<File, String> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)
        .map_err(|error| format!("open {} for append: {error}", path.display()))?;
    file.seek(SeekFrom::End(0))
        .map_err(|error| format!("seek {} to end: {error}", path.display()))?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{Read, Write};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_PATH: AtomicU64 = AtomicU64::new(0);

    struct TempFile(PathBuf);

    impl TempFile {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "ekpi-local-io-{}-{}.jsonl",
                std::process::id(),
                NEXT_PATH.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_file(&path);
            Self(path)
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    fn read(path: &Path) -> String {
        let mut contents = String::new();
        fs::File::open(path)
            .expect("file opens")
            .read_to_string(&mut contents)
            .expect("file reads");
        contents
    }

    #[test]
    fn append_handle_starts_at_end_and_preserves_prior_bytes() {
        let file = TempFile::new();
        fs::write(&file.0, b"first\n").expect("seed writes");
        let mut handle = open_for_append(&file.0).expect("handle opens");
        handle.write_all(b"second\n").expect("second line writes");
        handle.sync_all().expect("sync");
        assert_eq!(read(&file.0), "first\nsecond\n");
    }

    #[test]
    fn append_handle_can_roll_back_a_partial_write() {
        let file = TempFile::new();
        fs::write(&file.0, b"existing\n").expect("seed writes");
        let mut handle = open_for_append(&file.0).expect("handle opens");
        let original_len = handle.metadata().expect("metadata").len();
        assert_eq!(original_len, 9);

        handle.write_all(b"partial").expect("partial write");
        let rollback = handle.set_len(original_len).and_then(|_| handle.sync_all());
        assert!(
            rollback.is_ok(),
            "the append handle must support rollback: {rollback:?}"
        );
        assert_eq!(read(&file.0), "existing\n");
    }
}
