use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::process;

use tukubai_core::{ParseError, RecordReader, STDIN_SOURCE_NAME, command_error, is_stdin_path};

const BINARY_NAME: &str = "lcnt";

fn main() {
    if let Err(error) = run() {
        let _ = writeln!(io::stderr().lock(), "{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(env::args_os().skip(1))?;
    let mut stdout = io::stdout().lock();

    if args.files.is_empty() {
        let stdin = io::stdin();
        let count =
            count_records(stdin.lock()).map_err(|error| format_parse_error(BINARY_NAME, error))?;
        let display_name = args.show_file_name.then_some(Path::new(STDIN_SOURCE_NAME));
        write_output(&mut stdout, display_name, count, args.show_file_name)
            .map_err(|error| error.to_string())?;
        return Ok(());
    }

    for file_name in &args.files {
        let count = if is_stdin_path(file_name) {
            let stdin = io::stdin();
            count_records(stdin.lock()).map_err(|error| format_parse_error(BINARY_NAME, error))?
        } else {
            let file = File::open(file_name).map_err(|error| command_error!(BINARY_NAME, error))?;
            count_records(BufReader::new(file))
                .map_err(|error| format_parse_error(BINARY_NAME, error))?
        };
        write_output(&mut stdout, Some(file_name), count, args.show_file_name)
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn count_records<R: BufRead>(reader: R) -> Result<u64, ParseError> {
    let mut reader = RecordReader::new(reader);
    let mut count = 0_u64;

    while reader.read_record()?.is_some() {
        count += 1;
    }

    Ok(count)
}

fn write_output<W: Write>(
    writer: &mut W,
    file_name: Option<&Path>,
    count: u64,
    show_file_name: bool,
) -> io::Result<()> {
    if show_file_name {
        let file_name = file_name.expect("file name is required when -f is enabled");
        writeln!(writer, "{} {}", file_name.display(), count)
    } else {
        writeln!(writer, "{count}")
    }
}

fn format_parse_error(binary_name: &str, error: ParseError) -> String {
    command_error!(binary_name, error)
}

fn parse_args<I>(args: I) -> Result<Args, String>
where
    I: IntoIterator,
    I::Item: Into<std::ffi::OsString>,
{
    let mut show_file_name = false;
    let mut files = Vec::new();

    for arg in args {
        let arg = arg.into();
        if arg == "-f" {
            show_file_name = true;
        } else {
            files.push(arg.into());
        }
    }

    Ok(Args {
        show_file_name,
        files,
    })
}

struct Args {
    show_file_name: bool,
    files: Vec<std::path::PathBuf>,
}
