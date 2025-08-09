use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use std::time::Duration;

#[test]
fn test_help_message() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Run a command with a timeout"))
        .stdout(predicate::str::contains("SECONDS"))
        .stdout(predicate::str::contains("COMMAND"));
}

#[test]
fn test_basic_command_success() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&["5", "echo", "Hello World"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}

#[test]
fn test_command_with_multiple_args() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&["5", "echo", "arg1", "arg2", "arg3"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("arg1 arg2 arg3"));
}

#[test]
fn test_command_with_flags() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&["5", "ls", "-la"]);

    cmd.assert().success();
}

#[test]
fn test_exit_code_forwarding_success() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&["5", "sh", "-c", "exit 0"]);

    cmd.assert().code(0);
}

#[test]
fn test_exit_code_forwarding_failure() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&["5", "sh", "-c", "exit 42"]);

    cmd.assert().code(42);
}

#[test]
fn test_timeout_kills_long_running_command() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&["1", "sleep", "5"]);

    let start = std::time::Instant::now();
    cmd.assert().code(124); // Standard timeout exit code
    let elapsed = start.elapsed();

    // Should timeout after approximately 1 second, not 5
    assert!(
        elapsed < Duration::from_secs(3),
        "Command took too long: {:?}",
        elapsed
    );
    assert!(
        elapsed >= Duration::from_millis(800),
        "Command finished too quickly: {:?}",
        elapsed
    );
}

#[test]
fn test_command_completes_before_timeout() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&["3", "echo", "quick_command"]);

    let start = std::time::Instant::now();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("quick_command"));
    let elapsed = start.elapsed();

    // Echo should complete very quickly, well before the 3 second timeout
    assert!(
        elapsed < Duration::from_secs(2),
        "Command took too long: {:?}",
        elapsed
    );
}

#[test]
fn test_nonexistent_command() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&["5", "this_command_does_not_exist"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to execute command"));
}

#[test]
fn test_invalid_timeout_value() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&["not_a_number", "echo", "test"]);

    cmd.assert().failure();
}

#[test]
fn test_missing_command() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&["5"]);

    cmd.assert().failure();
}

#[test]
fn test_zero_timeout() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&["0", "echo", "test"]);

    // With zero timeout, the command should be killed immediately
    cmd.assert().code(124);
}

#[test]
fn test_very_short_timeout() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&["1", "echo", "fast_command"]);

    // Echo should complete well within 1 second
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("fast_command"));
}

#[test]
fn test_command_with_stdout_and_stderr() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&[
        "5",
        "sh",
        "-c",
        "echo 'stdout message'; echo 'stderr message' >&2",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("stdout message"))
        .stderr(predicate::str::contains("stderr message"));
}

#[test]
fn test_long_timeout_with_quick_command() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&["30", "echo", "quick"]);

    // Should finish quickly despite long timeout
    let start = std::time::Instant::now();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("quick"));
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(5),
        "Command took too long: {:?}",
        elapsed
    );
}

#[test]
fn test_command_with_spaces_in_args() {
    let mut cmd = Command::cargo_bin("timeout").unwrap();
    cmd.args(&["5", "echo", "hello world", "with spaces"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("hello world with spaces"));
}

#[test]
fn test_multiple_timeouts_sequential() {
    // Test running multiple timeout commands in sequence
    for i in 1..=3 {
        let mut cmd = Command::cargo_bin("timeout").unwrap();
        cmd.args(&["2", "echo", &format!("test_{}", i)]);

        cmd.assert()
            .success()
            .stdout(predicate::str::contains(&format!("test_{}", i)));
    }
}
