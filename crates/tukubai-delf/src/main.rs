use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::process;

use tukubai_core::{
    ParseError, RecordReader, SelectorOptions, SelectorParseError, SelectorResolveError,
    command_error, is_stdin_path, parse_selectors, resolve_selector_positions, split_fields,
};

const BINARY_NAME: &str = "delf";

fn main() {
    if let Err(error) = run() {
        let _ = writeln!(io::stderr().lock(), "{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(env::args_os().skip(1))?;
    let mut stdout = io::stdout().lock();

    if let Some(file_name) = args.file.as_deref() {
        if is_stdin_path(file_name) {
            let stdin = io::stdin();
            process_records(stdin.lock(), &args.selectors, &mut stdout)
        } else {
            let file = File::open(file_name).map_err(|error| command_error!(BINARY_NAME, error))?;
            process_records(BufReader::new(file), &args.selectors, &mut stdout)
        }
    } else {
        let stdin = io::stdin();
        process_records(stdin.lock(), &args.selectors, &mut stdout)
    }
}

fn process_records<R: BufRead, W: Write>(
    reader: R,
    selectors: &[OsString],
    writer: &mut W,
) -> Result<(), String> {
    let program = parse_selector_program(selectors)?;
    let mut reader = RecordReader::new(reader);

    while let Some(record) = reader
        .read_record()
        .map_err(|error| format_parse_error(BINARY_NAME, error))?
    {
        let fields: Vec<&[u8]> = split_fields(record).collect();
        let positions = resolve_selector_positions(&program, record)
            .map_err(|error| format_selector_resolve_error(BINARY_NAME, error))?;

        let mut removed = vec![false; fields.len()];
        for position in positions {
            let index = position.to_zero_based().ok_or_else(|| {
                command_error!(BINARY_NAME, "internal selector position overflow")
            })?;
            removed[index] = true;
        }

        write_filtered_record(writer, &fields, &removed).map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn write_filtered_record<W: Write>(
    writer: &mut W,
    fields: &[&[u8]],
    removed: &[bool],
) -> io::Result<()> {
    let mut wrote_any = false;

    for (field, is_removed) in fields.iter().zip(removed.iter().copied()) {
        if is_removed {
            continue;
        }

        if wrote_any {
            writer.write_all(b" ")?;
        }

        writer.write_all(field)?;
        wrote_any = true;
    }

    writer.write_all(b"\n")
}

fn parse_selector_program(selectors: &[OsString]) -> Result<tukubai_core::SelectorProgram, String> {
    parse_selectors(
        selectors
            .iter()
            .map(OsString::as_os_str)
            .map(OsStr::as_bytes),
        SelectorOptions::multi_field(false),
    )
    .map_err(|error| format_selector_parse_error(BINARY_NAME, error))
}

fn format_parse_error(binary_name: &str, error: ParseError) -> String {
    command_error!(binary_name, error)
}

fn format_selector_parse_error(binary_name: &str, error: SelectorParseError) -> String {
    command_error!(binary_name, error)
}

fn format_selector_resolve_error(binary_name: &str, error: SelectorResolveError) -> String {
    command_error!(binary_name, error)
}

fn parse_args<I>(args: I) -> Result<Args, String>
where
    I: IntoIterator,
    I::Item: Into<OsString>,
{
    let tokens: Vec<OsString> = args.into_iter().map(Into::into).collect();
    if tokens.is_empty() {
        return Err(command_error!(
            BINARY_NAME,
            "at least one selector is required"
        ));
    }

    if tokens.len() == 1 {
        return Ok(Args {
            selectors: tokens,
            file: None,
        });
    }

    let maybe_file = tokens.last().cloned().expect("tokens is not empty");
    let selector_tokens = &tokens[..tokens.len() - 1];

    if selector_tokens.is_empty() {
        return Err(command_error!(
            BINARY_NAME,
            "at least one selector is required"
        ));
    }

    let last_is_selector = parse_selectors(
        [maybe_file.as_os_str().as_bytes()],
        SelectorOptions::multi_field(false),
    )
    .is_ok();

    if last_is_selector {
        return Ok(Args {
            selectors: tokens,
            file: None,
        });
    }

    Ok(Args {
        selectors: selector_tokens.to_vec(),
        file: Some(PathBuf::from(maybe_file)),
    })
}

struct Args {
    selectors: Vec<OsString>,
    file: Option<PathBuf>,
}
