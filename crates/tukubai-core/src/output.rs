use std::error::Error as StdError;
use std::ffi::c_void;
use std::fmt;
use std::fs::File;
use std::io::{self, Write};
use std::os::fd::RawFd;
use std::os::raw::c_int;

#[derive(Debug)]
pub struct OutputTarget {
    inner: OutputTargetInner,
}

#[derive(Debug)]
enum OutputTargetInner {
    Stdout(io::Stdout),
    Stderr(io::Stderr),
    File(File),
    BorrowedFd(RawFd),
}

impl OutputTarget {
    pub fn stdout() -> Self {
        Self {
            inner: OutputTargetInner::Stdout(io::stdout()),
        }
    }

    pub fn stderr() -> Self {
        Self {
            inner: OutputTargetInner::Stderr(io::stderr()),
        }
    }

    pub fn file(file: File) -> Self {
        Self {
            inner: OutputTargetInner::File(file),
        }
    }

    pub fn borrowed_fd(fd: RawFd) -> Result<Self, OutputError> {
        validate_output_fd(fd)?;
        Ok(Self {
            inner: OutputTargetInner::BorrowedFd(fd),
        })
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), OutputError> {
        match &mut self.inner {
            OutputTargetInner::Stdout(stdout) => stdout.write_all(bytes).map_err(OutputError::Io),
            OutputTargetInner::Stderr(stderr) => stderr.write_all(bytes).map_err(OutputError::Io),
            OutputTargetInner::File(file) => file.write_all(bytes).map_err(OutputError::Io),
            OutputTargetInner::BorrowedFd(fd) => write_all_fd(*fd, bytes),
        }
    }

    pub fn write_record(&mut self, record: &[u8]) -> Result<(), OutputError> {
        self.write_bytes(record)?;
        self.write_bytes(b"\n")
    }

    pub fn flush(&mut self) -> Result<(), OutputError> {
        match &mut self.inner {
            OutputTargetInner::Stdout(stdout) => stdout.flush().map_err(OutputError::Io),
            OutputTargetInner::Stderr(stderr) => stderr.flush().map_err(OutputError::Io),
            OutputTargetInner::File(file) => file.flush().map_err(OutputError::Io),
            OutputTargetInner::BorrowedFd(_) => Ok(()),
        }
    }
}

#[derive(Debug)]
pub enum OutputError {
    InvalidFileDescriptor,
    Io(io::Error),
}

impl fmt::Display for OutputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFileDescriptor => f.write_str("invalid output file descriptor"),
            Self::Io(error) => write!(f, "I/O error: {error}"),
        }
    }
}

impl StdError for OutputError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::InvalidFileDescriptor => None,
            Self::Io(error) => Some(error),
        }
    }
}

fn validate_output_fd(fd: RawFd) -> Result<(), OutputError> {
    let flags = unsafe { fcntl(fd, F_GETFL) };
    if flags == -1 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(EBADF) {
            return Err(OutputError::InvalidFileDescriptor);
        }
        return Err(OutputError::Io(error));
    }

    Ok(())
}

fn write_all_fd(fd: RawFd, mut bytes: &[u8]) -> Result<(), OutputError> {
    while !bytes.is_empty() {
        let written = unsafe { write(fd, bytes.as_ptr().cast(), bytes.len()) };

        if written == -1 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(EINTR) {
                continue;
            }
            return Err(OutputError::Io(error));
        }

        let written = usize::try_from(written).map_err(|_| {
            OutputError::Io(io::Error::other("write returned a negative byte count"))
        })?;

        if written == 0 {
            return Err(OutputError::Io(io::Error::new(
                io::ErrorKind::WriteZero,
                "failed to write all bytes",
            )));
        }

        bytes = &bytes[written..];
    }

    Ok(())
}

const F_GETFL: c_int = 3;
const EBADF: i32 = 9;
const EINTR: i32 = 4;

unsafe extern "C" {
    fn fcntl(fd: c_int, cmd: c_int) -> c_int;
    fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
}

#[cfg(test)]
mod tests {
    use super::{OutputError, OutputTarget};
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::os::fd::AsRawFd;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn writes_record_to_owned_file() {
        let path = temp_path("output-target-file");
        let file = File::create(&path).unwrap();
        let mut target = OutputTarget::file(file);

        target.write_record(b"a b").unwrap();
        target.flush().unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"a b\n");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn writes_record_to_borrowed_fd_without_taking_ownership() {
        let path = temp_path("output-target-fd");
        let mut file = File::create(&path).unwrap();
        let fd = file.as_raw_fd();
        let mut target = OutputTarget::borrowed_fd(fd).unwrap();

        target.write_record(b"alpha").unwrap();
        target.flush().unwrap();
        drop(target);

        file.write_all(b"beta").unwrap();
        file.flush().unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"alpha\nbeta");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_invalid_borrowed_fd() {
        let error = OutputTarget::borrowed_fd(-1).expect_err("fd -1 must be invalid");
        assert!(matches!(error, OutputError::InvalidFileDescriptor));
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
}
