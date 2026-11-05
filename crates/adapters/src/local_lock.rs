#![forbid(unsafe_code)]

//! Cross-process writer lock for local JSONL stores.
//!
//! The lock is an operating-system advisory lock on a sibling `<PATH>.lock`
//! file. The kernel owns the lock for the lifetime of the open handle, so a
//! writer that crashes or is killed releases it automatically. There is no
//! timestamp-based stale recovery: a second process can never steal a lock that
//! a live process still holds, no matter how long the first process runs.
//!
//! The `.lock` file itself is an empty mutex token and is intentionally left in
//! place after release. Deleting a lock file while another process has it open
//! would break mutual exclusion, so cleanup is deliberately not automatic.
//! Operators may delete an unheld lock file only after confirming that no
//! writer is active. The lock file must live in a directory writable only by
//! the operator; a shared, attacker-controlled directory is outside the local
//! threat model.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

const RETRY_LIMIT: usize = 200;
const RETRY_DELAY: Duration = Duration::from_millis(25);

#[derive(Debug)]
pub(crate) struct LocalFileLock {
    file: File,
}

impl LocalFileLock {
    pub(crate) fn acquire(target: &Path) -> Result<Self, String> {
        Self::acquire_with(target, RETRY_LIMIT, RETRY_DELAY)
    }

    fn acquire_with(
        target: &Path,
        retry_limit: usize,
        retry_delay: Duration,
    ) -> Result<Self, String> {
        let path = lock_path(target);
        reject_legacy_or_unsafe_lock_path(&path)?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|error| format!("open lock {}: {error}", path.display()))?;

        for attempt in 0..=retry_limit {
            match file.try_lock() {
                Ok(()) => return Ok(Self { file }),
                Err(TryLockError::WouldBlock) => {
                    if attempt < retry_limit {
                        std::thread::sleep(retry_delay);
                    }
                }
                Err(TryLockError::Error(error)) => {
                    return Err(format!("lock {}: {error}", path.display()));
                }
            }
        }
        Err(format!(
            "{} is locked by another writer (lock file {})",
            target.display(),
            path.display()
        ))
    }
}

impl Drop for LocalFileLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

pub(crate) fn lock_path(target: &Path) -> PathBuf {
    let mut name = target.as_os_str().to_os_string();
    name.push(".lock");
    PathBuf::from(name)
}

/// Refuse paths that cannot be an OS lock file. A directory is a leftover from
/// the previous directory-lock format; following a symbolic link could lock or
/// modify an unrelated file.
fn reject_legacy_or_unsafe_lock_path(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!(
            "lock path {} is a symbolic link; refusing to follow it",
            path.display()
        )),
        Ok(metadata) if metadata.is_dir() => Err(format!(
            "lock path {} is a directory left by the previous directory-lock format; \
             confirm no old writer is running, remove it, and retry",
            path.display()
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("inspect lock {}: {error}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_PATH: AtomicU64 = AtomicU64::new(0);

    struct TempTarget(PathBuf);

    impl TempTarget {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "ekpi-local-lock-{}-{}.jsonl",
                std::process::id(),
                NEXT_PATH.fetch_add(1, Ordering::Relaxed)
            ));
            cleanup_lock(&path);
            Self(path)
        }
    }

    impl Drop for TempTarget {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
            cleanup_lock(&self.0);
        }
    }

    fn cleanup_lock(target: &Path) {
        let lock = lock_path(target);
        let _ = fs::remove_file(&lock);
        let _ = fs::remove_dir_all(&lock);
    }

    #[test]
    fn an_active_lock_blocks_a_second_writer() {
        let target = TempTarget::new();
        let lock = LocalFileLock::acquire(&target.0).expect("first lock acquires");
        let started = std::time::Instant::now();
        let error = LocalFileLock::acquire_with(&target.0, 2, Duration::from_millis(1))
            .expect_err("active lock is not stolen");
        assert!(error.contains("locked by another writer"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(2));
        drop(lock);
        let second = LocalFileLock::acquire_with(&target.0, 0, Duration::from_millis(1))
            .expect("released lock is reacquired");
        drop(second);
        assert!(
            lock_path(&target.0).exists(),
            "the lock file is retained so another process cannot bypass the lock by recreation"
        );
    }

    #[test]
    fn a_leftover_lock_file_does_not_block_a_new_writer() {
        let target = TempTarget::new();
        fs::write(lock_path(&target.0), b"left by a crashed writer")
            .expect("leftover lock file writes");
        let lock = LocalFileLock::acquire_with(&target.0, 0, Duration::from_millis(1))
            .expect("an unheld lock file carries no lock");
        drop(lock);
    }

    #[test]
    fn closing_the_holding_handle_releases_the_lock() {
        let target = TempTarget::new();
        let path = lock_path(&target.0);
        let raw = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .expect("raw lock file opens");
        raw.try_lock().expect("raw handle acquires the lock");
        let error = LocalFileLock::acquire_with(&target.0, 0, Duration::from_millis(1))
            .expect_err("a live handle is respected");
        assert!(error.contains("locked by another writer"), "{error}");

        // Closing the last handle releases the advisory lock. The kernel does
        // this when a process exits, so a crashed writer needs no stale timeout.
        drop(raw);
        let lock = LocalFileLock::acquire_with(&target.0, 0, Duration::from_millis(1))
            .expect("the kernel released the lock with the handle");
        drop(lock);
    }

    #[test]
    fn a_legacy_lock_directory_is_refused() {
        let target = TempTarget::new();
        fs::create_dir(lock_path(&target.0)).expect("legacy lock directory creates");
        let error = LocalFileLock::acquire_with(&target.0, 0, Duration::from_millis(1))
            .expect_err("a legacy directory is not silently replaced");
        assert!(error.contains("previous directory-lock format"), "{error}");
        assert!(
            lock_path(&target.0).is_dir(),
            "the legacy directory is left for the operator to inspect"
        );
    }

    #[test]
    fn a_symbolic_link_lock_path_is_refused() {
        let target = TempTarget::new();
        let link = lock_path(&target.0);
        let destination = target.0.with_extension("link-target");
        fs::write(&destination, b"do not lock or truncate this file").expect("target writes");
        if !try_create_symlink(&destination, &link) {
            // Windows may require Developer Mode or SeCreateSymbolicLinkPrivilege.
            // The refusal itself is covered by the path-type check when a link
            // can be created; skip only this platform capability.
            let _ = fs::remove_file(&destination);
            return;
        }
        let error = LocalFileLock::acquire_with(&target.0, 0, Duration::from_millis(1))
            .expect_err("a symlink is not followed");
        assert!(error.contains("symbolic link"), "{error}");
        assert_eq!(
            fs::read(&destination).expect("target remains readable"),
            b"do not lock or truncate this file"
        );
        let _ = fs::remove_file(&link);
        let _ = fs::remove_file(&destination);
    }

    #[cfg(unix)]
    fn try_create_symlink(destination: &Path, link: &Path) -> bool {
        std::os::unix::fs::symlink(destination, link).is_ok()
    }

    #[cfg(windows)]
    fn try_create_symlink(destination: &Path, link: &Path) -> bool {
        std::os::windows::fs::symlink_file(destination, link).is_ok()
    }
}
