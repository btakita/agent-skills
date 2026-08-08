//! Coverage for `.github/scripts/cargo-publish.sh`, the release workflow's
//! publish step. It must tolerate ONLY the idempotent "this version is already
//! on crates.io" case and fail loudly on everything else.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::process::Command;

struct PublishRun {
    success: bool,
    stdout: String,
}

/// Run the publish script with a fake `cargo` that prints `output` and exits
/// with `code`.
fn run_publish(output: &str, code: i32) -> PublishRun {
    let dir = tempfile::tempdir().unwrap();
    let fake_cargo = dir.path().join("fake-cargo");
    std::fs::write(
        &fake_cargo,
        format!("#!/usr/bin/env bash\ncat <<'FAKE_CARGO_EOF'\n{output}\nFAKE_CARGO_EOF\nexit {code}\n"),
    )
    .unwrap();
    std::fs::set_permissions(&fake_cargo, std::fs::Permissions::from_mode(0o755)).unwrap();

    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/.github/scripts/cargo-publish.sh");
    let out = Command::new("bash")
        .arg(script)
        .current_dir(dir.path())
        .env("CARGO", &fake_cargo)
        .output()
        .unwrap();

    PublishRun {
        success: out.status.success(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
    }
}

#[test]
fn publish_succeeds_when_cargo_succeeds() {
    let run = run_publish("    Uploading skill-harness v0.1.4 (/w/skill-harness)\n     Uploaded skill-harness v0.1.4 to registry `crates-io`", 0);
    assert!(run.success, "stdout:\n{}", run.stdout);
    assert!(!run.stdout.contains("::notice::"), "stdout:\n{}", run.stdout);
}

/// crates.io's server-side rejection. This is the wording the registry actually
/// returns (`duplicate_version_error` in crates.io's publish controller), and
/// it does NOT contain the phrase "already exists".
#[test]
fn publish_tolerates_server_duplicate_version() {
    let run = run_publish(
        "error: failed to publish to registry at https://crates.io\n\nCaused by:\n  \
         the remote server responded with an error (status 200 OK): crate version `0.1.3` is already uploaded",
        101,
    );
    assert!(run.success, "stdout:\n{}", run.stdout);
    assert!(run.stdout.contains("::notice::"), "stdout:\n{}", run.stdout);
}

/// cargo's client-side pre-check (`verify_unpublished`), which fires before the
/// upload when the registry index already has the version.
#[test]
fn publish_tolerates_client_duplicate_version() {
    let run = run_publish(
        "error: crate skill-harness@0.1.3 already exists on registry `crates-io`",
        101,
    );
    assert!(run.success, "stdout:\n{}", run.stdout);
    assert!(run.stdout.contains("::notice::"), "stdout:\n{}", run.stdout);
}

/// The regression that motivated 32836ad: an expired `CARGO_REGISTRY_TOKEN`
/// must turn the job red, never green.
#[test]
fn publish_fails_on_authentication_error() {
    let run = run_publish(
        "error: failed to publish to registry at https://crates.io\n\nCaused by:\n  \
         the remote server responded with an error (status 403 Forbidden): authentication failed",
        101,
    );
    assert!(!run.success, "stdout:\n{}", run.stdout);
    assert!(run.stdout.contains("::error::"), "stdout:\n{}", run.stdout);
}

#[test]
fn publish_fails_on_unrelated_error() {
    let run = run_publish(
        "error: failed to verify package tarball\n\nCaused by:\n  \
         could not compile `skill-harness` (lib) due to 1 previous error",
        101,
    );
    assert!(!run.success, "stdout:\n{}", run.stdout);
    assert!(run.stdout.contains("::error::"), "stdout:\n{}", run.stdout);
}
