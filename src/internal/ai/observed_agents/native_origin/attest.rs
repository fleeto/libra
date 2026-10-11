//! Bounded, domain-separated installation challenge. Successful attestation
//! proves key/binary association only; it creates no receipt or artifact.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
#[cfg(unix)]
use std::{fs::OpenOptions, io::Read};

#[cfg(not(unix))]
use anyhow::bail;
use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::STANDARD};
use ring::{
    rand::{SecureRandom, SystemRandom},
    signature,
};
use sea_orm::ConnectionTrait;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
};

use super::{
    approved_build, codec,
    registry::{IssuerRecord, IssuerState, NativeOriginError, decode_owned, lower_hex},
};
use crate::internal::ai::observed_agents::{
    rpc::{RPC_ENV_PASSTHROUGH_ALLOWLIST, RPC_MAX_STDERR_BYTES},
    trust::{self, TrustRecord},
};

const ATTEST_BYTE_CAP: usize = 4096;
const ATTEST_DEADLINE: Duration = Duration::from_secs(30);
const EXECUTABLE_BYTE_CAP: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Installation {
    version: u32,
    build_identity: String,
    producer_binary_digest: String,
    issuer_public_key: String,
    key_generation: u32,
    policy_descriptor: codec::Policy,
    policy_digest: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Attestation {
    version: u32,
    challenge: String,
    issuer: String,
    public_key: String,
    key_generation: u32,
    build_identity: String,
    producer_version: String,
    producer_binary_digest: String,
    policy_digest: String,
    signature: String,
}

pub(super) struct PreparedRegistration {
    pub(super) exporter_trust: TrustRecord,
    pub(super) record: IssuerRecord,
    // Transient actual executable measurement. It is never a signed policy or
    // persisted registry field and need not match a historical Libra build.
    pub(super) observer_executable_sha256: String,
}

struct LocalBinding {
    installation: Installation,
    key_dir: PathBuf,
    command_path: PathBuf,
    observer_executable_sha256: String,
}

/// No-follow, nonblocking open refuses symlinks and FIFOs before a read can
/// block. Compare identity and posture on the descriptor and pathname again.
/// This is the existing best-effort local-user tier, not an fd-exec guarantee.
#[cfg(unix)]
fn read_local_file(path: &Path, cap: usize, owned_mode: Option<u32>) -> Result<Vec<u8>> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

    let parent = path.parent().ok_or(NativeOriginError::Descriptor)?;
    if !path.is_absolute() || parent.canonicalize()? != parent {
        return Err(NativeOriginError::Descriptor.into());
    }
    let directory = fs::symlink_metadata(parent)?;
    // SAFETY: geteuid reads this process's uid and has no pointer arguments.
    let uid = unsafe { libc::geteuid() };
    if !directory.is_dir()
        || directory.uid() != uid
        || directory.permissions().mode() & 0o022 != 0
        || (owned_mode.is_some() && directory.permissions().mode() & 0o7777 != 0o700)
    {
        return Err(NativeOriginError::Descriptor.into());
    }
    let before = fs::symlink_metadata(path)?;
    let check = |meta: &fs::Metadata| {
        meta.is_file()
            && meta.uid() == uid
            && meta.len() <= cap as u64
            && meta.permissions().mode() & 0o022 == 0
            && owned_mode.is_none_or(|mode| meta.permissions().mode() & 0o7777 == mode)
    };
    if !check(&before) {
        return Err(NativeOriginError::Descriptor.into());
    }
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    let opened = file.metadata()?;
    if !check(&opened) || opened.dev() != before.dev() || opened.ino() != before.ino() {
        return Err(NativeOriginError::Descriptor.into());
    }
    let mut bytes = Vec::new();
    (&mut file).take((cap + 1) as u64).read_to_end(&mut bytes)?;
    let after = fs::symlink_metadata(path)?;
    let final_meta = file.metadata()?;
    if bytes.len() > cap
        || bytes.len() as u64 != opened.len()
        || !check(&after)
        || final_meta.dev() != opened.dev()
        || final_meta.ino() != opened.ino()
        || final_meta.len() != opened.len()
        || final_meta.mtime() != opened.mtime()
        || final_meta.mtime_nsec() != opened.mtime_nsec()
        || after.dev() != opened.dev()
        || after.ino() != opened.ino()
        || after.len() != opened.len()
        || after.mtime() != opened.mtime()
        || after.mtime_nsec() != opened.mtime_nsec()
    {
        return Err(NativeOriginError::Descriptor.into());
    }
    Ok(bytes)
}

#[cfg(not(unix))]
fn read_local_file(_path: &Path, _cap: usize, _owned_mode: Option<u32>) -> Result<Vec<u8>> {
    bail!("Native-origin registration requires the reviewed macOS private producer installation")
}

fn measure_executable(path: &Path) -> Result<trust::Provenance> {
    let canonical = path
        .canonicalize()
        .context("resolve controlled native-origin executable")?;
    let meta = fs::symlink_metadata(&canonical)?;
    if !meta.is_file() || meta.len() == 0 || meta.len() > EXECUTABLE_BYTE_CAP {
        return Err(NativeOriginError::PriorTrust.into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        // SAFETY: geteuid reads this process's uid and has no pointer arguments.
        let uid = unsafe { libc::geteuid() };
        if (meta.uid() != uid && meta.uid() != 0)
            || meta.permissions().mode() & 0o022 != 0
            || meta.permissions().mode() & 0o111 == 0
        {
            return Err(NativeOriginError::PriorTrust.into());
        }
    }
    trust::ensure_parent_not_world_writable(&canonical)?;
    trust::compute_provenance(&canonical)
}

fn local_binding(worktree: &Path, manifest: &Path, trusted: &TrustRecord) -> Result<LocalBinding> {
    if manifest
        .file_name()
        .is_none_or(|name| name != "installation.json")
    {
        return Err(NativeOriginError::Descriptor.into());
    }
    let raw = read_local_file(manifest, codec::DESCRIPTOR_BYTE_CAP, Some(0o600))
        .context("read owned public installation.json; Libra never reads the private key")?;
    let installation: Installation = decode_owned(&raw)?;
    lower_hex(&installation.build_identity, 16)?;
    lower_hex(&installation.producer_binary_digest, 32)?;
    lower_hex(&installation.issuer_public_key, 32)?;
    lower_hex(&installation.policy_digest, 32)?;
    if installation.version != 1 || installation.key_generation == 0 {
        return Err(NativeOriginError::Descriptor.into());
    }
    let provenance = measure_executable(&trusted.path)?;
    if trust::provenance_drifted(&provenance, trusted)
        || provenance.sha256 != installation.producer_binary_digest
    {
        return Err(NativeOriginError::PriorTrust.into());
    }
    // Select the catalog BEFORE passing a private installation directory to a
    // child. A trusted ordinary exporter is not an approved attest producer.
    let build = approved_build::lookup(
        codec::PRODUCER_VERSION,
        &installation.build_identity,
        &provenance.sha256,
    )?;
    let command = measure_executable(
        &std::env::current_exe()
            .context("locate current Libra executable for native-origin registration")?,
    )?;
    let command_path = command.canonical_path;
    let command_text = command_path.to_str().ok_or(NativeOriginError::Observer)?;
    let observer = read_local_file(
        &worktree.join(".opencode/plugin/libra-hooks.js"),
        codec::DESCRIPTOR_BYTE_CAP,
        None,
    )
    .map_err(|_| NativeOriginError::Observer)?;
    let policy = build.installation_policy(command_text, &observer)?;
    let policy_digest = hex::encode(Sha256::digest(codec::policy_bytes(&policy)?));
    if installation.policy_descriptor != policy || installation.policy_digest != policy_digest {
        return Err(NativeOriginError::Observer.into());
    }
    Ok(LocalBinding {
        installation,
        key_dir: manifest
            .parent()
            .ok_or(NativeOriginError::Descriptor)?
            .to_owned(),
        command_path,
        observer_executable_sha256: command.sha256,
    })
}

async fn bounded_read<R: AsyncRead + Unpin>(mut reader: R, cap: usize) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut chunk = [0u8; 1024];
    loop {
        let n = reader
            .read(&mut chunk)
            .await
            .context("read native-origin child pipe")?;
        if n == 0 {
            return Ok(result);
        }
        if result.len() + n > cap {
            return Err(NativeOriginError::ChildBounds.into());
        }
        result.extend_from_slice(&chunk[..n]);
    }
}

struct AttestChild {
    child: Child,
    process_group: Option<i32>,
}

impl AttestChild {
    fn terminate(&mut self) {
        #[cfg(unix)]
        if let Some(pid) = self.process_group {
            // SAFETY: this positive pid is the newly spawned child's private
            // process group (process_group(0)); no caller-supplied pid is used.
            unsafe {
                libc::kill(-pid, libc::SIGKILL);
            }
        }
        let _ = self.child.start_kill();
    }

    async fn terminate_and_reap(&mut self) -> Result<()> {
        self.terminate();
        tokio::time::timeout(Duration::from_secs(1), self.child.wait())
            .await
            .context("native-origin child did not reap after termination")?
            .context("reap terminated native-origin child")?;
        self.process_group = None;
        Ok(())
    }
}

impl Drop for AttestChild {
    fn drop(&mut self) {
        if self.process_group.is_some() {
            self.terminate();
        }
        // Tokio's kill_on_drop additionally covers cancellation on platforms
        // without process groups, and its child owner reaps a dropped child.
    }
}

async fn run_child(
    binary: &Path,
    worktree: &Path,
    key_dir: &Path,
    command_path: &Path,
    challenge: &str,
    deadline: Duration,
) -> Result<Vec<u8>> {
    let mut command = Command::new(binary);
    command
        .args(["native-origin", "attest", "--challenge", challenge])
        .current_dir(worktree)
        .env_clear()
        .env("OPENCODE_LIBRA_NATIVE_ORIGIN", "1")
        .env("OPENCODE_LIBRA_NATIVE_KEY_DIR", key_dir)
        .env("OPENCODE_LIBRA_COMMAND", command_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    for name in RPC_ENV_PASSTHROUGH_ALLOWLIST {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.as_std_mut().process_group(0);
    }
    let child = command
        .spawn()
        .context("start reviewed private producer challenge")?;
    let group = child.id().and_then(|pid| i32::try_from(pid).ok());
    let mut owner = AttestChild {
        child,
        process_group: group,
    };
    let stdout = owner
        .child
        .stdout
        .take()
        .ok_or(NativeOriginError::Challenge)?;
    let stderr = owner
        .child
        .stderr
        .take()
        .ok_or(NativeOriginError::Challenge)?;
    let result = tokio::time::timeout(deadline, async {
        let (stdout, _stderr, status) = tokio::try_join!(
            bounded_read(stdout, ATTEST_BYTE_CAP),
            bounded_read(stderr, RPC_MAX_STDERR_BYTES),
            async {
                owner
                    .child
                    .wait()
                    .await
                    .context("wait for native-origin challenge")
            },
        )?;
        if !status.success() {
            return Err(NativeOriginError::Challenge.into());
        }
        Ok(stdout)
    })
    .await;
    match result {
        Ok(Ok(bytes)) => {
            owner.process_group = None;
            Ok(bytes)
        }
        other => {
            let error = match other {
                Ok(Err(error)) => error,
                Err(_) => NativeOriginError::ChildBounds.into(),
                Ok(Ok(_)) => return Err(NativeOriginError::Challenge.into()),
            };
            owner
                .terminate_and_reap()
                .await
                .with_context(|| format!("native-origin challenge refused: {error}"))?;
            // Captured stderr never reaches output, even on failure. The
            // fixed actionable diagnostic is the only surfaced child text.
            Err(error)
        }
    }
}

fn verify_challenge(bytes: &[u8], challenge: &str, local: &LocalBinding) -> Result<IssuerRecord> {
    if bytes.len() > ATTEST_BYTE_CAP || bytes.last() != Some(&b'\n') {
        return Err(NativeOriginError::Challenge.into());
    }
    let response: Attestation =
        decode_owned(&bytes[..bytes.len() - 1]).map_err(|_| NativeOriginError::Challenge)?;
    let installation = &local.installation;
    let public_key = lower_hex(&response.public_key, 32)?;
    let issuer = hex::encode(Sha256::digest(&public_key));
    if response.version != 1
        || response.challenge != challenge
        || response.issuer != issuer
        || response.public_key != installation.issuer_public_key
        || response.key_generation != installation.key_generation
        || response.build_identity != installation.build_identity
        || response.producer_version != codec::PRODUCER_VERSION
        || response.producer_binary_digest != installation.producer_binary_digest
        || response.policy_digest != installation.policy_digest
    {
        return Err(NativeOriginError::Challenge.into());
    }
    approved_build::lookup(
        &response.producer_version,
        &response.build_identity,
        &response.producer_binary_digest,
    )?;
    let signature = STANDARD
        .decode(&response.signature)
        .map_err(|_| NativeOriginError::Challenge)?;
    if signature.len() != 64 || STANDARD.encode(&signature) != response.signature {
        return Err(NativeOriginError::Challenge.into());
    }
    let mut preimage = b"libra-opencode-native-origin-attest-v1\0".to_vec();
    preimage.extend_from_slice(&response.version.to_be_bytes());
    preimage.extend(lower_hex(challenge, 32)?);
    preimage.extend(lower_hex(&issuer, 32)?);
    preimage.extend(&public_key);
    preimage.extend_from_slice(&response.key_generation.to_be_bytes());
    preimage.extend(lower_hex(&response.build_identity, 16)?);
    let version_len =
        u32::try_from(response.producer_version.len()).map_err(|_| NativeOriginError::Challenge)?;
    preimage.extend_from_slice(&version_len.to_be_bytes());
    preimage.extend_from_slice(response.producer_version.as_bytes());
    preimage.extend(lower_hex(&response.producer_binary_digest, 32)?);
    preimage.extend(lower_hex(&response.policy_digest, 32)?);
    signature::UnparsedPublicKey::new(&signature::ED25519, &public_key)
        .verify(&preimage, &signature)
        .map_err(|_| NativeOriginError::Challenge)?;
    Ok(IssuerRecord {
        issuer,
        public_key: response.public_key,
        key_generation: response.key_generation,
        build_identity: response.build_identity,
        producer_version: response.producer_version,
        producer_binary_digest: response.producer_binary_digest,
        producer_path: String::new(), // Set only from the actual trust owner below.
        policy_descriptor: installation.policy_descriptor.clone(),
        policy_digest: response.policy_digest,
        state: IssuerState::Active,
    })
}

pub(super) async fn prepare<C: ConnectionTrait>(
    conn: &C,
    worktree: &Path,
    manifest: &Path,
) -> Result<PreparedRegistration> {
    let exporter_trust = trust::read_trust_with_conn(conn, trust::OPENCODE_EXPORTER_SLUG)
        .await?
        .ok_or(NativeOriginError::PriorTrust)?;
    let dirs = trust::read_trusted_dirs_with_conn(conn).await?;
    if !trust::path_within_trusted_dirs(&exporter_trust.path, &dirs) {
        return Err(NativeOriginError::PriorTrust.into());
    }
    let repo = worktree.to_owned();
    let descriptor_path = manifest.to_owned();
    let trusted = exporter_trust.clone();
    let local =
        tokio::task::spawn_blocking(move || local_binding(&repo, &descriptor_path, &trusted))
            .await
            .context("measure native-origin installation binding")??;
    let mut nonce = [0u8; 32];
    SystemRandom::new()
        .fill(&mut nonce)
        .map_err(|_| NativeOriginError::Challenge)?;
    let challenge = hex::encode(nonce);
    let bytes = run_child(
        &exporter_trust.path,
        worktree,
        &local.key_dir,
        &local.command_path,
        &challenge,
        ATTEST_DEADLINE,
    )
    .await?;
    let mut record = verify_challenge(&bytes, &challenge, &local)?;
    let trusted = exporter_trust.clone();
    let unchanged = tokio::task::spawn_blocking(move || -> Result<()> {
        let measured = measure_executable(&trusted.path)?;
        if trust::provenance_drifted(&measured, &trusted) {
            return Err(NativeOriginError::PriorTrust.into());
        }
        Ok(())
    })
    .await
    .context("recheck exporter after native-origin challenge")?;
    unchanged?;
    record.producer_path = exporter_trust
        .path
        .to_str()
        .ok_or(NativeOriginError::PriorTrust)?
        .to_owned();
    Ok(PreparedRegistration {
        exporter_trust,
        record,
        observer_executable_sha256: local.observer_executable_sha256,
    })
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use ring::signature::KeyPair;
    use serde_json::{Value, json};

    use super::*;

    fn script(root: &Path, text: &str) -> PathBuf {
        let path = root.join("synthetic-attest-child.sh");
        fs::write(&path, format!("#!/bin/sh\n{text}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        path
    }

    // These fixtures exercise the private runner's mechanics only. Their
    // hashes are never catalog entries, and no fixture can register an issuer.
    #[tokio::test]
    async fn npr_09_child_mechanics() {
        let root = tempfile::tempdir().unwrap();
        let path = script(root.path(), "exec /usr/bin/head -c 4096 /dev/zero");
        let challenge = "0".repeat(64);
        let run = |duration| {
            run_child(
                &path,
                root.path(),
                root.path(),
                Path::new("/bin/sh"),
                &challenge,
                duration,
            )
        };
        assert_eq!(run(Duration::from_secs(2)).await.unwrap().len(), 4096);
        script(root.path(), "exec /usr/bin/head -c 4097 /dev/zero");
        let error = run(Duration::from_secs(2)).await.unwrap_err();
        assert_eq!(error.downcast_ref(), Some(&NativeOriginError::ChildBounds));
        script(root.path(), "printf '{}\\n'; exit 7");
        assert_eq!(
            run(Duration::from_secs(2))
                .await
                .unwrap_err()
                .downcast_ref(),
            Some(&NativeOriginError::Challenge)
        );
        script(
            root.path(),
            "printf 'private-child-secret\\033[31m' >&2; exit 1",
        );
        let error = run(Duration::from_secs(2)).await.unwrap_err().to_string();
        assert!(!error.contains("private-child-secret"));
        assert!(!error.contains('\u{1b}'));
        script(root.path(), "exec /usr/bin/head -c 65537 /dev/zero >&2");
        assert_eq!(
            run(Duration::from_secs(2))
                .await
                .unwrap_err()
                .downcast_ref(),
            Some(&NativeOriginError::ChildBounds)
        );
        for body in [
            "exec /bin/sleep 10",
            "printf '{}\\n'; exec /bin/sleep 10",
            "while :; do printf x; /bin/sleep 0.05; done",
        ] {
            script(root.path(), body);
            let start = std::time::Instant::now();
            assert_eq!(
                run(Duration::from_millis(150))
                    .await
                    .unwrap_err()
                    .downcast_ref(),
                Some(&NativeOriginError::ChildBounds)
            );
            assert!(
                start.elapsed() < Duration::from_secs(3),
                "timeout must include cleanup"
            );
        }
        script(
            root.path(),
            "test -z \"$NODE_OPTIONS$BUN_OPTIONS$LIBRA_D1_TOKEN$OPENCODE_CONFIG\" || exit 9; test \"$OPENCODE_LIBRA_NATIVE_ORIGIN\" = 1 || exit 8; printf '{}\\n'",
        );
        assert_eq!(run(Duration::from_secs(2)).await.unwrap(), b"{}\n");
    }

    #[tokio::test]
    async fn npr_09_cancel_reaps_child() {
        let root = tempfile::tempdir().unwrap();
        let path = script(
            root.path(),
            "echo $$ > \"$OPENCODE_LIBRA_NATIVE_KEY_DIR/pid\"; exec /bin/sleep 10",
        );
        let cwd = root.path().to_owned();
        let task = tokio::spawn(async move {
            run_child(
                &path,
                &cwd,
                &cwd,
                Path::new("/bin/sh"),
                &"0".repeat(64),
                Duration::from_secs(30),
            )
            .await
        });
        let pid_file = root.path().join("pid");
        let pid: i32 = tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                if let Ok(text) = fs::read_to_string(&pid_file)
                    && let Ok(pid) = text.trim().parse()
                {
                    break pid;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                // SAFETY: signal 0 only probes the specific fixture pid.
                if unsafe { libc::kill(pid, 0) } != 0 {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("cancelled fixture child must be reaped");
    }

    #[test]
    fn npr_09_owned_file_posture() {
        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let root = root.path().canonicalize().unwrap();
        let file = root.join("installation.json");
        fs::write(&file, b"{}").unwrap();
        fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(read_local_file(&file, 2, Some(0o600)).unwrap(), b"{}");
        assert!(read_local_file(&file, 1, Some(0o600)).is_err());
        fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(read_local_file(&file, 2, Some(0o600)).is_err());
        fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(read_local_file(&file, 2, Some(0o600)).is_err());
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let link = root.join("alias.json");
        std::os::unix::fs::symlink(&file, &link).unwrap();
        assert!(read_local_file(&link, 2, Some(0o600)).is_err());
    }

    #[test]
    fn npr_09_challenge_bindings() {
        // Synthetic public-wire predicate tests, not approved CLI enrollment
        // or native provider evidence. The real CLI tests separately consume
        // an independently implemented producer signature from actual bytes.
        let key = signature::Ed25519KeyPair::from_seed_unchecked(&[5u8; 32]).unwrap();
        let public = hex::encode(key.public_key().as_ref());
        let issuer = hex::encode(Sha256::digest(key.public_key().as_ref()));
        let challenge = "1".repeat(64);
        let build = approved_build::lookup(
            codec::PRODUCER_VERSION,
            "bac21cc4027896b8d1836133c347d20f",
            "547c36eb928da163ef61d019bad669860d519f2da0b67737c9a3920e831c9196",
        )
        .unwrap();
        let rendered = build
            .render_observer("/Applications/Libra.app/Contents/MacOS/libra")
            .unwrap();
        let policy = build
            .installation_policy(
                "/Applications/Libra.app/Contents/MacOS/libra",
                rendered.as_bytes(),
            )
            .unwrap();
        let local = LocalBinding {
            installation: Installation {
                version: 1,
                build_identity: policy.build_identity.clone(),
                producer_binary_digest:
                    "547c36eb928da163ef61d019bad669860d519f2da0b67737c9a3920e831c9196".into(),
                issuer_public_key: public.clone(),
                key_generation: 1,
                policy_digest: hex::encode(Sha256::digest(codec::policy_bytes(&policy).unwrap())),
                policy_descriptor: policy,
            },
            key_dir: "/synthetic/private".into(),
            command_path: "/synthetic/libra".into(),
            observer_executable_sha256: "0".repeat(64),
        };
        let original = json!({"version":1,"challenge":challenge,"issuer":issuer,"publicKey":public,"keyGeneration":1,
            "buildIdentity":local.installation.build_identity,"producerVersion":codec::PRODUCER_VERSION,
            "producerBinaryDigest":local.installation.producer_binary_digest,"policyDigest":local.installation.policy_digest,"signature":""});
        let signed = |mut wire: Value, domain: &[u8]| {
            let mut preimage = domain.to_vec();
            preimage.extend((wire["version"].as_u64().unwrap() as u32).to_be_bytes());
            for name in ["challenge", "issuer", "publicKey"] {
                preimage.extend(hex::decode(wire[name].as_str().unwrap()).unwrap());
            }
            preimage.extend((wire["keyGeneration"].as_u64().unwrap() as u32).to_be_bytes());
            preimage.extend(hex::decode(wire["buildIdentity"].as_str().unwrap()).unwrap());
            let version = wire["producerVersion"].as_str().unwrap().as_bytes();
            preimage.extend((version.len() as u32).to_be_bytes());
            preimage.extend(version);
            for name in ["producerBinaryDigest", "policyDigest"] {
                preimage.extend(hex::decode(wire[name].as_str().unwrap()).unwrap());
            }
            wire["signature"] = json!(STANDARD.encode(key.sign(&preimage).as_ref()));
            let mut bytes = serde_json::to_vec(&wire).unwrap();
            bytes.push(b'\n');
            bytes
        };
        let domain = b"libra-opencode-native-origin-attest-v1\0";
        let valid = signed(original.clone(), domain);
        assert!(verify_challenge(&valid, &challenge, &local).is_ok());
        for (field, value) in [
            ("challenge", json!("2".repeat(64))),
            ("version", json!(2)),
            ("keyGeneration", json!(2)),
            ("issuer", json!("3".repeat(64))),
            ("publicKey", json!("4".repeat(64))),
            ("buildIdentity", json!("5".repeat(32))),
            ("producerVersion", json!("2.0.26")),
            ("producerBinaryDigest", json!("6".repeat(64))),
            ("policyDigest", json!("7".repeat(64))),
        ] {
            let mut changed = original.clone();
            changed[field] = value;
            assert!(
                verify_challenge(&signed(changed, domain), &challenge, &local).is_err(),
                "even an authentic signature cannot change the requested installation binding"
            );
        }
        assert!(
            verify_challenge(
                &signed(
                    original.clone(),
                    b"libra-opencode-native-origin-receipt-v1\0"
                ),
                &challenge,
                &local
            )
            .is_err()
        );
        let mut bad: Value = serde_json::from_slice(&valid).unwrap();
        bad["signature"] = json!(STANDARD.encode([0u8; 64]));
        let mut bytes = serde_json::to_vec(&bad).unwrap();
        bytes.push(b'\n');
        assert!(verify_challenge(&bytes, &challenge, &local).is_err());
        bad["extra"] = json!(1);
        let mut bytes = serde_json::to_vec(&bad).unwrap();
        bytes.push(b'\n');
        assert!(verify_challenge(&bytes, &challenge, &local).is_err());
        let mut trailing = valid.clone();
        trailing.extend(b"{}\n");
        assert!(verify_challenge(&trailing, &challenge, &local).is_err());
        assert!(verify_challenge(&valid, &"8".repeat(64), &local).is_err());
    }
}
