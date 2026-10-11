//! Real macOS private-artifact enrollment. This target deliberately requires
//! test-live-agent and four explicit local paths; missing inputs FAIL. It never
//! contacts a model, installs globally, or promotes synthetic native fields.

#![cfg(target_os = "macos")]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use libra::internal::{
    ai::observed_agents::{native_origin::registry::REGISTRY_KEY, trust::compute_provenance},
    db::begin_write_transaction,
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};
use serde_json::{Value, json};

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    repo: PathBuf,
    home: PathBuf,
    producer: PathBuf,
    bun: PathBuf,
    source: PathBuf,
    recipe: PathBuf,
    libra: PathBuf,
}

fn required_path(name: &str) -> PathBuf {
    let path = PathBuf::from(std::env::var_os(name).unwrap_or_else(|| {
        panic!(
            "{name} must select the reviewed local artifact; this real gate must not silently skip"
        )
    }));
    assert!(path.is_absolute(), "{name} must be absolute");
    path.canonicalize().unwrap()
}

fn describe(out: &Output) -> String {
    // These isolated test children have no credentials or provider requests.
    format!(
        "status={} stdout={} stderr={}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

struct ResumeAttestChild(i32);

impl Drop for ResumeAttestChild {
    fn drop(&mut self) {
        // SAFETY: this is the specific still-stopped private attestation child
        // of the fixture's owned Libra process, identified by its actual PPID.
        unsafe {
            libc::kill(self.0, libc::SIGCONT);
        }
    }
}

async fn stop_actual_attest_child(parent: u32, producer: &Path) -> ResumeAttestChild {
    let pid = tokio::time::timeout(std::time::Duration::from_secs(8), async {
        loop {
            let out = Command::new("/bin/ps")
                .args(["-axo", "pid=,ppid="])
                .env_clear()
                .output()
                .unwrap();
            assert!(out.status.success());
            for line in String::from_utf8(out.stdout).unwrap().lines() {
                let mut fields = line.split_whitespace();
                let Some(pid) = fields.next().and_then(|v| v.parse::<i32>().ok()) else {
                    continue;
                };
                let Some(ppid) = fields.next().and_then(|v| v.parse::<u32>().ok()) else {
                    continue;
                };
                if ppid != parent {
                    continue;
                }
                let args = Command::new("/bin/ps")
                    .args(["-p", &pid.to_string(), "-o", "args="])
                    .env_clear()
                    .output()
                    .unwrap();
                let text = String::from_utf8(args.stdout).unwrap();
                if text.contains(producer.to_str().unwrap())
                    && text.contains("native-origin attest --challenge")
                {
                    // SAFETY: positive pid came from the exact fixture parent
                    // and actual approved producer argv, never a global CLI.
                    if unsafe { libc::kill(pid, libc::SIGSTOP) } == 0 {
                        return pid;
                    }
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("the actual challenge must start before testing the untrust winner");
    ResumeAttestChild(pid)
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let repo = root.join("repo");
        let home = root.join("home");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&home).unwrap();
        let this = Self {
            _temp: temp,
            root,
            repo,
            home,
            producer: required_path("LIBRA_NATIVE_TEST_PRODUCER"),
            bun: required_path("LIBRA_NATIVE_TEST_BUN"),
            source: required_path("LIBRA_NATIVE_TEST_SOURCE"),
            recipe: required_path("LIBRA_NATIVE_TEST_RECIPE"),
            libra: Path::new(env!("CARGO_BIN_EXE_libra"))
                .canonicalize()
                .unwrap(),
        };
        assert_eq!(
            compute_provenance(&this.producer).unwrap().sha256,
            "547c36eb928da163ef61d019bad669860d519f2da0b67737c9a3920e831c9196"
        );
        this.ok(&["init"]);
        this.ok(&["agent", "enable", "--agent", "opencode"]);
        this
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(&self.libra);
        command
            .args(args)
            .current_dir(&self.repo)
            .env_clear()
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .env("HOME", &self.home)
            .env("LIBRA_TEST_HOME", &self.home)
            .env("LIBRA_TEST", "1");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }
    fn ok(&self, args: &[&str]) -> Output {
        let out = self.run(args);
        assert!(out.status.success(), "{}", describe(&out));
        out
    }
    fn trust(&self) {
        self.ok(&[
            "agent",
            "rpc",
            "trust",
            "--dir",
            self.producer.parent().unwrap().to_str().unwrap(),
        ]);
        self.ok(&["agent", "rpc", "trust", "opencode"]);
    }
    fn install(&self, name: &str, command_path: &Path) -> PathBuf {
        let dir = self.root.join(name);
        let out = Command::new(&self.bun)
            .args([
                "--no-env-file",
                "--no-install",
                "run",
                "script/native-origin-init.ts",
                "--key-dir",
            ])
            .arg(&dir)
            .arg("--producer")
            .arg(&self.producer)
            .arg("--libra-command")
            .arg(command_path)
            .arg("--recipe")
            .arg(&self.recipe)
            .current_dir(self.source.join("packages/cli"))
            .env_clear()
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .env("HOME", &self.home)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "private initialization: {}",
            describe(&out)
        );
        let manifest = dir.join("installation.json");
        assert_eq!(
            fs::metadata(&dir).unwrap().permissions().mode() & 0o7777,
            0o700
        );
        assert_eq!(
            fs::metadata(&manifest).unwrap().permissions().mode() & 0o7777,
            0o600
        );
        manifest
    }
    fn manifest(&self, name: &str) -> PathBuf {
        self.install(name, &self.libra)
    }
    fn registration(&self, manifest: &Path, epoch: u64) -> Output {
        self.run(&[
            "agent",
            "rpc",
            "trust",
            "opencode",
            "--native-origin-manifest",
            manifest.to_str().unwrap(),
            "--native-origin-expected-epoch",
            &epoch.to_string(),
            "--json",
        ])
    }
    async fn db(&self) -> DatabaseConnection {
        Database::connect(format!(
            "sqlite:{}?mode=rw",
            self.repo.join(".libra/libra.db").display()
        ))
        .await
        .unwrap()
    }
    async fn rows(&self) -> Vec<(String, String, i32)> {
        self.db()
            .await
            .query_all_raw(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT key,value,encrypted FROM config_kv ORDER BY id",
            ))
            .await
            .unwrap()
            .iter()
            .map(|row| {
                (
                    row.try_get("", "key").unwrap(),
                    row.try_get("", "value").unwrap(),
                    row.try_get("", "encrypted").unwrap(),
                )
            })
            .collect()
    }
    async fn registry(&self) -> Option<Value> {
        self.rows()
            .await
            .into_iter()
            .find(|(key, _, _)| key == REGISTRY_KEY)
            .map(|(_, value, encrypted)| {
                assert_eq!(encrypted, 0);
                serde_json::from_str(&value).unwrap()
            })
    }
    async fn assert_refused_unchanged(&self, manifest: &Path, epoch: u64) -> Output {
        let before = self.rows().await;
        let out = self.registration(manifest, epoch);
        assert!(!out.status.success(), "registration unexpectedly succeeded");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("LBR-AGENT-005"),
            "{}",
            describe(&out)
        );
        assert_eq!(
            self.rows().await,
            before,
            "refusal must leave every config row unchanged"
        );
        out
    }
}

#[tokio::test]
async fn npr_09_explicit() {
    let repo = Fixture::new();
    repo.trust();
    assert!(
        repo.registry().await.is_none(),
        "ordinary exporter trust grants no issuer"
    );
    let manifest = repo.manifest("explicit");
    for args in [
        vec![
            "agent",
            "rpc",
            "trust",
            "opencode",
            "--native-origin-manifest",
            manifest.to_str().unwrap(),
        ],
        vec![
            "agent",
            "rpc",
            "trust",
            "opencode",
            "--native-origin-expected-epoch",
            "0",
        ],
        vec![
            "agent",
            "rpc",
            "trust",
            "claude",
            "--native-origin-manifest",
            manifest.to_str().unwrap(),
            "--native-origin-expected-epoch",
            "0",
        ],
        vec![
            "agent",
            "rpc",
            "trust",
            "--dir",
            repo.root.to_str().unwrap(),
            "--native-origin-manifest",
            manifest.to_str().unwrap(),
            "--native-origin-expected-epoch",
            "0",
        ],
    ] {
        let before = repo.rows().await;
        assert!(!repo.run(&args).status.success());
        assert_eq!(repo.rows().await, before);
    }
    let out = repo.registration(&manifest, 0);
    assert!(out.status.success(), "{}", describe(&out));
    assert_eq!(repo.registry().await.unwrap()["epoch"], 1);
}

#[tokio::test]
async fn npr_09_prior_trust() {
    let repo = Fixture::new();
    let manifest = repo.manifest("prior");
    repo.assert_refused_unchanged(&manifest, 0).await;
    repo.trust();
    let before = repo.rows().await;
    let trusted = before
        .iter()
        .find(|(key, _, _)| key == "agent.trust.opencode")
        .unwrap()
        .1
        .clone();
    let db = repo.db().await;
    let original: Value = serde_json::from_str(&trusted).unwrap();
    for (field, value) in [
        ("sha256", json!("0".repeat(64))),
        ("device", json!(0)),
        ("inode", json!(0)),
        ("mtime", json!(0)),
        ("path", json!("/missing/opencode")),
    ] {
        let mut changed = original.clone();
        changed[field] = value;
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE config_kv SET value=? WHERE key='agent.trust.opencode'",
            [changed.to_string().into()],
        ))
        .await
        .unwrap();
        repo.assert_refused_unchanged(&manifest, 0).await;
    }
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Sqlite,
        "UPDATE config_kv SET value=? WHERE key='agent.trust.opencode'",
        [trusted.into()],
    ))
    .await
    .unwrap();
    db.execute_unprepared(
        "UPDATE config_kv SET value='[]' WHERE key='agent.external_agents.trusted_dirs'",
    )
    .await
    .unwrap();
    repo.assert_refused_unchanged(&manifest, 0).await;
    repo.trust();
    assert!(repo.registration(&manifest, 0).status.success());
}

#[tokio::test]
async fn npr_09_approved() {
    let mut repo = Fixture::new();
    repo.trust();
    let manifest = repo.manifest("approved");
    let original = fs::read(&manifest).unwrap();
    let descriptor: Value = serde_json::from_slice(&original).unwrap();
    for (field, value) in [
        ("buildIdentity", json!("0".repeat(32))),
        ("producerBinaryDigest", json!("0".repeat(64))),
        ("version", json!(2)),
        ("issuerPublicKey", json!("F".repeat(64))),
        ("policyDigest", json!("0".repeat(64))),
    ] {
        let mut changed = descriptor.clone();
        changed[field] = value;
        fs::write(&manifest, changed.to_string()).unwrap();
        repo.assert_refused_unchanged(&manifest, 0).await;
    }
    for field in [
        "libraCommandPath",
        "libraRenderedDigest",
        "eventMapDigest",
        "libraTemplateDigest",
        "builtinSetDigest",
    ] {
        let mut changed = descriptor.clone();
        changed["policyDescriptor"][field] = if field == "libraCommandPath" {
            json!("/Applications/Forged/libra")
        } else {
            json!("0".repeat(64))
        };
        fs::write(&manifest, changed.to_string()).unwrap();
        repo.assert_refused_unchanged(&manifest, 0).await;
    }
    fs::write(&manifest, &original).unwrap();
    fs::set_permissions(&manifest, fs::Permissions::from_mode(0o644)).unwrap();
    repo.assert_refused_unchanged(&manifest, 0).await;
    fs::set_permissions(&manifest, fs::Permissions::from_mode(0o600)).unwrap();
    let plugin = repo.repo.join(".opencode/plugin/libra-hooks.js");
    let original_plugin = fs::read(&plugin).unwrap();
    let mut changed = original_plugin.clone();
    changed.extend(b"\n// forged marker is not source approval\n");
    fs::write(&plugin, changed).unwrap();
    repo.assert_refused_unchanged(&manifest, 0).await;
    fs::write(&plugin, &original_plugin).unwrap();
    let out = repo.registration(&manifest, 0);
    assert!(out.status.success(), "{}", describe(&out));
    assert_eq!(
        fs::read(&manifest).unwrap(),
        original,
        "registration must not rewrite installation data"
    );
    let registry = repo.registry().await.unwrap();
    assert_eq!(
        registry["issuers"][0]["policyDescriptor"],
        descriptor["policyDescriptor"]
    );
    // A second actual canonical executable path binds a different private
    // policy; its bytes are the same reviewed current consumer, never a new
    // producer catalog entry or a manifest-authorized executable.
    let alternate = repo.root.join("alternate-libra");
    fs::copy(&repo.libra, &alternate).unwrap();
    fs::set_permissions(&alternate, fs::Permissions::from_mode(0o755)).unwrap();
    repo.libra = alternate.canonicalize().unwrap();
    repo.ok(&["agent", "enable", "--agent", "opencode"]);
    let second = repo.manifest("approved-second-path");
    let out = repo.registration(&second, 1);
    assert!(
        out.status.success(),
        "second actual binding: {}",
        describe(&out)
    );
    let registry = repo.registry().await.unwrap();
    assert_ne!(
        registry["issuers"][0]["policyDigest"],
        registry["issuers"][1]["policyDigest"]
    );
    assert_eq!(
        registry["issuers"][1]["policyDescriptor"]["libraCommandPath"],
        repo.libra.to_str().unwrap()
    );
}

#[tokio::test]
async fn npr_09_challenge() {
    let repo = Fixture::new();
    repo.trust();
    let manifest = repo.manifest("challenge");
    let before = fs::read(&manifest).unwrap();
    let out = repo.registration(&manifest, 0);
    assert!(
        out.status.success(),
        "actual reviewed producer signature must verify: {}",
        describe(&out)
    );
    let first = repo.registry().await.unwrap();
    assert_eq!(first["epoch"], 1);
    assert_eq!(first["issuers"][0]["state"], "active");
    repo.assert_refused_unchanged(&manifest, 1).await; // Fresh challenge cannot reactivate the same key.
    assert_eq!(fs::read(&manifest).unwrap(), before);
    let second = repo.manifest("new-key");
    let out = repo.registration(&second, 1);
    assert!(out.status.success(), "{}", describe(&out));
    let registry = repo.registry().await.unwrap();
    assert_eq!(registry["epoch"], 2);
    assert_ne!(
        registry["issuers"][0]["issuer"],
        registry["issuers"][1]["issuer"]
    );
    assert_eq!(registry["issuers"][0]["state"], "verify-only");
    assert_eq!(registry["issuers"][1]["state"], "active");
}

#[tokio::test]
async fn npr_09_child_bounds() {
    let repo = Fixture::new();
    repo.trust();
    let manifest = repo.manifest("bounds");
    // CLI refuses an altered/unapproved binary before it can be run. The
    // runner's exact 4096/4097, timeout/drip/stderr/kill/reap cases are private
    // unit mechanics, never fake catalog enrollment.
    let fake_dir = repo.root.join("unapproved");
    fs::create_dir(&fake_dir).unwrap();
    let fake = fake_dir.join("opencode");
    fs::write(
        &fake,
        "#!/bin/sh\necho spawned > \"$OPENCODE_LIBRA_NATIVE_KEY_DIR/spawned\"\nsleep 60\n",
    )
    .unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o700)).unwrap();
    let measured = compute_provenance(&fake).unwrap();
    let trust = json!({"path":measured.canonical_path,"sha256":measured.sha256,"device":measured.device,"inode":measured.inode,"mtime":measured.mtime});
    repo.db()
        .await
        .execute_raw(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE config_kv SET value=? WHERE key='agent.trust.opencode'",
            [trust.to_string().into()],
        ))
        .await
        .unwrap();
    repo.ok(&["agent", "rpc", "trust", "--dir", fake_dir.to_str().unwrap()]);
    repo.assert_refused_unchanged(&manifest, 0).await;
    assert!(!manifest.parent().unwrap().join("spawned").exists());
}

#[tokio::test]
async fn npr_09_atomic() {
    let repo = Fixture::new();
    repo.trust();
    let manifest = repo.manifest("atomic");
    let db = repo.db().await;
    db.execute_unprepared(
        "INSERT INTO config_kv(key,value,encrypted) VALUES('canary.native','preserve',0)",
    )
    .await
    .unwrap();
    db.execute_unprepared("CREATE TRIGGER reject_native_insert BEFORE INSERT ON config_kv WHEN NEW.key='agent.native_origin.opencode.v1' BEGIN SELECT RAISE(ABORT,'synthetic write fault'); END").await.unwrap();
    repo.assert_refused_unchanged(&manifest, 0).await;
    db.execute_unprepared("DROP TRIGGER reject_native_insert")
        .await
        .unwrap();
    let before = repo.rows().await;
    let args = [
        "agent",
        "rpc",
        "trust",
        "opencode",
        "--native-origin-manifest",
        manifest.to_str().unwrap(),
        "--native-origin-expected-epoch",
        "0",
    ];
    let mut first = repo.command(&args);
    let mut second = repo.command(&args);
    first
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    second
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child1 = first.spawn().unwrap();
    let child2 = second.spawn().unwrap();
    let out1 = child1.wait_with_output().unwrap();
    let out2 = child2.wait_with_output().unwrap();
    assert_eq!(
        usize::from(out1.status.success()) + usize::from(out2.status.success()),
        1,
        "one CAS winner: {} / {}",
        describe(&out1),
        describe(&out2)
    );
    let after = repo.rows().await;
    assert_eq!(
        after
            .iter()
            .filter(|(key, _, _)| key == REGISTRY_KEY)
            .count(),
        1
    );
    assert_eq!(
        after
            .into_iter()
            .filter(|(key, _, _)| key != REGISTRY_KEY)
            .collect::<Vec<_>>(),
        before
    );
    assert_eq!(repo.registry().await.unwrap()["epoch"], 1);
    // Freeze only this fixture's actual approved attestation child, acquire
    // the DB writer, then let the real challenge complete while registration
    // cannot publish. The owner port removes trust on that same connection
    // after the child has exited. There is no fake producer or runtime bypass.
    let before_untrust = repo.rows().await;
    let second_manifest = repo.manifest("post-handshake-untrust");
    let args = [
        "agent",
        "rpc",
        "trust",
        "opencode",
        "--native-origin-manifest",
        second_manifest.to_str().unwrap(),
        "--native-origin-expected-epoch",
        "1",
    ];
    let mut command = repo.command(&args);
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = command.spawn().unwrap();
    let stopped = stop_actual_attest_child(child.id(), &repo.producer).await;
    let pid = stopped.0;
    let txn = begin_write_transaction(&db).await.unwrap();
    drop(stopped);
    tokio::time::timeout(std::time::Duration::from_secs(8), async {
        loop {
            // SAFETY: signal 0 only probes the specific fixture child.
            if unsafe { libc::kill(pid, 0) } != 0 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("actual approved challenge child must exit while the writer is held");
    assert!(
        libra::internal::ai::observed_agents::trust::revoke_trust_with_conn(&txn, "opencode")
            .await
            .unwrap()
    );
    txn.commit().await.unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(!out.status.success(), "post-handshake untrust must win");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("exporter trust is missing or changed"),
        "{}",
        describe(&out)
    );
    assert_eq!(
        repo.rows().await,
        before_untrust
            .into_iter()
            .filter(|(key, _, _)| key != "agent.trust.opencode")
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn npr_09_forged_export() {
    let repo = Fixture::new();
    repo.trust();
    let manifest = repo.manifest("forged");
    let mut forged: Value = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    forged["pubkey"] = json!("attacker supplied");
    fs::write(&manifest, forged.to_string()).unwrap();
    repo.assert_refused_unchanged(&manifest, 0).await;
    repo.ok(&["agent", "rpc", "trust", "opencode"]);
    assert!(
        repo.registry().await.is_none(),
        "ordinary trust never discovers a descriptor/public key"
    );
    let before = repo.rows().await;
    let stdin = json!({"pubkey":"attacker supplied","manifest":manifest,"nativeOrigin":forged});
    assert!(
        !repo
            .run(&[
                "agent",
                "rpc",
                "trust",
                "opencode",
                "--native-origin-manifest",
                &stdin.to_string(),
                "--native-origin-expected-epoch",
                "0"
            ])
            .status
            .success()
    );
    assert_eq!(repo.rows().await, before);
}

#[tokio::test]
async fn npr_09_single_row() {
    let repo = Fixture::new();
    repo.trust();
    let manifest = repo.manifest("rows");
    let db = repo.db().await;
    for keys in [
        vec![REGISTRY_KEY, REGISTRY_KEY],
        vec![REGISTRY_KEY, "AGENT.NATIVE_ORIGIN.OPENCODE.V1"],
        vec!["Agent.Native_Origin.Opencode.V1"],
        vec![REGISTRY_KEY],
    ] {
        db.execute_unprepared(
            "DELETE FROM config_kv WHERE lower(key)=lower('agent.native_origin.opencode.v1')",
        )
        .await
        .unwrap();
        for key in &keys {
            db.execute_raw(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "INSERT INTO config_kv(key,value,encrypted) VALUES(?,?,0)",
                [(*key).into(), "{}".into()],
            ))
            .await
            .unwrap();
        }
        repo.assert_refused_unchanged(&manifest, 0).await;
    }
    db.execute_unprepared(
        "DELETE FROM config_kv WHERE lower(key)=lower('agent.native_origin.opencode.v1')",
    )
    .await
    .unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Sqlite,
        "INSERT INTO config_kv(key,value,encrypted) VALUES(?,?,0)",
        [REGISTRY_KEY.into(), "x".repeat(65537).into()],
    ))
    .await
    .unwrap();
    repo.assert_refused_unchanged(&manifest, 0).await;
}
