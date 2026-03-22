use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn filters_non_matching_records_from_stdin() {
    let output = Command::new(env!("CARGO_BIN_EXE_delr"))
        .args(["2", "b"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child
                .stdin
                .take()
                .unwrap()
                .write_all(b"a b c\nx y z\nq b r\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"x y z\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn filters_non_matching_records_from_file() {
    let file_path = write_temp_file("delr-filters-file-records", b"a b c\nx y z\n");

    let output = Command::new(env!("CARGO_BIN_EXE_delr"))
        .arg("2")
        .arg("y")
        .arg(&file_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"a b c\n");
    assert_eq!(output.stderr, b"");

    fs::remove_file(file_path).unwrap();
}

#[test]
fn preserves_original_record_bytes() {
    let output = Command::new(env!("CARGO_BIN_EXE_delr"))
        .args(["2", "x"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"  a  b  \n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"  a  b  \n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn resolves_nf_per_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_delr"))
        .args(["NF", "done"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child
                .stdin
                .take()
                .unwrap()
                .write_all(b"a done\nb c\nx y done\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"b c\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn accepts_zero_padded_selector_numbers() {
    let output = Command::new(env!("CARGO_BIN_EXE_delr"))
        .args(["01", "a"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"a b\nx a\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"x a\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn empty_string_matches_all_records_as_special_case() {
    let output = Command::new(env!("CARGO_BIN_EXE_delr"))
        .args(["1", ""])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"\n  a  \n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"\n  a  \n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn rejects_zero_selector() {
    let output = Command::new(env!("CARGO_BIN_EXE_delr"))
        .args(["0", "x"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"Error(108)[delr] : selector 0 is not supported by this command\n"
    );
}

#[test]
fn rejects_range_selector() {
    let output = Command::new(env!("CARGO_BIN_EXE_delr"))
        .args(["1/2", "x"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"Error(108)[delr] : range selectors are not supported by this command\n"
    );
}

#[test]
fn rejects_missing_field() {
    let output = Command::new(env!("CARGO_BIN_EXE_delr"))
        .args(["4", "x"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"a b c\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"Error(112)[delr] : selector resolved to a non-existent field\n"
    );
}

#[test]
fn rejects_unterminated_final_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_delr"))
        .args(["1", "alpha"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"alpha")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"Error(104)[delr] : final record is not terminated by LF\n"
    );
}

#[test]
fn reads_stdin_when_dash_is_given_as_file_name() {
    let output = Command::new(env!("CARGO_BIN_EXE_delr"))
        .args(["2", "b", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"a b\nx y\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"x y\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn rejects_invalid_argument_count() {
    let output = Command::new(env!("CARGO_BIN_EXE_delr"))
        .arg("1")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"Error(133)[delr] : usage: delr <fldnum> <str> [<file>]\n"
    );
}

fn write_temp_file(prefix: &str, contents: &[u8]) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{unique}.txt"));
    fs::write(&path, contents).unwrap();
    path
}
