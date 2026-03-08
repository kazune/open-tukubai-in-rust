use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn counts_stdin_records() {
    let output = Command::new(env!("CARGO_BIN_EXE_lcnt"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"a\n\nb\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"3\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn counts_file_records_with_names() {
    let file_path = write_temp_file("lcnt-counts-file-records", b"a\nb\n");

    let output = Command::new(env!("CARGO_BIN_EXE_lcnt"))
        .arg("-f")
        .arg(&file_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        format!("{} 2\n", file_path.display()).into_bytes()
    );
    assert_eq!(output.stderr, b"");

    fs::remove_file(file_path).unwrap();
}

#[test]
fn reports_unterminated_final_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_lcnt"))
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
    assert_eq!(output.stderr, b"final record is not terminated by LF\n");
}

#[test]
fn reads_stdin_when_dash_is_given_as_file_name() {
    let output = Command::new(env!("CARGO_BIN_EXE_lcnt"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"a\nb\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"2\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn prints_dash_for_stdin_with_f_option() {
    let output = Command::new(env!("CARGO_BIN_EXE_lcnt"))
        .arg("-f")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"a\nb\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"- 2\n");
    assert_eq!(output.stderr, b"");
}

#[test]
fn prints_dash_for_implicit_stdin_with_f_option() {
    let output = Command::new(env!("CARGO_BIN_EXE_lcnt"))
        .arg("-f")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"a\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"- 1\n");
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
