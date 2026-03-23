use std::cmp::Ordering;
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use std::process;

use tukubai_core::{
    KeyCompareError, KeyOptions, KeyParseError, KeyProgram, KeyResolveError, OutputError,
    OutputTarget, ParseError, RecordReader, command_error, compare_resolved_keys, is_stdin_path,
    normalize_key_positions_to_one, parse_key_program, resolve_key, resolve_key_positions,
    resolve_key_with_positions,
};

const BINARY_NAME: &str = "join0";

fn main() {
    if let Err(error) = run() {
        let _ = writeln!(io::stderr().lock(), "{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(env::args_os().skip(1))?;

    if is_stdin_path(&args.master) && args.tran.as_deref().is_none_or(is_stdin_path) {
        return Err(command_error!(
            BINARY_NAME,
            "master and tran must not both read from standard input"
        ));
    }

    let key_program = parse_key_program(
        args.key.as_os_str().as_bytes(),
        KeyOptions {
            allow_numeric: true,
            allow_descending: true,
        },
    )
    .map_err(|error| format_key_parse_error(BINARY_NAME, error))?;

    let mut stdout = OutputTarget::stdout();
    let mut ng_file = open_ng_writer(args.ng_fd)?;
    let ng_writer = ng_file.as_mut();

    if is_stdin_path(&args.master) {
        let tran_path = args
            .tran
            .as_deref()
            .expect("validated that tran is present when master is stdin");
        let tran_file =
            File::open(tran_path).map_err(|error| command_error!(BINARY_NAME, error))?;
        let stdin = io::stdin();
        return process_join(
            stdin.lock(),
            BufReader::new(tran_file),
            &key_program,
            &mut stdout,
            ng_writer,
        );
    }

    let master_file =
        File::open(&args.master).map_err(|error| command_error!(BINARY_NAME, error))?;

    if let Some(tran_path) = args.tran.as_deref() {
        if is_stdin_path(tran_path) {
            let stdin = io::stdin();
            process_join(
                BufReader::new(master_file),
                stdin.lock(),
                &key_program,
                &mut stdout,
                ng_writer,
            )
        } else {
            let tran_file =
                File::open(tran_path).map_err(|error| command_error!(BINARY_NAME, error))?;
            process_join(
                BufReader::new(master_file),
                BufReader::new(tran_file),
                &key_program,
                &mut stdout,
                ng_writer,
            )
        }
    } else {
        let stdin = io::stdin();
        process_join(
            BufReader::new(master_file),
            stdin.lock(),
            &key_program,
            &mut stdout,
            ng_writer,
        )
    }
}

fn process_join<M: BufRead, T: BufRead>(
    master: M,
    tran: T,
    key_program: &KeyProgram,
    stdout: &mut OutputTarget,
    mut ng_writer: Option<&mut OutputTarget>,
) -> Result<(), String> {
    let mut master_reader = RecordReader::new(master);
    let mut tran_reader = RecordReader::new(tran);
    let mut current_master = read_owned_record(&mut master_reader)?;

    while let Some(tran_record) = tran_reader
        .read_record()
        .map_err(|error| format_parse_error(BINARY_NAME, error))?
    {
        let tran_positions = resolve_key_positions(key_program, tran_record)
            .map_err(|error| format_key_resolve_error(BINARY_NAME, error))?;
        let master_positions = normalize_key_positions_to_one(&tran_positions);
        let tran_key = resolve_key(key_program, tran_record)
            .map_err(|error| format_key_resolve_error(BINARY_NAME, error))?;

        let matched = loop {
            let Some(master_record) = current_master.as_deref() else {
                break false;
            };

            let master_key = resolve_key_with_positions(&master_positions, master_record)
                .map_err(|error| format_key_resolve_error(BINARY_NAME, error))?;

            match compare_resolved_keys(&master_key, &tran_key)
                .map_err(|error| format_key_compare_error(BINARY_NAME, error))?
            {
                Ordering::Less => {
                    current_master = read_owned_record(&mut master_reader)?;
                }
                Ordering::Equal => break true,
                Ordering::Greater => break false,
            }
        };

        if matched {
            stdout
                .write_record(tran_record)
                .map_err(|error| format_output_error(BINARY_NAME, error))?;
        } else if let Some(writer) = ng_writer.as_deref_mut() {
            writer
                .write_record(tran_record)
                .map_err(|error| format_output_error(BINARY_NAME, error))?;
        }
    }

    Ok(())
}

fn read_owned_record<R: BufRead>(reader: &mut RecordReader<R>) -> Result<Option<Vec<u8>>, String> {
    reader
        .read_record()
        .map_err(|error| format_parse_error(BINARY_NAME, error))
        .map(|record| record.map(|bytes| bytes.to_vec()))
}

fn open_ng_writer(fd: Option<u32>) -> Result<Option<OutputTarget>, String> {
    let Some(fd) = fd else {
        return Ok(None);
    };

    OutputTarget::borrowed_fd(fd as i32)
        .map(Some)
        .map_err(|error| format_output_error(BINARY_NAME, error))
}

fn format_parse_error(binary_name: &str, error: ParseError) -> String {
    command_error!(binary_name, error)
}

fn format_key_parse_error(binary_name: &str, error: KeyParseError) -> String {
    command_error!(binary_name, error)
}

fn format_key_resolve_error(binary_name: &str, error: KeyResolveError) -> String {
    command_error!(binary_name, error)
}

fn format_key_compare_error(binary_name: &str, error: KeyCompareError) -> String {
    command_error!(binary_name, error)
}

fn format_output_error(binary_name: &str, error: OutputError) -> String {
    command_error!(binary_name, error)
}

fn parse_args<I>(args: I) -> Result<Args, String>
where
    I: IntoIterator,
    I::Item: Into<OsString>,
{
    let tokens: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let (ng_fd, rest) = parse_optional_ng(&tokens)?;

    match rest {
        [key, master] => Ok(Args {
            ng_fd,
            key: parse_key_argument(key)?,
            master: PathBuf::from(master),
            tran: None,
        }),
        [key, master, tran] => Ok(Args {
            ng_fd,
            key: parse_key_argument(key)?,
            master: PathBuf::from(master),
            tran: Some(PathBuf::from(tran)),
        }),
        _ => Err(command_error!(
            BINARY_NAME,
            "usage: join0 [+ng<fd>] key=<key> <master> [<tran>]"
        )),
    }
}

fn parse_optional_ng(tokens: &[OsString]) -> Result<(Option<u32>, &[OsString]), String> {
    let Some(first) = tokens.first() else {
        return Ok((None, tokens));
    };

    let bytes = first.as_os_str().as_bytes();
    let Some(fd_bytes) = bytes.strip_prefix(b"+ng") else {
        return Ok((None, tokens));
    };

    if fd_bytes.is_empty() || !fd_bytes.iter().all(|byte| byte.is_ascii_digit()) {
        return Err(command_error!(BINARY_NAME, "invalid +ng file descriptor"));
    }

    let fd = parse_u32(fd_bytes)?;
    Ok((Some(fd), &tokens[1..]))
}

fn parse_key_argument(token: &OsString) -> Result<OsString, String> {
    let bytes = token.as_os_str().as_bytes();
    let Some(key) = bytes.strip_prefix(b"key=") else {
        return Err(command_error!(BINARY_NAME, "missing key=<key> argument"));
    };

    if key.is_empty() {
        return Err(command_error!(BINARY_NAME, "missing key=<key> argument"));
    }

    Ok(OsString::from_vec(key.to_vec()))
}

fn parse_u32(input: &[u8]) -> Result<u32, String> {
    let mut value = 0_u32;

    for byte in input {
        value = value
            .checked_mul(10)
            .and_then(|current| current.checked_add(u32::from(byte - b'0')))
            .ok_or_else(|| command_error!(BINARY_NAME, "invalid +ng file descriptor"))?;
    }

    Ok(value)
}

struct Args {
    ng_fd: Option<u32>,
    key: OsString,
    master: PathBuf,
    tran: Option<PathBuf>,
}
