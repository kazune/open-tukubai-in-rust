use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn selects_fields_from_stdin() {
    let output = Command::new(env!("CARGO_BIN_EXE_self"))
        .args(["2", "1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"a b c\nx y z\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"b a\ny x\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn selects_fields_from_file() {
    let file_path = write_temp_file("self-selects-file-fields", b"a b c\n");

    let output = Command::new(env!("CARGO_BIN_EXE_self"))
        .arg("3")
        .arg(&file_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"c\n");
    assert_eq!(output.stderr, b"");

    fs::remove_file(file_path).unwrap();
}

#[test]
fn resolves_nf_per_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_self"))
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
                .write_all(b"a b c\nalpha beta\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"b c\nalpha beta\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn expands_reverse_ranges() {
    let output = Command::new(env!("CARGO_BIN_EXE_self"))
        .arg("4/2")
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
    assert_eq!(output.stdout, b"d c b\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn outputs_raw_record_with_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_self"))
        .args(["2", "0"])
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
    assert_eq!(output.stdout, b"b   a  b  \n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn preserves_duplicate_selectors() {
    let output = Command::new(env!("CARGO_BIN_EXE_self"))
        .args(["2", "2/3", "2"])
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

    assert!(output.status.success());
    assert_eq!(output.stdout, b"b b c b\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn rejects_invalid_selector_syntax() {
    let output = Command::new(env!("CARGO_BIN_EXE_self"))
        .arg("NF+1")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"Error(93)[self] : invalid selector syntax\n"
    );
}

#[test]
fn rejects_missing_field() {
    let output = Command::new(env!("CARGO_BIN_EXE_self"))
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
        b"Error(97)[self] : selector resolved to a non-existent field\n"
    );
}

#[test]
fn rejects_unterminated_final_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_self"))
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
        b"Error(89)[self] : final record is not terminated by LF\n"
    );
}

#[test]
fn rejects_empty_record_for_non_zero_selector() {
    let output = Command::new(env!("CARGO_BIN_EXE_self"))
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
        b"Error(97)[self] : selector resolved to a non-existent field\n"
    );
}

#[test]
fn allows_zero_for_empty_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_self"))
        .arg("0")
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

    assert!(output.status.success());
    assert_eq!(output.stdout, b"\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn reads_stdin_when_dash_is_given_as_file_name() {
    let output = Command::new(env!("CARGO_BIN_EXE_self"))
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
    assert_eq!(output.stdout, b"a\n");
    assert_eq!(output.stderr, b"");
}

fn write_temp_file(prefix: &str, contents: &[u8]) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{unique}.tmp"));
    fs::write(&path, contents).unwrap();
    path
}
