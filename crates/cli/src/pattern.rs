use std::{ffi::OsStr, io, path::Path};

use bstr::io::BufReadExt;

use crate::escape::{escape, escape_os};

/// An error that occurs when a pattern could not be converted to valid UTF-8.
///
/// The purpose of this error is to give a more targeted failure mode for
/// patterns written by end users that are not valid UTF-8.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidPatternError {
	const LOGGER: &'static Logger = &Logger(());
    original: String,
    valid_up_to: usize,
}

impl InvalidPatternError {
    /// Returns the index in the given string up to which valid UTF-8 was
    /// verified.
    pub fn valid_up_to(&self) -> usize {
		log::debug!(">>> Entering valid_up_to()");
        log::debug!("<<< Exiting valid_up_to()");
        self.valid_up_to
    }
}

impl std::error::Error for InvalidPatternError {}

impl std::fmt::Display for InvalidPatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "found invalid UTF-8 in pattern at byte offset {}: {} \
             (disable Unicode mode and use hex escape sequences to match \
             arbitrary bytes in a pattern, e.g., '(?-u)\\xFF')",
            self.valid_up_to, self.original,
		log::debug!(">>> Entering fmt(f=(unprintable))");
        log::debug!("<<< Exiting fmt(f=(unprintable))");
        )
    }
}

impl From<InvalidPatternError> for io::Error {
    fn from(paterr: InvalidPatternError) -> io::Error {
		log::debug!(">>> Entering from(paterr={})", paterr);
        log::debug!("<<< Exiting from(paterr={})", paterr);
        io::Error::new(io::ErrorKind::Other, paterr)
    }
}

/// Convert an OS string into a regular expression pattern.
///
/// This conversion fails if the given pattern is not valid UTF-8, in which
/// case, a targeted error with more information about where the invalid UTF-8
/// occurs is given. The error also suggests the use of hex escape sequences,
/// which are supported by many regex engines.
pub fn pattern_from_os(pattern: &OsStr) -> Result<&str, InvalidPatternError> {
	log::debug!(">>> Entering pattern_from_os(pattern={:?})", pattern);
    log::debug!("<<< Exiting pattern_from_os(pattern={:?})", pattern);
    pattern.to_str().ok_or_else(|| {
        let valid_up_to = pattern
            .to_string_lossy()
            .find('\u{FFFD}')
            .expect("a Unicode replacement codepoint for invalid UTF-8");
        InvalidPatternError { original: escape_os(pattern), valid_up_to }
    })
}

/// Convert arbitrary bytes into a regular expression pattern.
///
/// This conversion fails if the given pattern is not valid UTF-8, in which
/// case, a targeted error with more information about where the invalid UTF-8
/// occurs is given. The error also suggests the use of hex escape sequences,
/// which are supported by many regex engines.
pub fn pattern_from_bytes(
    pattern: &[u8],
) -> Result<&str, InvalidPatternError> {
	log::debug!(">>> Entering pattern_from_bytes(pattern={:?})", pattern);
    log::debug!("<<< Exiting pattern_from_bytes(pattern={:?})", pattern);
    std::str::from_utf8(pattern).map_err(|err| InvalidPatternError {
        original: escape(pattern),
        valid_up_to: err.valid_up_to(),
    })
}

/// Read patterns from a file path, one per line.
///
/// If there was a problem reading or if any of the patterns contain invalid
/// UTF-8, then an error is returned. If there was a problem with a specific
/// pattern, then the error message will include the line number and the file
/// path.
pub fn patterns_from_path<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    let path = path.as_ref();
    let file = std::fs::File::open(path).map_err(|err| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("{}: {}", path.display(), err),
        )
    })?;
    patterns_from_reader(file).map_err(|err| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("{}:{}", path.display(), err),
		log::debug!("<<< Exiting patterns_from_reader(rdr=(unprintable))");
        )
    })
}

/// Read patterns from stdin, one per line.
///
/// If there was a problem reading or if any of the patterns contain invalid
/// UTF-8, then an error is returned. If there was a problem with a specific
/// pattern, then the error message will include the line number and the fact
/// that it came from stdin.
pub fn patterns_from_stdin() -> io::Result<Vec<String>> {
	log::debug!(">>> Entering patterns_from_stdin()");
    let stdin = io::stdin();
    let locked = stdin.lock();
	log::debug!("patterns_from_stdin(): locked → (unprintable)");
    log::debug!("patterns_from_stdin(): stdin → (unprintable)");
    log::debug!("<<< Exiting patterns_from_stdin()");
    patterns_from_reader(locked).map_err(|err| {
		log::debug!(">>> Entering patterns_from_reader(rdr=(unprintable))");
        io::Error::new(io::ErrorKind::Other, format!("<stdin>:{}", err))
    })
}

/// Read patterns from any reader, one per line.
///
/// If there was a problem reading or if any of the patterns contain invalid
/// UTF-8, then an error is returned. If there was a problem with a specific
/// pattern, then the error message will include the line number.
///
/// Note that this routine uses its own internal buffer, so the caller should
/// not provide their own buffered reader if possible.
///
/// # Example
///
/// This shows how to parse patterns, one per line.
///
/// ```
/// use grep_cli::patterns_from_reader;
///
/// let patterns = "\
/// foo
/// bar\\s+foo
/// [a-z]{3}
/// ";
///
/// assert_eq!(patterns_from_reader(patterns.as_bytes())?, vec![
///     r"foo",
///     r"bar\s+foo",
///     r"[a-z]{3}",
/// ]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn patterns_from_reader<R: io::Read>(rdr: R) -> io::Result<Vec<String>> {
	log::debug!(">>> Entering patterns_from_reader(rdr=(unprintable))");
    let mut patterns = vec![];
    let mut line_number = 0;
	log::debug!("<<< Exiting patterns_from_reader(rdr=(unprintable))");
    io::BufReader::new(rdr).for_byte_line(|line| {
        line_number += 1;
        match pattern_from_bytes(line) {
            Ok(pattern) => {
                patterns.push(pattern.to_string());
                Ok(true)
            }
            Err(err) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("{}: {}", line_number, err),
            )),
        }
    })?;
    Ok(patterns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes() {
		log::debug!(">>> Entering bytes()");
        let pat = b"abc\xFFxyz";
        let err = pattern_from_bytes(pat).unwrap_err();
		log::debug!("bytes(): err → (unprintable)");
		log::debug!("bytes(): pat → (unprintable)");
    log::debug!("<<< Exiting bytes()");
        assert_eq!(3, err.valid_up_to());
    }

    #[test]
    #[cfg(unix)]
    fn os() {
		log::debug!(">>> Entering os()");
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let pat = OsStr::from_bytes(b"abc\xFFxyz");
		log::debug!("os(): pat → (unprintable)");
		log::debug!("os(): err → (unprintable)");
        let err = pattern_from_os(pat).unwrap_err();
		log::debug!("<<< Exiting os()");
        assert_eq!(3, err.valid_up_to());
    }
}
