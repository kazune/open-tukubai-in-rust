use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::process;

use tukubai_core::{
    ParseError, RecordReader, ResolvedItem, SelectorOptions, SelectorParseError,
    SelectorResolveError, command_error, is_stdin_path, parse_selectors, resolve_selectors,
};

const BINARY_NAME: &str = "delr";

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
            process_records(stdin.lock(), &args.selector, &args.needle, &mut stdout)
        } else {
            let file = File::open(file_name).map_err(|error| command_error!(BINARY_NAME, error))?;
            process_records(
                BufReader::new(file),
                &args.selector,
                &args.needle,
                &mut stdout,
            )
        }
    } else {
        let stdin = io::stdin();
        process_records(stdin.lock(), &args.selector, &args.needle, &mut stdout)
    }
}

fn process_records<R: BufRead, W: Write>(
    reader: R,
    selector: &OsStr,
    needle: &OsStr,
    writer: &mut W,
) -> Result<(), String> {
    let needle = needle.as_bytes();
    let mut reader = RecordReader::new(reader);

    if needle.is_empty() {
        while let Some(record) = reader
            .read_record()
            .map_err(|error| format_parse_error(BINARY_NAME, error))?
        {
            write_record(writer, record).map_err(|error| error.to_string())?;
        }

        return Ok(());
    }

    let program = parse_selector_program(selector)?;

    while let Some(record) = reader
        .read_record()
        .map_err(|error| format_parse_error(BINARY_NAME, error))?
    {
        let resolved = resolve_selectors(&program, record)
            .map_err(|error| format_selector_resolve_error(BINARY_NAME, error))?;

        let field = match resolved.as_slice() {
            [ResolvedItem::Field(field)] => *field,
            _ => {
                return Err(command_error!(
                    BINARY_NAME,
                    "internal error: selector did not resolve to exactly one field"
                ));
            }
        };

        if field != needle {
            write_record(writer, record).map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}

fn write_record<W: Write>(writer: &mut W, record: &[u8]) -> io::Result<()> {
    writer.write_all(record)?;
    writer.write_all(b"\n")
}

fn parse_selector_program(selector: &OsStr) -> Result<tukubai_core::SelectorProgram, String> {
    parse_selectors([selector.as_bytes()], SelectorOptions::single_field(false))
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

    match tokens.as_slice() {
        [selector, needle] => Ok(Args {
            selector: selector.clone(),
            needle: needle.clone(),
            file: None,
        }),
        [selector, needle, file] => Ok(Args {
            selector: selector.clone(),
            needle: needle.clone(),
            file: Some(PathBuf::from(file)),
        }),
        _ => Err(command_error!(
            BINARY_NAME,
            "usage: delr <fldnum> <str> [<file>]"
        )),
    }
}

struct Args {
    selector: OsString,
    needle: OsString,
    file: Option<PathBuf>,
}
