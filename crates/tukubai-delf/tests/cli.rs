use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn removes_fields_from_stdin() {
    let output = Command::new(env!("CARGO_BIN_EXE_delf"))
        .args(["2", "4"])
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
                .write_all(b"a b c d\nx y z w\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"a c\nx z\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn removes_fields_from_file() {
    let file_path = write_temp_file("delf-removes-file-fields", b"a b c\n");

    let output = Command::new(env!("CARGO_BIN_EXE_delf"))
        .arg("2")
        .arg(&file_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"a c\n");
    assert_eq!(output.stderr, b"");

    fs::remove_file(file_path).unwrap();
}

#[test]
fn resolves_nf_per_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_delf"))
        .args(["NF-1", "NF"])
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
                .write_all(b"a b c\nalpha beta gamma delta\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"a\nalpha beta\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn removes_reverse_ranges() {
    let output = Command::new(env!("CARGO_BIN_EXE_delf"))
        .arg("4/2")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"a b c d e\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"a e\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn duplicate_selectors_remove_once() {
    let output = Command::new(env!("CARGO_BIN_EXE_delf"))
        .args(["2", "2/3", "2"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"a b c d\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"a d\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn removes_all_fields_as_empty_line() {
    let output = Command::new(env!("CARGO_BIN_EXE_delf"))
        .args(["1", "2"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"a b\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn rejects_zero_selector() {
    let output = Command::new(env!("CARGO_BIN_EXE_delf"))
        .arg("0")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"Error(110)[delf] : selector 0 is not supported by this command\n"
    );
}

#[test]
fn rejects_missing_field() {
    let output = Command::new(env!("CARGO_BIN_EXE_delf"))
        .arg("4")
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
        b"Error(114)[delf] : selector resolved to a non-existent field\n"
    );
}

#[test]
fn rejects_unterminated_final_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_delf"))
        .arg("1")
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
        b"Error(106)[delf] : final record is not terminated by LF\n"
    );
}

#[test]
fn rejects_empty_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_delf"))
        .arg("1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"Error(114)[delf] : selector resolved to a non-existent field\n"
    );
}

#[test]
fn reads_stdin_when_dash_is_given_as_file_name() {
    let output = Command::new(env!("CARGO_BIN_EXE_delf"))
        .args(["1", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"a b\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"b\n");
    assert_eq!(output.stderr, b"");
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
