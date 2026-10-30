//! CI carrier for the installer verification smoke (plan-20260821 A1-05).
//!
//! Drives `tests/data/install-smoke/run.sh`: twenty-four scenarios covering the
//! signed stable channel (verified install, tampered signature and tampered
//! payload, sha256 and size mismatches, expired / paused / revoked policy
//! branches, the stale-replay anti-downgrade floor, zero-size and
//! min_key_generation and key-validity-window policy rejections, the
//! non-canonical-serialization gate, the manifest-404 and
//! verifier-unavailable transition states, each with and without
//! `LIBRA_ALLOW_FALLBACK=1`). The harness
//! rewrites trust markers only in a COPY of the production installer and
//! asserts the production file stays byte-identical with zero runtime
//! key-override entry points.

use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    process::Command,
};

fn tool_available(tool: &str, arg: &str) -> bool {
    Command::new(tool)
        .arg(arg)
        .output()
        .map(|out| out.status.success())
        .is_ok_and(|ok| ok)
}

/// True when `bin` can create an Ed25519 key. macOS `/usr/bin/openssl` is
/// LibreSSL 3.3.6 and reports a version, but `genpkey -algorithm ed25519` fails.
fn openssl_can_ed25519(bin: &OsStr) -> bool {
    let dir = std::env::temp_dir().join(format!(
        "libra-install-smoke-ed25519-{}",
        std::process::id()
    ));
    if std::fs::create_dir_all(&dir).is_err() {
        return false;
    }
    let pem = dir.join("t.pem");
    let ok = Command::new(bin)
        .args(["genpkey", "-algorithm", "ed25519", "-out"])
        .arg(&pem)
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false);
    let _ = std::fs::remove_file(&pem);
    let _ = std::fs::remove_dir(&dir);
    ok
}

/// Which `openssl` the smoke child should see.
///
/// `OnPath` keeps the caller's `PATH`. `Prefixed` prepends a Homebrew OpenSSL 3
/// directory for the child only. The production `install.sh` is not modified.
enum SmokeOpenssl {
    OnPath,
    Prefixed(PathBuf),
    Unavailable,
}

fn smoke_openssl() -> SmokeOpenssl {
    if openssl_can_ed25519(OsStr::new("openssl")) {
        return SmokeOpenssl::OnPath;
    }
    for dir in [
        "/opt/homebrew/opt/openssl@3/bin",
        "/usr/local/opt/openssl@3/bin",
    ] {
        let bin = Path::new(dir).join("openssl");
        if bin.is_file() && openssl_can_ed25519(bin.as_os_str()) {
            return SmokeOpenssl::Prefixed(PathBuf::from(dir));
        }
    }
    SmokeOpenssl::Unavailable
}

#[test]
fn install_sh_smoke_scenarios() {
    for (tool, arg) in [
        ("bash", "--version"),
        ("python3", "--version"),
        ("openssl", "version"),
    ] {
        if !tool_available(tool, arg) {
            eprintln!("skipped (install-smoke needs {tool} on PATH)");
            return;
        }
    }
    let prefix = match smoke_openssl() {
        SmokeOpenssl::OnPath => None,
        SmokeOpenssl::Prefixed(dir) => Some(dir),
        SmokeOpenssl::Unavailable => {
            eprintln!(
                "skipped (install-smoke needs an Ed25519 openssl; system openssl cannot genpkey ed25519 and no Homebrew OpenSSL 3 was found)"
            );
            return;
        }
    };
    let harness = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/install-smoke/run.sh"
    );
    let mut command = Command::new("bash");
    command.arg(harness);
    if let Some(dir) = &prefix {
        let mut path = OsString::from(dir);
        path.push(":");
        if let Some(current) = std::env::var_os("PATH") {
            path.push(current);
        }
        command.env("PATH", path);
        eprintln!("install-smoke using Ed25519 openssl from {}", dir.display());
    }
    let output = command.output().expect("install-smoke harness must spawn");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "install-smoke harness failed\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
    assert!(
        stdout.contains("all 24 scenarios passed"),
        "harness did not report full coverage:\n{stdout}"
    );
}

#[test]
fn install_ps1_smoke_scenarios() {
    // The PowerShell smoke needs pwsh; environments without it skip with a
    // notice (the gap is registered in plan-20260821 A1-05).
    if !tool_available("pwsh", "-Version") {
        eprintln!("skipped (set up pwsh to run the install.ps1 smoke)");
        return;
    }
    let harness = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/install-smoke/run.ps1"
    );
    let output = Command::new("pwsh")
        .args(["-NoProfile", "-File", harness])
        .output()
        .expect("install.ps1 smoke harness must spawn");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "install.ps1 smoke failed\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
}
