//! The CLI status helper must work under the public platform asset filename.

use std::{fs, path::Path, process::Command};

fn run_libra(binary: &Path, repo: &Path, home: &Path, args: &[&str]) {
    let output = Command::new(binary)
        .args(args)
        .current_dir(repo)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("LIBRA_CONFIG_GLOBAL_DB", home.join(".libra/config.db"))
        .env(
            "LIBRA_CONFIG_SYSTEM_DB",
            home.join(".libra/system-config.db"),
        )
        .env("LIBRA_TEST", "1")
        .output()
        .expect("run release-named Libra binary");
    assert!(
        output.status.success(),
        "{args:?} failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn renamed_release_binary_can_start_status_io_worker() {
    let root = tempfile::tempdir().expect("isolated release fixture");
    let repo = root.path().join("repo");
    fs::create_dir(&repo).expect("create repository directory");
    let home = root.path().join("home");
    fs::create_dir(&home).expect("create isolated home");

    let binary_name = if cfg!(windows) {
        "libra-windows-amd64.exe"
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "aarch64") {
            "libra-linux-arm64"
        } else {
            "libra-linux-amd64"
        }
    } else {
        "libra-darwin-arm64"
    };
    let release_binary = root.path().join(binary_name);
    fs::copy(env!("CARGO_BIN_EXE_libra"), &release_binary).expect("copy release binary");

    run_libra(&release_binary, &repo, &home, &["init", "--vault=false"]);
    fs::write(repo.join("renamed.txt"), "renamed binary\n").expect("write fixture");
    run_libra(&release_binary, &repo, &home, &["add", "renamed.txt"]);
}
