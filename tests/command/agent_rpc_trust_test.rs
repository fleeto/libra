//! CLI-level coverage for the AG-18 external-agent security surface:
//! the `agent.external_agents.enabled` gate (`LBR-AGENT-002`), the
//! trust/quarantine flow, provenance revalidation (`LBR-AGENT-005`),
//! transport fail-closed (`LBR-AGENT-012`) and stderr redaction at the
//! CLI boundary.

#![cfg(unix)]

use std::path::Path;

use super::{init_repo_via_cli, run_libra_command_with_stdin_and_env};

fn plant_script(dir: &Path, slug: &str, body: &str) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(format!("libra-agent-{slug}"));
    std::fs::write(&path, body).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

/// Run libra with a PATH that contains the fixture dir (the spawned
/// binary discovers `libra-agent-*` through its own PATH).
fn run_with_path(args: &[&str], cwd: &Path, fixture_dir: &Path) -> std::process::Output {
    run_with_path_and_env(args, cwd, fixture_dir, &[])
}

fn run_with_path_and_env(
    args: &[&str],
    cwd: &Path,
    fixture_dir: &Path,
    extra_env: &[(&str, &str)],
) -> std::process::Output {
    let path_var = format!(
        "{}:{}",
        fixture_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut envs: Vec<(&str, &str)> = vec![("PATH", &path_var)];
    envs.extend_from_slice(extra_env);
    run_libra_command_with_stdin_and_env(args, cwd, "", &envs)
}

/// Init a repo with a fixture-bin dir and the external-agents gate open.
/// A0-08: the fixture dir is registered as a trusted directory so
/// `rpc trust <slug>` binaries planted there pass the allowlist enforcement.
fn setup_enabled(temp: &Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let repo = temp.join("repo");
    init_repo_via_cli(&repo);
    let fixtures = temp.join("bin");
    std::fs::create_dir_all(&fixtures).unwrap();
    let set = run_with_path(
        &["config", "set", "agent.external_agents.enabled", "true"],
        &repo,
        &fixtures,
    );
    assert!(set.status.success(), "config set must succeed");
    let trust_dir = run_with_path(
        &["agent", "rpc", "trust", "--dir", fixtures.to_str().unwrap()],
        &repo,
        &fixtures,
    );
    assert!(
        trust_dir.status.success(),
        "trust --dir must succeed: {}",
        String::from_utf8_lossy(&trust_dir.stderr)
    );
    (repo, fixtures)
}

const ANSWERER: &str = "#!/bin/sh\n\
    read _l; printf '{\"jsonrpc\":\"2.0\",\"id\":1,\"error\":{\"code\":-32601,\"message\":\"no info\"}}\\n'\n\
    read _l; printf '{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"methods\":[\"capabilities\",\"provider_kind\"]}}\\n'\n\
    read _l; printf '{\"jsonrpc\":\"2.0\",\"id\":3,\"result\":{\"kind\":\"ext\"}}\\n'\n";

/// The external-agent surface is fail-closed by default: `list`, `trust`
/// and `invoke` all refuse with `LBR-AGENT-002` until the operator opts
/// in (`untrust` stays available); after opting in, an untrusted binary
/// stays quarantined (`LBR-AGENT-005`), and the full trust → invoke path
/// works end to end.
#[test]
fn external_agents_gate_trust_and_invoke_flow() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    init_repo_via_cli(&repo);
    let fixtures = temp.path().join("bin");
    std::fs::create_dir_all(&fixtures).unwrap();
    plant_script(&fixtures, "ext", ANSWERER);

    // Gate off: every entry point that touches external binaries —
    // list discovery included — refuses with LBR-AGENT-002.
    for args in [
        vec!["agent", "rpc", "list"],
        vec!["agent", "rpc", "invoke", "ext", "provider_kind"],
        vec!["agent", "rpc", "trust", "ext"],
    ] {
        let out = run_with_path(&args, &repo, &fixtures);
        assert!(!out.status.success(), "{args:?} must refuse while gated");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("LBR-AGENT-002"),
            "{args:?} carries the gate code: {stderr}"
        );
        assert!(
            stderr.contains("agent.external_agents.enabled"),
            "{args:?} names the opt-in key: {stderr}"
        );
    }

    // untrust bypasses the gate (revoking trust only tightens security).
    let gated_untrust = run_with_path(&["agent", "rpc", "untrust", "ext"], &repo, &fixtures);
    assert!(
        gated_untrust.status.success(),
        "untrust works while gated: {}",
        String::from_utf8_lossy(&gated_untrust.stderr)
    );

    // Opt in.
    let set = run_with_path(
        &["config", "set", "agent.external_agents.enabled", "true"],
        &repo,
        &fixtures,
    );
    assert!(set.status.success(), "config set must succeed");
    // A0-08: register the fixture dir as trusted so `trust ext` (below) passes
    // the trusted-directory allowlist.
    let trust_dir = run_with_path(
        &["agent", "rpc", "trust", "--dir", fixtures.to_str().unwrap()],
        &repo,
        &fixtures,
    );
    assert!(
        trust_dir.status.success(),
        "trust --dir must succeed: {}",
        String::from_utf8_lossy(&trust_dir.stderr)
    );

    // list now works and shows the binary as quarantined.
    let list = run_with_path(&["agent", "rpc", "list"], &repo, &fixtures);
    assert!(list.status.success());
    let list_out = String::from_utf8_lossy(&list.stdout);
    assert!(list_out.contains("ext") && list_out.contains("quarantined"));

    // Still quarantined before trust: invoke refuses with LBR-AGENT-005.
    let untrusted = run_with_path(
        &["agent", "rpc", "invoke", "ext", "provider_kind"],
        &repo,
        &fixtures,
    );
    assert!(!untrusted.status.success());
    let untrusted_err = String::from_utf8_lossy(&untrusted.stderr);
    assert!(
        untrusted_err.contains("LBR-AGENT-005") && untrusted_err.contains("quarantined"),
        "untrusted invoke fails closed: {untrusted_err}"
    );

    // Trust, then invoke succeeds.
    let trust = run_with_path(&["agent", "rpc", "trust", "ext"], &repo, &fixtures);
    assert!(
        trust.status.success(),
        "trust: {}",
        String::from_utf8_lossy(&trust.stderr)
    );
    let invoke = run_with_path(
        &["agent", "rpc", "invoke", "ext", "provider_kind"],
        &repo,
        &fixtures,
    );
    assert!(
        invoke.status.success(),
        "trusted invoke: {}",
        String::from_utf8_lossy(&invoke.stderr)
    );
    assert!(String::from_utf8_lossy(&invoke.stdout).contains("ext"));

    // Tamper with the binary: provenance drift revokes trust fail-closed.
    plant_script(&fixtures, "ext", "#!/bin/sh\n# tampered\nexit 0\n");
    let tampered = run_with_path(
        &["agent", "rpc", "invoke", "ext", "provider_kind"],
        &repo,
        &fixtures,
    );
    assert!(!tampered.status.success());
    let tampered_err = String::from_utf8_lossy(&tampered.stderr);
    assert!(
        tampered_err.contains("LBR-AGENT-005"),
        "drift is a provenance rejection: {tampered_err}"
    );
    // And the trust record is gone — the binary is quarantined again.
    let relist = run_with_path(&["agent", "rpc", "list"], &repo, &fixtures);
    assert!(String::from_utf8_lossy(&relist.stdout).contains("quarantined"));
}

/// mtime-only drift (identical bytes, touched timestamp) is enough to
/// revoke trust: a swap-and-swap-back attack shows up as an mtime
/// change even when the hash matches again.
#[test]
fn mtime_only_drift_revokes_trust() {
    let temp = tempfile::tempdir().expect("tempdir");
    let (repo, fixtures) = setup_enabled(temp.path());
    let binary = plant_script(&fixtures, "ext", ANSWERER);

    let trust = run_with_path(&["agent", "rpc", "trust", "ext"], &repo, &fixtures);
    assert!(
        trust.status.success(),
        "trust: {}",
        String::from_utf8_lossy(&trust.stderr)
    );

    // Backdate the binary's mtime — bytes unchanged.
    let touch = std::process::Command::new("touch")
        .args(["-t", "202001010000"])
        .arg(&binary)
        .status()
        .expect("run touch");
    assert!(touch.success(), "touch must backdate the fixture");

    let invoke = run_with_path(
        &["agent", "rpc", "invoke", "ext", "provider_kind"],
        &repo,
        &fixtures,
    );
    assert!(!invoke.status.success(), "mtime drift must fail closed");
    let stderr = String::from_utf8_lossy(&invoke.stderr);
    assert!(
        stderr.contains("LBR-AGENT-005"),
        "mtime drift is a provenance rejection: {stderr}"
    );
    let relist = run_with_path(&["agent", "rpc", "list"], &repo, &fixtures);
    assert!(
        String::from_utf8_lossy(&relist.stdout).contains("quarantined"),
        "drifted binary returns to quarantine"
    );
}

/// A binary sitting in a world-writable directory can never be trusted:
/// any local user could swap it between trust and invoke.
#[test]
fn world_writable_dir_binary_cannot_be_trusted() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().expect("tempdir");
    let (repo, fixtures) = setup_enabled(temp.path());
    plant_script(&fixtures, "ext", ANSWERER);
    std::fs::set_permissions(&fixtures, std::fs::Permissions::from_mode(0o777)).unwrap();

    let trust = run_with_path(&["agent", "rpc", "trust", "ext"], &repo, &fixtures);
    assert!(
        !trust.status.success(),
        "world-writable parent must refuse trust"
    );
    let stderr = String::from_utf8_lossy(&trust.stderr);
    assert!(
        stderr.contains("LBR-AGENT-005") && stderr.contains("world-writable"),
        "trust rejection is a provenance rejection naming the cause: {stderr}"
    );
}

/// A trusted binary that dies before answering the invoked method is a
/// transport failure: the CLI surfaces `LBR-AGENT-012`, not a raw IO
/// error or the IO-cap/redaction code.
#[test]
fn transport_failure_surfaces_stable_code_at_cli() {
    let temp = tempfile::tempdir().expect("tempdir");
    let (repo, fixtures) = setup_enabled(temp.path());
    // Answers info (error frame) + capabilities, then exits before the
    // method call — the invoke sees EOF / broken pipe.
    let body = "#!/bin/sh\n\
        read _l; printf '{\"jsonrpc\":\"2.0\",\"id\":1,\"error\":{\"code\":-32601,\"message\":\"no info\"}}\\n'\n\
        read _l; printf '{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"methods\":[\"capabilities\",\"provider_kind\"]}}\\n'\n\
        exit 0\n";
    plant_script(&fixtures, "ext", body);
    let trust = run_with_path(&["agent", "rpc", "trust", "ext"], &repo, &fixtures);
    assert!(trust.status.success());

    let invoke = run_with_path(
        &["agent", "rpc", "invoke", "ext", "provider_kind"],
        &repo,
        &fixtures,
    );
    assert!(!invoke.status.success());
    let stderr = String::from_utf8_lossy(&invoke.stderr);
    assert!(
        stderr.contains("LBR-AGENT-012"),
        "early exit is a transport failure: {stderr}"
    );
}

/// Child stderr never reaches the operator raw: secrets are redacted in
/// the default human error output and in the structured JSON error
/// payload alike.
#[test]
fn stderr_redaction_holds_at_cli_boundary() {
    let temp = tempfile::tempdir().expect("tempdir");
    let (repo, fixtures) = setup_enabled(temp.path());
    // Leaks an AWS access key id to stderr, then dies before answering
    // the method so the failure path attaches the stderr excerpt.
    let body = "#!/bin/sh\n\
        printf 'leaked credential AKIAIOSFODNN7EXAMPLE end\\n' >&2\n\
        read _l; printf '{\"jsonrpc\":\"2.0\",\"id\":1,\"error\":{\"code\":-32601,\"message\":\"no info\"}}\\n'\n\
        read _l; printf '{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"methods\":[\"capabilities\",\"provider_kind\"]}}\\n'\n\
        exit 0\n";
    plant_script(&fixtures, "ext", body);
    let trust = run_with_path(&["agent", "rpc", "trust", "ext"], &repo, &fixtures);
    assert!(trust.status.success());

    // Default human output.
    let invoke = run_with_path(
        &["agent", "rpc", "invoke", "ext", "provider_kind"],
        &repo,
        &fixtures,
    );
    assert!(!invoke.status.success());
    let human = format!(
        "{}{}",
        String::from_utf8_lossy(&invoke.stdout),
        String::from_utf8_lossy(&invoke.stderr)
    );
    assert!(
        !human.contains("AKIAIOSFODNN7EXAMPLE"),
        "raw secret must not leak into default output: {human}"
    );

    // Structured JSON error payload.
    let json_invoke = run_with_path_and_env(
        &["agent", "rpc", "invoke", "ext", "provider_kind"],
        &repo,
        &fixtures,
        &[("LIBRA_ERROR_JSON", "1")],
    );
    assert!(!json_invoke.status.success());
    let json_out = format!(
        "{}{}",
        String::from_utf8_lossy(&json_invoke.stdout),
        String::from_utf8_lossy(&json_invoke.stderr)
    );
    assert!(
        !json_out.contains("AKIAIOSFODNN7EXAMPLE"),
        "raw secret must not leak into JSON error payload: {json_out}"
    );
    assert!(
        json_out.contains("LBR-AGENT-012"),
        "JSON error still carries the stable code: {json_out}"
    );
}

/// A PATH binary whose slug impersonates a built-in agent is invisible to
/// discovery and rejected by trust AND invoke with `LBR-AGENT-006`.
#[test]
fn builtin_slug_impersonation_is_rejected() {
    let temp = tempfile::tempdir().expect("tempdir");
    let (repo, fixtures) = setup_enabled(temp.path());
    plant_script(&fixtures, "claude-code", ANSWERER);

    let list = run_with_path(&["agent", "rpc", "list"], &repo, &fixtures);
    assert!(
        !String::from_utf8_lossy(&list.stdout).contains("claude-code"),
        "impersonator must not be discovered"
    );

    for args in [
        vec!["agent", "rpc", "trust", "claude-code"],
        vec!["agent", "rpc", "invoke", "claude-code", "provider_kind"],
    ] {
        let out = run_with_path(&args, &repo, &fixtures);
        assert!(!out.status.success(), "{args:?} must refuse impersonation");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("LBR-AGENT-006"),
            "{args:?} rejects impersonation with the stable code"
        );
    }
}

/// A0-08: `rpc trust --dir` registers a trusted directory; a binary under a
/// trusted dir is trustable, one outside every trusted dir is rejected
/// (`LBR-AGENT-005`), and a world-writable directory can never be registered.
#[test]
fn agent_rpc_trust_dir() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().expect("tempdir");
    // setup_enabled opens the gate AND registers `fixtures` as a trusted dir.
    let (repo, fixtures) = setup_enabled(temp.path());

    // (a) `trust --dir` emits the canonical trusted directory (JSON).
    let other = temp.path().join("other-bin");
    std::fs::create_dir_all(&other).unwrap();
    let reg = run_with_path(
        &[
            "--json",
            "agent",
            "rpc",
            "trust",
            "--dir",
            other.to_str().unwrap(),
        ],
        &repo,
        &fixtures,
    );
    assert!(
        reg.status.success(),
        "trust --dir must succeed: {}",
        String::from_utf8_lossy(&reg.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&reg.stdout).unwrap_or_default();
    assert!(
        json["data"]["trusted_dir"]
            .as_str()
            .unwrap_or("")
            .contains("other-bin"),
        "trust --dir emits the canonical dir: {}",
        String::from_utf8_lossy(&reg.stdout)
    );

    // (b) a binary under a trusted dir is trustable.
    plant_script(&fixtures, "inn", ANSWERER);
    let ok = run_with_path(&["agent", "rpc", "trust", "inn"], &repo, &fixtures);
    assert!(
        ok.status.success(),
        "binary under a trusted dir must be trustable: {}",
        String::from_utf8_lossy(&ok.stderr)
    );

    // (c) a binary NOT under any trusted directory is rejected (LBR-AGENT-005).
    let untrusted_dir = temp.path().join("untrusted-bin");
    std::fs::create_dir_all(&untrusted_dir).unwrap();
    plant_script(&untrusted_dir, "out", ANSWERER);
    let path_var = format!(
        "{}:{}:{}",
        untrusted_dir.display(),
        fixtures.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let rej = run_libra_command_with_stdin_and_env(
        &["agent", "rpc", "trust", "out"],
        &repo,
        "",
        &[("PATH", &path_var)],
    );
    assert!(
        !rej.status.success(),
        "a binary outside every trusted dir must be rejected"
    );
    let stderr = String::from_utf8_lossy(&rej.stderr);
    assert!(
        stderr.contains("LBR-AGENT-005") && stderr.contains("trusted director"),
        "rejection is a provenance refusal naming the cause: {stderr}"
    );

    // (d) a world-writable directory can never be registered.
    let ww = temp.path().join("ww-bin");
    std::fs::create_dir_all(&ww).unwrap();
    std::fs::set_permissions(&ww, std::fs::Permissions::from_mode(0o777)).unwrap();
    let ww_reg = run_with_path(
        &["agent", "rpc", "trust", "--dir", ww.to_str().unwrap()],
        &repo,
        &fixtures,
    );
    assert!(
        !ww_reg.status.success(),
        "a world-writable directory must be refused"
    );
    assert!(
        String::from_utf8_lossy(&ww_reg.stderr).contains("world-writable"),
        "refusal names the cause: {}",
        String::from_utf8_lossy(&ww_reg.stderr)
    );
}

/// DR-04b: provider-exporter trust (`rpc trust opencode`) pins the provider's
/// OWN CLI binary for the sandboxed export bridge without opening the
/// external-RPC surface: it works while `agent.external_agents.enabled` is
/// off, resolves the binary only from registered trusted directories (never
/// `$PATH`), refuses with `LBR-AGENT-005` when no trusted directory holds
/// the binary, stays un-invokable as an RPC agent (`LBR-AGENT-006`), and is
/// revocable through the ordinary `untrust` path.
#[test]
fn provider_exporter_trust_is_ungated_and_trusted_dir_bound() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    init_repo_via_cli(&repo);
    let fixtures = temp.path().join("bin");
    std::fs::create_dir_all(&fixtures).unwrap();
    // The provider's own CLI binary (NOT libra-agent-* prefixed).
    let exporter = fixtures.join("opencode");
    std::fs::write(&exporter, "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(&exporter, std::fs::Permissions::from_mode(0o755)).unwrap();

    // Gate deliberately left OFF for the whole registration flow.

    // (a) No trusted directory holds the binary yet: refusal carries the
    // provenance code and the actionable --dir hint, NOT the RPC gate code.
    let missing = run_with_path(&["agent", "rpc", "trust", "opencode"], &repo, &fixtures);
    assert!(!missing.status.success(), "must refuse without trusted dir");
    let stderr = String::from_utf8_lossy(&missing.stderr);
    assert!(
        stderr.contains("LBR-AGENT-005") && stderr.contains("trust --dir"),
        "refusal is a provenance rejection with the --dir hint: {stderr}"
    );
    assert!(
        !stderr.contains("LBR-AGENT-002"),
        "exporter trust must not be gated behind the external-RPC opt-in: {stderr}"
    );

    // (b) Registering the trusted directory is also un-gated preparation.
    let trust_dir = run_with_path(
        &["agent", "rpc", "trust", "--dir", fixtures.to_str().unwrap()],
        &repo,
        &fixtures,
    );
    assert!(
        trust_dir.status.success(),
        "trust --dir works while gated: {}",
        String::from_utf8_lossy(&trust_dir.stderr)
    );

    // (c) Exporter trust now succeeds and names the provider-exporter role.
    let trusted = run_with_path(&["agent", "rpc", "trust", "opencode"], &repo, &fixtures);
    assert!(
        trusted.status.success(),
        "exporter trust succeeds: {}",
        String::from_utf8_lossy(&trusted.stderr)
    );
    let stdout = String::from_utf8_lossy(&trusted.stdout);
    assert!(
        stdout.contains("provider exporter 'opencode'") && stdout.contains("sha256"),
        "trust output names the exporter role and provenance: {stdout}"
    );

    // (d) The exporter slug never becomes an invokable RPC agent: while the
    // surface is gated the gate answers; once enabled, impersonation does.
    let gated_invoke = run_with_path(
        &["agent", "rpc", "invoke", "opencode", "provider_kind"],
        &repo,
        &fixtures,
    );
    assert!(!gated_invoke.status.success());
    assert!(
        String::from_utf8_lossy(&gated_invoke.stderr).contains("LBR-AGENT-002"),
        "invoke stays gated regardless of exporter trust"
    );
    let set = run_with_path(
        &["config", "set", "agent.external_agents.enabled", "true"],
        &repo,
        &fixtures,
    );
    assert!(set.status.success());
    let enabled_invoke = run_with_path(
        &["agent", "rpc", "invoke", "opencode", "provider_kind"],
        &repo,
        &fixtures,
    );
    assert!(!enabled_invoke.status.success());
    assert!(
        String::from_utf8_lossy(&enabled_invoke.stderr).contains("LBR-AGENT-006"),
        "an exporter slug is never invokable as an external RPC binary: {}",
        String::from_utf8_lossy(&enabled_invoke.stderr)
    );

    // (e) Revocation goes through the ordinary untrust path with the
    // exporter label.
    let untrust = run_with_path(&["agent", "rpc", "untrust", "opencode"], &repo, &fixtures);
    assert!(
        untrust.status.success(),
        "untrust works: {}",
        String::from_utf8_lossy(&untrust.stderr)
    );
    assert!(
        String::from_utf8_lossy(&untrust.stdout).contains("provider exporter 'opencode'"),
        "untrust names the exporter role: {}",
        String::from_utf8_lossy(&untrust.stdout)
    );
}

/// The `.bin/opencode` shim mise's npm backend (aube) writes for
/// `@opencode/cli`: it execs a target named relative to its own location.
fn aube_opencode_shim(target: &str) -> String {
    format!(
        "#!/bin/sh\n# aube-bin-shim v2 target={target}\nlink=\"$0\"\n\
         basedir=$(dirname \"$link\")\n\
         export NODE_PATH=\"$basedir/..:$basedir/../.mise/node_modules\"\n\
         exec \"$basedir/{target}\" \"$@\"\n"
    )
}

fn write_executable(path: &Path, bytes: &[u8]) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

fn opencode_export_bridge_method(repo: &Path, fixtures: &Path) -> serde_json::Value {
    let list = run_with_path(
        &["--json", "agent", "list", "--schema-version", "2"],
        repo,
        fixtures,
    );
    assert!(
        list.status.success(),
        "agent list v2 must succeed: {}",
        String::from_utf8_lossy(&list.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    json["data"]["agents"]
        .as_array()
        .and_then(|agents| agents.iter().find(|row| row["slug"] == "opencode"))
        .and_then(|row| row["methods"].as_array())
        .and_then(|methods| methods.iter().find(|m| m["name"] == "export_bridge"))
        .cloned()
        .unwrap_or_else(|| panic!("no OpenCode export_bridge method in {json}"))
}

/// The export sandbox maps only the pinned exporter file, so a
/// package-manager shim (here mise's aube `.bin/opencode`) cannot reach the
/// native OpenCode it launches and every capture silently exported nothing.
/// `rpc trust opencode` therefore pins the shim's declared native target —
/// once that target's own directory is trusted, which the refusal names —
/// and a record that still pins a shim (written by an earlier release)
/// reports the export bridge unavailable, without being revoked, until it is
/// re-trusted.
#[test]
fn provider_exporter_trust_pins_the_native_target_of_a_package_manager_shim() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    init_repo_via_cli(&repo);
    let modules = temp
        .path()
        .canonicalize()
        .unwrap()
        .join("npm-opencode-cli/2.0.29/node_modules");
    let shim_dir = modules.join(".bin");
    let package_bin = modules.join(".mise/@opencode+cli@2.0.29/node_modules/@opencode/cli/bin");
    let native = package_bin.join("opencode.exe");
    write_executable(&native, b"\x7fELF\x02\x01\x01\x00");
    let shim = shim_dir.join("opencode");
    write_executable(
        &shim,
        aube_opencode_shim(
            "../.mise/@opencode+cli@2.0.29/node_modules/@opencode/cli/bin/opencode.exe",
        )
        .as_bytes(),
    );

    // (a) Trusting only the shim's directory is refused with the provenance
    // code and the exact directory of the native executable to register.
    let shim_dir_arg = shim_dir.to_str().unwrap();
    let registered = run_with_path(
        &["agent", "rpc", "trust", "--dir", shim_dir_arg],
        &repo,
        &shim_dir,
    );
    assert!(
        registered.status.success(),
        "{}",
        String::from_utf8_lossy(&registered.stderr)
    );
    let refused = run_with_path(&["agent", "rpc", "trust", "opencode"], &repo, &shim_dir);
    assert!(!refused.status.success(), "a shim must never be pinned");
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains("LBR-AGENT-005")
            && stderr.contains("package-manager launcher")
            && stderr.contains(&format!("trust --dir {}", package_bin.display())),
        "refusal names the native executable's directory: {stderr}"
    );

    // (b) With that directory trusted too, the native target is pinned and
    // the shim is reported as its launcher.
    let package_bin_arg = package_bin.to_str().unwrap();
    let registered = run_with_path(
        &["agent", "rpc", "trust", "--dir", package_bin_arg],
        &repo,
        &shim_dir,
    );
    assert!(
        registered.status.success(),
        "{}",
        String::from_utf8_lossy(&registered.stderr)
    );
    let trusted = run_with_path(
        &["--json", "agent", "rpc", "trust", "opencode"],
        &repo,
        &shim_dir,
    );
    assert!(
        trusted.status.success(),
        "{}",
        String::from_utf8_lossy(&trusted.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&trusted.stdout).unwrap();
    assert_eq!(json["data"]["path"], native.to_str().unwrap(), "{json}");
    assert_eq!(json["data"]["launcher"], shim.to_str().unwrap(), "{json}");
    let human = run_with_path(&["agent", "rpc", "trust", "opencode"], &repo, &shim_dir);
    assert!(
        String::from_utf8_lossy(&human.stdout).contains(&format!("launched by {}", shim.display())),
        "{}",
        String::from_utf8_lossy(&human.stdout)
    );

    // (c) A record that still pins the shim keeps the export bridge
    // unavailable but is not revoked: it did not drift.
    let provenance = libra::internal::ai::observed_agents::compute_provenance(&shim).unwrap();
    let stale = serde_json::json!({
        "path": provenance.canonical_path,
        "sha256": provenance.sha256,
        "device": provenance.device,
        "inode": provenance.inode,
        "mtime": provenance.mtime,
    })
    .to_string();
    let seeded = run_with_path(
        &["config", "set", "agent.trust.opencode", &stale],
        &repo,
        &shim_dir,
    );
    assert!(
        seeded.status.success(),
        "{}",
        String::from_utf8_lossy(&seeded.stderr)
    );
    let method = opencode_export_bridge_method(&repo, &shim_dir);
    assert_eq!(method["available"], false, "{method}");
    let kept = run_with_path(&["config", "get", "agent.trust.opencode"], &repo, &shim_dir);
    assert!(
        kept.status.success()
            && String::from_utf8_lossy(&kept.stdout).contains(shim.to_str().unwrap()),
        "a launcher record is refused, not revoked: {}",
        String::from_utf8_lossy(&kept.stdout)
    );

    // (d) Re-trusting repairs it.
    let repaired = run_with_path(
        &["--json", "agent", "rpc", "trust", "opencode"],
        &repo,
        &shim_dir,
    );
    assert!(
        repaired.status.success(),
        "{}",
        String::from_utf8_lossy(&repaired.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&repaired.stdout).unwrap();
    assert_eq!(json["data"]["path"], native.to_str().unwrap(), "{json}");
}

/// A JavaScript launcher (npm's `opencode-ai` wrapper) locates the native
/// binary in its package tree at run time, which the export sandbox does not
/// provide: it is refused rather than pinned.
#[test]
fn provider_exporter_trust_refuses_a_javascript_launcher() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    init_repo_via_cli(&repo);
    let fixtures = temp.path().join("bin");
    write_executable(
        &fixtures.join("opencode"),
        b"#!/usr/bin/env node\nrequire('../lib/launch.js')\n",
    );
    let registered = run_with_path(
        &["agent", "rpc", "trust", "--dir", fixtures.to_str().unwrap()],
        &repo,
        &fixtures,
    );
    assert!(
        registered.status.success(),
        "{}",
        String::from_utf8_lossy(&registered.stderr)
    );
    let refused = run_with_path(&["agent", "rpc", "trust", "opencode"], &repo, &fixtures);
    assert!(
        !refused.status.success(),
        "a JavaScript launcher must not be pinned"
    );
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains("LBR-AGENT-005") && stderr.contains("JavaScript launcher"),
        "{stderr}"
    );
}

/// R85: provider-exporter trust has no fixed binary-size cap. The stock
/// OpenCode CLI is a ~171 MiB single-file Bun binary, so a 200 MiB (sparse)
/// `opencode` must be trustable, its sha256 must be the whole-file digest, and
/// the read-only `agent list --schema-version 2` revalidation must keep (not
/// revoke) the record. The former 64 MiB cap refused the trust and revoked an
/// existing record on the next list.
#[test]
fn provider_exporter_trust_accepts_large_single_file_binary_and_list_keeps_it() {
    use std::os::unix::fs::PermissionsExt;

    const LARGE_EXPORTER_BYTES: u64 = 200 * 1024 * 1024;
    // `printf '#!/bin/sh\nexit 0\n'` zero-extended to 200 MiB, digested
    // independently with `shasum -a 256`.
    const LARGE_EXPORTER_SHA256: &str =
        "4e045ec6b0ef4cdc8c82e9d87983f6bafb0e2950c0c8ad971d391657589ec0c4";

    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    init_repo_via_cli(&repo);
    let fixtures = temp.path().join("bin");
    std::fs::create_dir_all(&fixtures).unwrap();
    let exporter = fixtures.join("opencode");
    std::fs::write(&exporter, "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::OpenOptions::new()
        .write(true)
        .open(&exporter)
        .and_then(|file| file.set_len(LARGE_EXPORTER_BYTES))
        .expect("extend exporter sparsely past the real OpenCode size");
    std::fs::set_permissions(&exporter, std::fs::Permissions::from_mode(0o755)).unwrap();

    let trust_dir = run_with_path(
        &["agent", "rpc", "trust", "--dir", fixtures.to_str().unwrap()],
        &repo,
        &fixtures,
    );
    assert!(
        trust_dir.status.success(),
        "trust --dir must succeed: {}",
        String::from_utf8_lossy(&trust_dir.stderr)
    );

    let trusted = run_with_path(
        &["--json", "agent", "rpc", "trust", "opencode"],
        &repo,
        &fixtures,
    );
    assert!(
        trusted.status.success(),
        "a realistic-size exporter must be trustable: {}",
        String::from_utf8_lossy(&trusted.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&trusted.stdout).unwrap_or_default();
    assert_eq!(
        json["data"]["kind"],
        "provider-exporter",
        "{}",
        String::from_utf8_lossy(&trusted.stdout)
    );
    assert_eq!(
        json["data"]["sha256"],
        LARGE_EXPORTER_SHA256,
        "trust records the whole-file sha256: {}",
        String::from_utf8_lossy(&trusted.stdout)
    );

    // `agent list --schema-version 2` revalidates the OpenCode exporter trust
    // (and revokes it on any revalidation error), so it must succeed and leave
    // the record in place.
    let list = run_with_path(
        &["--json", "agent", "list", "--schema-version", "2"],
        &repo,
        &fixtures,
    );
    assert!(
        list.status.success(),
        "agent list v2 must succeed: {}",
        String::from_utf8_lossy(&list.stderr)
    );
    let untrust = run_with_path(
        &["--json", "agent", "rpc", "untrust", "opencode"],
        &repo,
        &fixtures,
    );
    assert!(
        untrust.status.success(),
        "untrust works: {}",
        String::from_utf8_lossy(&untrust.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&untrust.stdout).unwrap_or_default();
    assert_eq!(
        json["data"]["removed"],
        true,
        "agent list revalidation must not revoke a valid large exporter trust: {}",
        String::from_utf8_lossy(&untrust.stdout)
    );
}

/// HM-08: `rpc trust hermes` pins the Hermes executable itself. OpenCode trust
/// does not satisfy it, and the slug stays uninvokable as an RPC agent.
#[test]
fn hermes_exporter_trust_is_not_opencode_trust() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    init_repo_via_cli(&repo);
    let fixtures = temp.path().join("bin");
    std::fs::create_dir_all(&fixtures).unwrap();
    for name in ["hermes", "opencode"] {
        let path = fixtures.join(name);
        std::fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let trust_dir = run_with_path(
        &["agent", "rpc", "trust", "--dir", fixtures.to_str().unwrap()],
        &repo,
        &fixtures,
    );
    assert!(
        trust_dir.status.success(),
        "{}",
        describe_output(&trust_dir)
    );

    let before = run_with_path(&["agent", "doctor"], &repo, &fixtures);
    assert!(before.status.success(), "{}", describe_output(&before));
    let before_out = String::from_utf8_lossy(&before.stdout);
    assert!(
        before_out.contains("Hermes export bridge: trusted=no")
            && before_out.contains("native origin is not claimed"),
        "{before_out}"
    );

    let opencode = run_with_path(&["agent", "rpc", "trust", "opencode"], &repo, &fixtures);
    assert!(opencode.status.success(), "{}", describe_output(&opencode));
    let still = run_with_path(&["agent", "doctor"], &repo, &fixtures);
    assert!(
        String::from_utf8_lossy(&still.stdout).contains("Hermes export bridge: trusted=no"),
        "{}",
        String::from_utf8_lossy(&still.stdout)
    );

    let trusted = run_with_path(&["agent", "rpc", "trust", "hermes"], &repo, &fixtures);
    assert!(trusted.status.success(), "{}", describe_output(&trusted));
    let stdout = String::from_utf8_lossy(&trusted.stdout);
    assert!(
        stdout.contains("provider exporter 'hermes'") && stdout.contains("sha256"),
        "{stdout}"
    );
    let after = run_with_path(&["agent", "doctor"], &repo, &fixtures);
    let after_out = String::from_utf8_lossy(&after.stdout);
    assert!(
        after_out.contains("Hermes export bridge: trusted=yes")
            && after_out.contains("cloud sync stays refused"),
        "{after_out}"
    );

    let invoke = run_with_path(
        &["agent", "rpc", "invoke", "hermes", "provider_kind"],
        &repo,
        &fixtures,
    );
    assert!(!invoke.status.success());
    assert!(
        String::from_utf8_lossy(&invoke.stderr).contains("LBR-AGENT-002"),
        "{}",
        String::from_utf8_lossy(&invoke.stderr)
    );
    let untrust = run_with_path(&["agent", "rpc", "untrust", "hermes"], &repo, &fixtures);
    assert!(untrust.status.success(), "{}", describe_output(&untrust));
    let untrust_out = String::from_utf8_lossy(&untrust.stdout);
    assert!(
        untrust_out.contains("provider exporter 'hermes'")
            && !untrust_out.contains("native issuers"),
        "{untrust_out}"
    );
}

fn describe_output(output: &std::process::Output) -> String {
    format!(
        "status={:?} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}
