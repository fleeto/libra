//! Native Windows key initialization. A protected ACL is supplied at creation,
//! and a locked staging namespace serializes no-replace publication.

use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    path::Path,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow, bail};
use ring::rand::{SecureRandom, SystemRandom};

use super::{
    CAPTURE_DEDUP_SECRET_BYTES, CAPTURE_DEDUP_SECRET_DIR, CAPTURE_DEDUP_SECRET_FILE,
    ensure_capture_dedup_mutation_deadline,
};
use crate::utils::windows_fs;

pub(super) struct CaptureDedupPrivateDir {
    file: File,
}

impl CaptureDedupPrivateDir {
    pub(super) fn open_path_read_only(path: &Path, _label: &str) -> Result<Self> {
        let file = windows_fs::open_private_directory(path)
            .context("open private Windows capture directory without reparse points")?;
        windows_fs::validate_private(&file)
            .context("validate private Windows capture directory ACL")?;
        Ok(Self { file })
    }

    pub(super) fn read_secret(&self) -> Result<Option<[u8; CAPTURE_DEDUP_SECRET_BYTES]>> {
        let mut file =
            match windows_fs::open_at(&self.file, OsStr::new(CAPTURE_DEDUP_SECRET_FILE), false) {
                Ok(file) => file,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(e).context("open private Windows capture key"),
            };
        windows_fs::validate_private(&file).context("validate private Windows capture key ACL")?;
        if windows_fs::information(&file)?.nNumberOfLinks != 1 {
            bail!("private Windows capture key has unexpected hard links; run libra agent doctor");
        }
        let mut secret = [0u8; CAPTURE_DEDUP_SECRET_BYTES];
        file.read_exact(&mut secret)
            .context("read private Windows capture key; run libra agent doctor if damaged")?;
        let mut extra = [0u8; 1];
        if file.read(&mut extra)? != 0 {
            bail!("private Windows capture key has an invalid length; run libra agent doctor");
        }
        Ok(Some(secret))
    }
}

pub(super) fn load_capture_dedup_secret_windows(
    storage: &Path,
    deadline: Option<i64>,
) -> Result<[u8; CAPTURE_DEDUP_SECRET_BYTES]> {
    ensure_capture_dedup_mutation_deadline(deadline)?;
    let storage = windows_fs::open_tree(storage)
        .context("open Windows capture storage without reparse points")?;
    let private =
        windows_fs::create_directory_at(&storage, OsStr::new(CAPTURE_DEDUP_SECRET_DIR), true)
            .context("create owner-only Windows capture key directory")?;
    let private = CaptureDedupPrivateDir { file: private };
    ensure_capture_dedup_mutation_deadline(deadline)?;
    let lock_name = OsStr::new(".agent-capture-dedup-v1.lock");
    let lock = windows_fs::open_private_lock_at(&private.file, lock_name)
        .context("open private Windows capture key lock")?;
    windows_fs::validate_private(&lock)?;
    if windows_fs::information(&lock)?.nNumberOfLinks != 1 {
        bail!("private Windows capture key lock has unexpected hard links");
    }
    let maximum_wait = Instant::now() + Duration::from_secs(30);
    loop {
        ensure_capture_dedup_mutation_deadline(deadline)?;
        if windows_fs::try_lock(&lock).context("lock private Windows capture key initializer")? {
            break;
        }
        if Instant::now() >= maximum_wait {
            bail!(
                "private Windows capture key initialization remained busy for 30 seconds; retry the command"
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    if let Some(secret) = private.read_secret()? {
        return Ok(secret);
    }
    ensure_capture_dedup_mutation_deadline(deadline)?;
    let staging = windows_fs::create_directory_at(
        &private.file,
        OsStr::new("agent-capture-dedup-v1.tmp"),
        true,
    )?;
    // The initializer lock proves there are no active writers. Only our exact
    // temporary name grammar, ACL and single-link regular files are reclaimed.
    for entry in std::fs::read_dir(windows_fs::handle_path(&staging)?)?.take(256) {
        ensure_capture_dedup_mutation_deadline(deadline)?;
        let entry = entry?;
        let name = entry.file_name();
        let text = name.to_string_lossy();
        let Some(suffix) = text.strip_prefix("agent-capture-dedup-v1-") else {
            continue;
        };
        if suffix.len() != 32
            || !suffix
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            continue;
        }
        let Ok(file) = windows_fs::open_at(&staging, &name, false) else {
            continue;
        };
        if windows_fs::validate_private(&file).is_err()
            || windows_fs::information(&file)?.nNumberOfLinks != 1
        {
            continue;
        }
        drop(file);
        windows_fs::remove_private_file_at(&staging, &name)
            .context("remove interrupted Windows capture key staging file")?;
    }
    let mut secret = [0u8; CAPTURE_DEDUP_SECRET_BYTES];
    SystemRandom::new()
        .fill(&mut secret)
        .map_err(|_| anyhow!("generate private Windows capture key"))?;
    let name = format!("agent-capture-dedup-v1-{}", uuid::Uuid::new_v4().simple());
    ensure_capture_dedup_mutation_deadline(deadline)?;
    let mut temporary = windows_fs::create_private_file_at(&staging, OsStr::new(&name))?;
    temporary
        .write_all(&secret)
        .context("write private Windows capture key")?;
    temporary
        .sync_all()
        .context("flush private Windows capture key")?;
    ensure_capture_dedup_mutation_deadline(deadline)?;
    if let Err(error) = windows_fs::publish_private_file(
        &temporary,
        &private.file,
        OsStr::new(CAPTURE_DEDUP_SECRET_FILE),
    ) {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            return private
                .read_secret()?
                .context("concurrent Windows capture key publisher left no key");
        }
        return Err(error).context("publish private Windows capture key atomically");
    }
    Ok(secret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_first_use_has_one_stable_key_and_authentication_does_not_repair_damage() {
        let root = tempfile::tempdir().unwrap();
        let storage = std::sync::Arc::new(root.path().to_path_buf());
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
        let workers: Vec<_> = (0..8)
            .map(|_| {
                let storage = storage.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    load_capture_dedup_secret_windows(&storage, None).unwrap()
                })
            })
            .collect();
        let keys: Vec<_> = workers.into_iter().map(|t| t.join().unwrap()).collect();
        assert!(keys.iter().all(|key| *key == keys[0]));
        assert_eq!(
            super::super::load_capture_dedup_secret(root.path()).unwrap(),
            keys[0]
        );
        let private = root.path().join("private");
        let directory =
            CaptureDedupPrivateDir::open_path_read_only(&private, "key directory").unwrap();
        assert_eq!(directory.read_secret().unwrap(), Some(keys[0]));
        std::fs::write(private.join(CAPTURE_DEDUP_SECRET_FILE), b"damaged").unwrap();
        assert!(directory.read_secret().is_err());
        assert!(load_capture_dedup_secret_windows(root.path(), None).is_err());
        assert_eq!(
            std::fs::read(private.join(CAPTURE_DEDUP_SECRET_FILE)).unwrap(),
            b"damaged"
        );
    }

    #[test]
    fn expired_deadline_cannot_create_private_namespace() {
        let root = tempfile::tempdir().unwrap();
        assert!(load_capture_dedup_secret_windows(root.path(), Some(0)).is_err());
        assert!(!root.path().join("private").exists());
    }
}
