//! `libra ui` stays a normal command when the desktop feature is off.
//!
//! The window itself needs `--features gui` and a display, so these cases
//! only cover help text and the feature-off refusal.

use tempfile::tempdir;

use super::run_libra_command;

#[test]
fn ui_help_shows_examples() {
    let dir = tempdir().expect("temp dir");
    let output = run_libra_command(&["ui", "--help"], dir.path());
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("EXAMPLES:"),
        "`libra ui --help` should show examples, got: {stdout}"
    );
    assert!(
        stdout.contains("libra ui"),
        "examples should include the launch command, got: {stdout}"
    );
}

#[cfg(not(feature = "gui"))]
#[test]
fn ui_without_feature_names_the_rebuild() {
    let dir = tempdir().expect("temp dir");
    let output = run_libra_command(&["--json", "ui"], dir.path());
    assert_eq!(
        output.status.code(),
        Some(128),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("LBR-UNSUPPORTED-001"),
        "feature-off `libra ui` should use the unsupported code, got: {stderr}"
    );
    assert!(
        stderr.contains("--features gui"),
        "feature-off `libra ui` should name the rebuild, got: {stderr}"
    );
}
