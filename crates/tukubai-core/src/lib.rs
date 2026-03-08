use std::error::Error as StdError;
use std::fmt;
use std::io::{self, BufRead};

pub const STDIN_SOURCE_NAME: &str = "-";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinalTermination {
    RequireLf,
    AllowUnterminatedFinalRecord,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderOptions {
    pub final_termination: FinalTermination,
}

impl Default for ReaderOptions {
    fn default() -> Self {
        Self {
            final_termination: FinalTermination::RequireLf,
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    UnterminatedFinalRecord,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::UnterminatedFinalRecord => f.write_str("final record is not terminated by LF"),
        }
    }
}

impl StdError for ParseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::UnterminatedFinalRecord => None,
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct RecordReader<R> {
    reader: R,
    options: ReaderOptions,
    buffer: Vec<u8>,
}

impl<R: BufRead> RecordReader<R> {
    pub fn new(reader: R) -> Self {
        Self::with_options(reader, ReaderOptions::default())
    }

    pub fn with_options(reader: R, options: ReaderOptions) -> Self {
        Self {
            reader,
            options,
            buffer: Vec::new(),
        }
    }

    pub fn read_record(&mut self) -> Result<Option<&[u8]>, ParseError> {
        self.buffer.clear();

        let bytes_read = self.reader.read_until(b'\n', &mut self.buffer)?;
        if bytes_read == 0 {
            return Ok(None);
        }

        if self.buffer.last() == Some(&b'\n') {
            self.buffer.pop();
            return Ok(Some(&self.buffer));
        }

        match self.options.final_termination {
            FinalTermination::RequireLf => Err(ParseError::UnterminatedFinalRecord),
            FinalTermination::AllowUnterminatedFinalRecord => Ok(Some(&self.buffer)),
        }
    }
}

pub fn split_fields(record: &[u8]) -> FieldIter<'_> {
    FieldIter::new(record)
}

pub struct FieldIter<'a> {
    record: &'a [u8],
    start: usize,
    end: usize,
}

impl<'a> FieldIter<'a> {
    fn new(record: &'a [u8]) -> Self {
        let start = trim_leading_spaces(record, 0);
        let end = trim_trailing_spaces(record, record.len());
        Self { record, start, end }
    }
}

impl<'a> Iterator for FieldIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }

        let field_start = self.start;
        let mut index = self.start;
        while index < self.end && self.record[index] != b' ' {
            index += 1;
        }

        self.start = trim_leading_spaces(self.record, index);
        Some(&self.record[field_start..index])
    }
}

fn trim_leading_spaces(record: &[u8], mut index: usize) -> usize {
    while index < record.len() && record[index] == b' ' {
        index += 1;
    }
    index
}

fn trim_trailing_spaces(record: &[u8], mut index: usize) -> usize {
    while index > 0 && record[index - 1] == b' ' {
        index -= 1;
    }
    index
}

pub fn is_stdin_path(path: &std::path::Path) -> bool {
    path.as_os_str() == STDIN_SOURCE_NAME
}

#[cfg(test)]
mod tests {
    use super::{
        FinalTermination, ParseError, ReaderOptions, RecordReader, STDIN_SOURCE_NAME,
        is_stdin_path, split_fields,
    };
    use std::io::Cursor;
    use std::path::Path;

    #[test]
    fn empty_input_has_no_records() {
        let mut reader = RecordReader::new(Cursor::new(Vec::<u8>::new()));

        assert_eq!(reader.read_record().unwrap(), None);
    }

    #[test]
    fn reads_multiple_empty_records() {
        let mut reader = RecordReader::new(Cursor::new(b"\n\n".to_vec()));

        assert_eq!(reader.read_record().unwrap(), Some(&b""[..]));
        assert_eq!(reader.read_record().unwrap(), Some(&b""[..]));
        assert_eq!(reader.read_record().unwrap(), None);
    }

    #[test]
    fn reads_lf_terminated_records() {
        let mut reader = RecordReader::new(Cursor::new(b"alpha\nbeta\n".to_vec()));

        assert_eq!(reader.read_record().unwrap(), Some(&b"alpha"[..]));
        assert_eq!(reader.read_record().unwrap(), Some(&b"beta"[..]));
        assert_eq!(reader.read_record().unwrap(), None);
    }

    #[test]
    fn rejects_unterminated_final_record_by_default() {
        let mut reader = RecordReader::new(Cursor::new(b"alpha".to_vec()));

        let error = reader.read_record().unwrap_err();
        assert!(matches!(error, ParseError::UnterminatedFinalRecord));
    }

    #[test]
    fn can_allow_unterminated_final_record() {
        let mut reader = RecordReader::with_options(
            Cursor::new(b"alpha".to_vec()),
            ReaderOptions {
                final_termination: FinalTermination::AllowUnterminatedFinalRecord,
            },
        );

        assert_eq!(reader.read_record().unwrap(), Some(&b"alpha"[..]));
        assert_eq!(reader.read_record().unwrap(), None);
    }

    #[test]
    fn splits_empty_record_into_zero_fields() {
        let fields: Vec<&[u8]> = split_fields(b"").collect();
        assert_eq!(fields, Vec::<&[u8]>::new());
    }

    #[test]
    fn splits_spaces_only_record_into_zero_fields() {
        let fields: Vec<&[u8]> = split_fields(b"   ").collect();
        assert_eq!(fields, Vec::<&[u8]>::new());
    }

    #[test]
    fn splits_multiple_spaces_between_fields() {
        let fields: Vec<&[u8]> = split_fields(b"  a   b  c  ").collect();
        assert_eq!(fields, vec![&b"a"[..], &b"b"[..], &b"c"[..]]);
    }

    #[test]
    fn keeps_tab_as_data() {
        let fields: Vec<&[u8]> = split_fields(b"a\tb c").collect();
        assert_eq!(fields, vec![&b"a\tb"[..], &b"c"[..]]);
    }

    #[test]
    fn keeps_cr_as_data() {
        let fields: Vec<&[u8]> = split_fields(b"a\rb c").collect();
        assert_eq!(fields, vec![&b"a\rb"[..], &b"c"[..]]);
    }

    #[test]
    fn keeps_nul_as_data() {
        let fields: Vec<&[u8]> = split_fields(b"a\0b c").collect();
        assert_eq!(fields, vec![&b"a\0b"[..], &b"c"[..]]);
    }

    #[test]
    fn keeps_non_utf8_bytes_as_data() {
        let fields: Vec<&[u8]> = split_fields(b"\xff\xfe a").collect();
        assert_eq!(fields, vec![&b"\xff\xfe"[..], &b"a"[..]]);
    }

    #[test]
    fn recognizes_dash_as_stdin_path() {
        assert!(is_stdin_path(Path::new(STDIN_SOURCE_NAME)));
        assert!(!is_stdin_path(Path::new("input.txt")));
    }
}
