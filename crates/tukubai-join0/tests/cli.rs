use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn filters_matching_transaction_records() {
    let master = write_temp_file("join0-master-basic", b"a\nc\n");
    let tran = write_temp_file("join0-tran-basic", b"a x\nb y\nc z\n");

    let output = Command::new(env!("CARGO_BIN_EXE_join0"))
        .args(["key=1", master.to_str().unwrap(), tran.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"a x\nc z\n");
    assert_eq!(output.stderr, b"");

    fs::remove_file(master).unwrap();
    fs::remove_file(tran).unwrap();
}

#[test]
fn reads_transaction_records_from_stdin_when_omitted() {
    let master = write_temp_file("join0-master-stdin", b"a\nc\n");

    let output = Command::new(env!("CARGO_BIN_EXE_join0"))
        .args(["key=1", master.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"a x\nb y\nc z\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"a x\nc z\n");
    assert_eq!(output.stderr, b"");

    fs::remove_file(master).unwrap();
}

#[test]
fn preserves_original_transaction_record_bytes() {
    let master = write_temp_file("join0-master-preserve", b"b\n");

    let output = Command::new(env!("CARGO_BIN_EXE_join0"))
        .args(["key=2", master.to_str().unwrap()])
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

    fs::remove_file(master).unwrap();
}

#[test]
fn normalizes_transaction_key_positions_for_master_lookup() {
    let master = write_temp_file("join0-master-nf", b"x y z\n");
    let tran = write_temp_file("join0-tran-nf", b"a x y z\nb p q r\n");

    let output = Command::new(env!("CARGO_BIN_EXE_join0"))
        .args(["key=2/NF", master.to_str().unwrap(), tran.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"a x y z\n");
    assert_eq!(output.stderr, b"");

    fs::remove_file(master).unwrap();
    fs::remove_file(tran).unwrap();
}

#[test]
fn supports_numeric_key_comparison() {
    let master = write_temp_file("join0-master-numeric", b"01\n3.50\n");
    let tran = write_temp_file("join0-tran-numeric", b"1 alpha\n2 beta\n3.5 gamma\n");

    let output = Command::new(env!("CARGO_BIN_EXE_join0"))
        .args(["key=1n", master.to_str().unwrap(), tran.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"1 alpha\n3.5 gamma\n");
    assert_eq!(output.stderr, b"");

    fs::remove_file(master).unwrap();
    fs::remove_file(tran).unwrap();
}

#[test]
fn writes_non_matching_records_to_ng_fd() {
    let master = write_temp_file("join0-master-ng", b"a\n");
    let tran = write_temp_file("join0-tran-ng", b"a x\nb y\n");
    let ng = temp_path("join0-ng-out");
    let script = r#"exec 3>"$1"; "$2" +ng3 key=1 "$3" "$4""#;

    let output = Command::new("sh")
        .arg("-c")
        .arg(script)
        .arg("sh")
        .arg(ng.to_str().unwrap())
        .arg(env!("CARGO_BIN_EXE_join0"))
        .arg(master.to_str().unwrap())
        .arg(tran.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"a x\n");
    assert_eq!(output.stderr, b"");
    assert_eq!(fs::read(&ng).unwrap(), b"b y\n");

    fs::remove_file(master).unwrap();
    fs::remove_file(tran).unwrap();
    fs::remove_file(ng).unwrap();
}

#[test]
fn rejects_both_inputs_as_stdin() {
    let output = Command::new(env!("CARGO_BIN_EXE_join0"))
        .arg("key=1")
        .arg("-")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_stderr_message(
        &output.stderr,
        "master and tran must not both read from standard input",
    );
}

#[test]
fn rejects_invalid_key_argument_syntax() {
    let output = Command::new(env!("CARGO_BIN_EXE_join0"))
        .args(["key=", "master"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_stderr_message(&output.stderr, "missing key=<key> argument");
}

#[test]
fn rejects_invalid_key_program() {
    let output = Command::new(env!("CARGO_BIN_EXE_join0"))
        .args(["key=2/NFn", "master", "tran"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_stderr_message(
        &output.stderr,
        "range endpoints must use the same comparison attributes",
    );
}

#[test]
fn rejects_missing_transaction_field() {
    let master = write_temp_file("join0-master-missing-tran", b"a\n");

    let output = Command::new(env!("CARGO_BIN_EXE_join0"))
        .args(["key=2", master.to_str().unwrap()])
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

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_stderr_message(&output.stderr, "key resolved to a non-existent field");

    fs::remove_file(master).unwrap();
}

#[test]
fn rejects_missing_master_field_after_normalization() {
    let master = write_temp_file("join0-master-missing-master", b"x y\n");
    let tran = write_temp_file("join0-tran-missing-master", b"a x y z\n");

    let output = Command::new(env!("CARGO_BIN_EXE_join0"))
        .args(["key=2/NF", master.to_str().unwrap(), tran.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_stderr_message(&output.stderr, "key resolved to a non-existent field");

    fs::remove_file(master).unwrap();
    fs::remove_file(tran).unwrap();
}

#[test]
fn rejects_invalid_numeric_value_during_comparison() {
    let master = write_temp_file("join0-master-bad-number", b"1e3\n");
    let tran = write_temp_file("join0-tran-bad-number", b"1000 x\n");

    let output = Command::new(env!("CARGO_BIN_EXE_join0"))
        .args(["key=1n", master.to_str().unwrap(), tran.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_stderr_message(
        &output.stderr,
        "numeric key field is not a valid decimal number",
    );

    fs::remove_file(master).unwrap();
    fs::remove_file(tran).unwrap();
}

#[test]
fn rejects_unterminated_final_record() {
    let master = write_temp_file("join0-master-unterminated", b"a\n");

    let output = Command::new(env!("CARGO_BIN_EXE_join0"))
        .args(["key=1", master.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            child.stdin.take().unwrap().write_all(b"a")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_stderr_message(&output.stderr, "final record is not terminated by LF");

    fs::remove_file(master).unwrap();
}

fn write_temp_file(name: &str, contents: &[u8]) -> PathBuf {
    let path = temp_path(name);
    fs::write(&path, contents).unwrap();
    path
}

fn temp_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push(format!("{name}-{unique}.tmp"));
    path
}

fn assert_stderr_message(stderr: &[u8], message: &str) {
    let stderr = String::from_utf8(stderr.to_vec()).unwrap();
    assert!(
        stderr.starts_with("Error("),
        "stderr did not start with Error(...): {stderr}"
    );
    assert!(
        stderr.ends_with(&format!("[join0] : {message}\n")),
        "stderr did not end with expected message: {stderr}"
    );
}
