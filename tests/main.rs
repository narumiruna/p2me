use std::process::Command;

#[cfg(unix)]
#[test]
fn rejects_named_pipes_before_reading() {
    use std::{
        process::Stdio,
        thread,
        time::{Duration, Instant},
    };

    let temp = tempfile::tempdir().unwrap();
    for (file, args, expected) in [
        (
            ".env",
            &["verify", "status", "sub-123"][..],
            ".env must be a regular file",
        ),
        (
            "solution.lean",
            &["verify", "submit", "thm-123", "solution.lean"][..],
            "the Lean file must be a regular file",
        ),
    ] {
        assert!(
            Command::new("mkfifo")
                .arg(temp.path().join(file))
                .status()
                .unwrap()
                .success()
        );
        let mut child = Command::new(env!("CARGO_BIN_EXE_p2me"))
            .current_dir(temp.path())
            .env_remove("PROVE2ME_API_KEY")
            .env_remove("P2ME_TEST_PORT")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while child.try_wait().unwrap().is_none() {
            if Instant::now() >= deadline {
                let _ = child.kill();
                child.wait().unwrap();
                panic!("CLI did not reject {file} before the deadline");
            }
            thread::sleep(Duration::from_millis(5));
        }
        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "{output:?}"
        );
    }
}

#[test]
fn arguments_and_help() {
    let temp = tempfile::tempdir().unwrap();
    let command = || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_p2me"));
        command
            .env_remove("PROVE2ME_API_KEY")
            .env_remove("P2ME_TEST_PORT")
            .current_dir(temp.path());
        command
    };
    for (args, expected) in [
        (&["--help"][..], "p2me theorems search poincare"),
        (&["theorems", "--help"][..], "search"),
        (&["verify", "--help"][..], "status"),
        (&["verify", "submit", "--help"][..], "--proof-type"),
    ] {
        let output = command().args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    }
    let output = command().args(["verify", "status"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("SUBMISSION_ID"));
}

#[test]
fn test_port_is_only_read_in_test_builds() {
    let temp = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_p2me"))
        .current_dir(temp.path())
        .env_remove("PROVE2ME_API_KEY")
        .env("P2ME_TEST_PORT", "invalid")
        .args(["verify", "status", "sub-123"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let error = String::from_utf8_lossy(&output.stderr);
    if cfg!(all(feature = "test-endpoint", debug_assertions)) {
        assert!(error.contains("invalid test server port"), "{error}");
    } else {
        assert!(error.contains("set PROVE2ME_API_KEY"), "{error}");
    }
}
