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
    ai::observed_agents::{
        native_origin::registry::{REGISTRY_BYTE_CAP, REGISTRY_KEY, active_issuer_with_conn},
        trust::{self, compute_provenance},
    },
    db::begin_write_transaction,
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

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
        this.install_legacy_observer();
        this
    }

    fn install_legacy_observer(&self) {
        // This local enrollment gate never starts OpenCode. Keep its old
        // producer fixture independent from the current ordinary installer.
        let recipe: Value = serde_json::from_slice(&fs::read(&self.recipe).unwrap()).unwrap();
        assert_eq!(recipe["buildIdentity"], "bac21cc4027896b8d1836133c347d20f");
        let template = recipe["libraTemplate"].as_str().unwrap();
        assert_eq!(template.len(), 13600);
        assert_eq!(
            hex::encode(Sha256::digest(template.as_bytes())),
            "9c67213be10e16538bfa550501a2dcb7a21c24493b3a8f05592f6c96ff4be27d"
        );
        let events = recipe["eventCommandsJson"].as_str().unwrap();
        assert_eq!(
            hex::encode(Sha256::digest(events.as_bytes())),
            "85977fd993350c87dde2cd8cafc4473d7f86309f362a397def7cd16efbcd5dc4"
        );
        assert_eq!(recipe["exportDeadlineMillis"], 30000);
        let observer = template
            .replace(
                "__LIBRA_COMMAND_JSON__",
                &serde_json::to_string(self.libra.to_str().unwrap()).unwrap(),
            )
            .replace("__EVENT_COMMANDS_JSON__", events)
            .replace("__EXPORT_DEADLINE_MS__", "30000");
        let plugin = self.repo.join(".opencode/plugin/libra-hooks.js");
        fs::create_dir_all(plugin.parent().unwrap()).unwrap();
        fs::write(&plugin, observer).unwrap();
        fs::set_permissions(&plugin, fs::Permissions::from_mode(0o644)).unwrap();
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

    async fn enroll(&self, name: &str) -> (PathBuf, String) {
        let manifest = self.manifest(name);
        let epoch = self
            .registry()
            .await
            .map_or(0, |r| r["epoch"].as_u64().unwrap());
        let out = self.registration(&manifest, epoch);
        assert!(out.status.success(), "{}", describe(&out));
        let installation: Value = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
        let key = hex::decode(installation["issuerPublicKey"].as_str().unwrap()).unwrap();
        let issuer = hex::encode(Sha256::digest(key));
        let result: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(result["data"]["issuer"], issuer);
        (manifest, issuer)
    }

    async fn admitted(&self, issuer: &str) -> bool {
        // This reads current permission data only. It does not verify a
        // receipt, admit a native field, or construct an opaque artifact.
        let measured = compute_provenance(&self.producer).unwrap();
        let db = self.db().await;
        let txn = begin_write_transaction(&db).await.unwrap();
        let allowed = active_issuer_with_conn(&txn, issuer, &measured)
            .await
            .unwrap()
            .is_some();
        txn.rollback().await.unwrap();
        allowed
    }

    fn untrust(&self) -> Value {
        let out = self.ok(&["agent", "rpc", "untrust", "opencode", "--json"]);
        let envelope: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(envelope["command"], "agent_rpc_untrust");
        envelope["data"].clone()
    }
}

#[tokio::test]
async fn npr_10_rotation() {
    let repo = Fixture::new();
    repo.trust();
    let (first_manifest, first) = repo.enroll("old").await;
    let before = fs::read(&first_manifest).unwrap();
    let (_, second) = repo.enroll("new").await;
    assert_ne!(first, second);
    let registry = repo.registry().await.unwrap();
    assert_eq!(registry["epoch"], 2);
    assert_eq!(registry["issuers"][0]["state"], "verify-only");
    assert_eq!(registry["issuers"][1]["state"], "active");
    assert_eq!(registry["issuers"][0]["issuer"], first);
    assert_eq!(registry["issuers"][1]["issuer"], second);
    assert_eq!(fs::read(first_manifest).unwrap(), before);
    assert!(!repo.admitted(&first).await);
    assert!(repo.admitted(&second).await);
}

#[tokio::test]
async fn npr_10_new_admission() {
    let repo = Fixture::new();
    repo.trust();
    let (_, first) = repo.enroll("first").await;
    let (_, second) = repo.enroll("second").await;
    assert!(!repo.admitted(&first).await);
    assert!(!repo.admitted(&"0".repeat(64)).await);
    let db = repo.db().await;
    let measured = compute_provenance(&repo.producer).unwrap();
    for field in 0..5 {
        let mut stale = measured.clone();
        match field {
            0 => stale.sha256 = "0".repeat(64),
            1 => stale.canonical_path = repo.root.join("different-producer"),
            2 => stale.device += 1,
            3 => stale.inode += 1,
            _ => stale.mtime += 1,
        }
        let txn = begin_write_transaction(&db).await.unwrap();
        assert!(
            active_issuer_with_conn(&txn, &second, &stale)
                .await
                .unwrap()
                .is_none()
        );
        txn.rollback().await.unwrap();
    }
    assert!(repo.admitted(&second).await);
    repo.untrust();
    assert!(!repo.admitted(&first).await);
    assert!(!repo.admitted(&second).await);
}

#[tokio::test]
async fn npr_10_untrust() {
    let repo = Fixture::new();
    // Revocation remains available while the external RPC surface is disabled.
    let empty = repo.untrust();
    assert_eq!(empty["removed"], false);
    assert_eq!(empty["native_revoked"], 0);
    assert!(empty["native_epoch"].is_null());
    repo.trust();
    repo.enroll("one").await;
    repo.enroll("two").await;
    let local = repo.ok(&["config", "--get", REGISTRY_KEY]);
    let inspected: Value = serde_json::from_slice(&local.stdout).unwrap();
    assert_eq!(inspected, repo.registry().await.unwrap());
    let data = repo.untrust();
    assert_eq!(data["removed"], true);
    assert_eq!(data["native_revoked"], 2);
    assert_eq!(data["native_epoch"], 3);
    assert!(
        repo.rows()
            .await
            .iter()
            .all(|(key, _, _)| key != "agent.trust.opencode")
    );
    let registry = repo.registry().await.unwrap();
    assert_eq!(registry["issuers"].as_array().unwrap().len(), 2);
    assert!(
        registry["issuers"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["state"] == "revoked")
    );
    let repeat = repo.untrust();
    assert_eq!(repeat["removed"], false);
    assert_eq!(repeat["native_revoked"], 0);
    assert_eq!(repeat["native_epoch"], 3);
    assert_eq!(repo.registry().await.unwrap(), registry);
}

#[tokio::test]
async fn npr_10_no_resurrection() {
    let repo = Fixture::new();
    repo.trust();
    let (old_manifest, old) = repo.enroll("old").await;
    repo.untrust();
    let tombstone = repo.registry().await.unwrap();
    repo.trust();
    assert_eq!(repo.registry().await.unwrap(), tombstone);
    assert!(!repo.admitted(&old).await);
    repo.assert_refused_unchanged(&old_manifest, 2).await;
    let (_, fresh) = repo.enroll("fresh").await;
    assert_ne!(fresh, old);
    assert!(!repo.admitted(&old).await);
    assert!(repo.admitted(&fresh).await);
    let registry = repo.registry().await.unwrap();
    assert_eq!(registry["issuers"][0]["state"], "revoked");
    assert_eq!(registry["issuers"][0]["issuer"], old);
    assert_eq!(registry["epoch"], 3);
}

#[tokio::test]
async fn npr_10_concurrency() {
    let repo = Fixture::new();
    repo.trust();
    repo.enroll("initial").await;
    let db = repo.db().await;
    db.execute_unprepared(
        "INSERT INTO config_kv(key,value,encrypted) VALUES('canary.lifecycle','preserve',0)",
    )
    .await
    .unwrap();
    let before = repo.rows().await;
    db.execute_unprepared("CREATE TRIGGER reject_lifecycle_update BEFORE UPDATE ON config_kv WHEN NEW.key='agent.native_origin.opencode.v1' BEGIN SELECT RAISE(ABORT,'synthetic lifecycle write fault'); END").await.unwrap();
    let out = repo.run(&["agent", "rpc", "untrust", "opencode"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("LBR-AGENT-005"));
    assert_eq!(
        repo.rows().await,
        before,
        "native write failure must also roll back binary deletion"
    );
    db.execute_unprepared("DROP TRIGGER reject_lifecycle_update")
        .await
        .unwrap();
    let manifest = repo.manifest("concurrent");
    let mut register = repo.command(&[
        "agent",
        "rpc",
        "trust",
        "opencode",
        "--native-origin-manifest",
        manifest.to_str().unwrap(),
        "--native-origin-expected-epoch",
        "1",
    ]);
    let mut untrust = repo.command(&["agent", "rpc", "untrust", "opencode"]);
    register
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    untrust
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let register = register.spawn().unwrap();
    let untrust = untrust.spawn().unwrap();
    let registration = register.wait_with_output().unwrap();
    let revocation = untrust.wait_with_output().unwrap();
    assert!(revocation.status.success(), "{}", describe(&revocation));
    let registry = repo.registry().await.unwrap();
    let entries = registry["issuers"].as_array().unwrap();
    assert!(entries.iter().all(|r| r["state"] == "revoked"));
    assert_eq!(
        entries.len(),
        if registration.status.success() { 2 } else { 1 }
    );
    assert_eq!(
        registry["epoch"],
        if registration.status.success() { 3 } else { 2 }
    );
    assert!(
        repo.rows()
            .await
            .iter()
            .all(|(key, _, _)| key != "agent.trust.opencode")
    );
    assert!(
        repo.rows()
            .await
            .iter()
            .any(|(key, value, _)| key == "canary.lifecycle" && value == "preserve")
    );
}

#[tokio::test]
async fn npr_10_cap() {
    let repo = Fixture::new();
    repo.trust();
    repo.enroll("old").await;
    repo.untrust();
    repo.trust();
    let tombstone = repo.registry().await.unwrap();
    let db = repo.db().await;
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Sqlite,
        "UPDATE config_kv SET value=? WHERE key=?",
        [
            "x".repeat(REGISTRY_BYTE_CAP + 1).into(),
            REGISTRY_KEY.into(),
        ],
    ))
    .await
    .unwrap();
    let before = repo.rows().await;
    let manifest = repo.manifest("cannot-fit");
    repo.assert_refused_unchanged(&manifest, 2).await;
    assert!(
        !repo
            .run(&["agent", "rpc", "untrust", "opencode"])
            .status
            .success()
    );
    assert_eq!(repo.rows().await, before);
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Sqlite,
        "UPDATE config_kv SET value=? WHERE key=?",
        [tombstone.to_string().into(), REGISTRY_KEY.into()],
    ))
    .await
    .unwrap();
    assert_eq!(
        repo.registry().await.unwrap(),
        tombstone,
        "cap refusal never deletes tombstones"
    );
    let restored = repo.rows().await;
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Sqlite,
        "INSERT INTO config_kv(key,value,encrypted) VALUES(?,?,0)",
        [
            "AGENT.NATIVE_ORIGIN.OPENCODE.V1".into(),
            tombstone.to_string().into(),
        ],
    ))
    .await
    .unwrap();
    let aliases = repo.rows().await;
    assert!(
        !repo
            .run(&["agent", "rpc", "untrust", "opencode"])
            .status
            .success()
    );
    assert_eq!(repo.rows().await, aliases);
    assert_eq!(aliases.len(), restored.len() + 1);
    // Registry damage must not prevent the documented binary-only emergency
    // quarantine. This is not permanent native revocation: retain every row.
    repo.ok(&["config", "unset", "agent.trust.opencode"]);
    let expected: Vec<_> = aliases
        .into_iter()
        .filter(|(key, _, _)| key != "agent.trust.opencode")
        .collect();
    assert_eq!(repo.rows().await, expected);
}

#[tokio::test]
async fn npr_10_immutable() {
    use git_internal::internal::object::{
        ObjectTrait,
        tree::{Tree, TreeItem, TreeItemMode},
    };
    use libra::utils::object::{read_git_object, write_git_object};
    let repo = Fixture::new();
    repo.trust();
    repo.enroll("retention").await;
    let storage = repo.repo.join(".libra");
    // Persisted synthetic legacy artifact, not a native-field positive. Use
    // actual content-addressed blob/tree storage and the canonical reasoning
    // manifest/layout; keep it reachable by an isolated fixture tag.
    let body = b"synthetic-legacy-opaque-artifact-canary";
    let sha = hex::encode(Sha256::digest(body));
    let artifact = write_git_object(&storage, "blob", body).unwrap();
    let encrypted = Tree::from_tree_items(vec![TreeItem::new(
        TreeItemMode::Blob,
        artifact,
        sha.clone(),
    )])
    .unwrap();
    let encrypted_oid = write_git_object(&storage, "tree", &encrypted.to_data().unwrap()).unwrap();
    let reasoning = Tree::from_tree_items(vec![TreeItem::new(
        TreeItemMode::Tree,
        encrypted_oid,
        "encrypted".into(),
    )])
    .unwrap();
    let reasoning_oid = write_git_object(&storage, "tree", &reasoning.to_data().unwrap()).unwrap();
    let ordinary_metadata = write_git_object(
        &storage,
        "blob",
        br#"{"checkpoint_id":"npr10-retention","agent_kind":"claude_code","created_at":0}"#,
    )
    .unwrap();
    let transcript = write_git_object(
        &storage,
        "blob",
        b"{\"role\":\"user\",\"content\":\"synthetic retention fixture\"}\n",
    )
    .unwrap();
    let transcript_tree = Tree::from_tree_items(vec![TreeItem::new(
        TreeItemMode::Blob,
        transcript,
        "claude_code.jsonl".into(),
    )])
    .unwrap();
    let transcript_tree_oid =
        write_git_object(&storage, "tree", &transcript_tree.to_data().unwrap()).unwrap();
    let metadata = json!({"schema_version":1,"entries":{
        "metadata":{"path":"metadata.json","oid":ordinary_metadata.to_string()},
        "transcript":{"path":"transcript/claude_code.jsonl","oid":transcript.to_string()}},"reasoning_artifacts":[{
        "path":format!("reasoning/encrypted/{sha}"),"oid":artifact.to_string(),"sha256":sha,"byte_len":body.len(),
        "locator":"claude_code:msg=0/part=0/metadata=signature","provider":"claude_code","source_kind":"signature",
        "availability":"encrypted_unavailable","decrypt_capability":"none"}]});
    let original = serde_json::to_vec(&metadata).unwrap();
    let manifest = write_git_object(&storage, "blob", &original).unwrap();
    let tree = Tree::from_tree_items(vec![
        TreeItem::new(TreeItemMode::Blob, manifest, "manifest.json".into()),
        TreeItem::new(
            TreeItemMode::Blob,
            ordinary_metadata,
            "metadata.json".into(),
        ),
        TreeItem::new(TreeItemMode::Tree, reasoning_oid, "reasoning".into()),
        TreeItem::new(TreeItemMode::Tree, transcript_tree_oid, "transcript".into()),
    ])
    .unwrap();
    let tree_oid = write_git_object(&storage, "tree", &tree.to_data().unwrap()).unwrap();
    // The supported tag command requires a commit target. Wrap the persisted
    // synthetic checkpoint tree in a commit instead of bypassing ref rules.
    let commit_data = format!(
        "tree {tree_oid}\nauthor NPR10 Fixture <npr10@example.invalid> 0 +0000\ncommitter NPR10 Fixture <npr10@example.invalid> 0 +0000\n\nsynthetic retention fixture\n"
    );
    let commit_oid = write_git_object(&storage, "commit", commit_data.as_bytes()).unwrap();
    repo.ok(&["tag", "npr10-artifact-retention", &commit_oid.to_string()]);
    let mut objects = Vec::new();
    for oid in [
        artifact,
        manifest,
        encrypted_oid,
        reasoning_oid,
        ordinary_metadata,
        transcript,
        transcript_tree_oid,
        tree_oid,
        commit_oid,
    ] {
        objects.push((oid, read_git_object(&storage, &oid).unwrap()));
    }
    let tag = repo
        .ok(&["rev-parse", "refs/tags/npr10-artifact-retention"])
        .stdout;
    repo.untrust();
    repo.trust();
    for (oid, before) in objects {
        assert_eq!(read_git_object(&storage, &oid).unwrap(), before);
    }
    assert_eq!(read_git_object(&storage, &artifact).unwrap(), body);
    assert_eq!(read_git_object(&storage, &manifest).unwrap(), original);
    assert_eq!(
        repo.ok(&["rev-parse", "refs/tags/npr10-artifact-retention"])
            .stdout,
        tag
    );
}

#[tokio::test]
async fn npr_10_automatic_suspend() {
    if std::env::var_os("LIBRA_NPR10_REVALIDATE_CHILD").is_some() {
        // Owned test subprocess: its cwd/home belong to the parent fixture.
        // Exercise the actual automatic revalidation owner, not a test stub.
        let record = trust::read_trust("opencode").await.unwrap().unwrap();
        assert!(trust::revalidate_trust("opencode", &record).await.is_err());
        assert!(trust::read_trust("opencode").await.unwrap().is_none());
        return;
    }
    let repo = Fixture::new();
    repo.trust();
    let (_, issuer) = repo.enroll("suspension").await;
    let original = repo.registry().await.unwrap();
    assert!(repo.admitted(&issuer).await);
    for cause in ["stale-metadata", "directory-withdrawal"] {
        let db = repo.db().await;
        if cause == "stale-metadata" {
            let mut record: Value = serde_json::from_str(
                &repo
                    .rows()
                    .await
                    .into_iter()
                    .find(|(k, _, _)| k == "agent.trust.opencode")
                    .unwrap()
                    .1,
            )
            .unwrap();
            record["mtime"] = json!(record["mtime"].as_i64().unwrap() + 1);
            db.execute_raw(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "UPDATE config_kv SET value=? WHERE key='agent.trust.opencode'",
                [record.to_string().into()],
            ))
            .await
            .unwrap();
        } else {
            repo.ok(&["config", "set", "agent.external_agents.trusted_dirs", "[]"]);
        }
        let out = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "npr_10_automatic_suspend", "--nocapture"])
            .current_dir(&repo.repo)
            .env_clear()
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .env("HOME", &repo.home)
            .env("LIBRA_TEST_HOME", &repo.home)
            .env("LIBRA_TEST", "1")
            .env("LIBRA_NPR10_REVALIDATE_CHILD", cause)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", describe(&out));
        assert!(!repo.admitted(&issuer).await);
        assert_eq!(
            repo.registry().await.unwrap(),
            original,
            "automatic removal must not create a permanent native tombstone"
        );
        repo.trust();
        assert!(repo.admitted(&issuer).await);
    }
    repo.untrust();
    repo.trust();
    assert!(
        !repo.admitted(&issuer).await,
        "explicit native revocation must remain permanent"
    );
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
    // The supported ordinary installer must keep its current v2 template;
    // it cannot satisfy this immutable private producer's v1 recipe.
    repo.ok(&["agent", "enable", "--agent", "opencode"]);
    let ordinary_plugin = fs::read(&plugin).unwrap();
    assert_ne!(ordinary_plugin, original_plugin);
    assert!(
        !String::from_utf8_lossy(&ordinary_plugin)
            .contains("// libra-plugin-template: opencode-2.0.26-v1")
    );
    let mismatch = repo.assert_refused_unchanged(&manifest, 0).await;
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("private producer's legacy recipe"));
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
    repo.install_legacy_observer();
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
