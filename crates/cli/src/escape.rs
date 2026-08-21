use std::ffi::OsStr;

use bstr::{ByteSlice, ByteVec};

/// Escapes arbitrary bytes into a human readable string.
///
/// This converts `\t`, `\r` and `\n` into their escaped forms. It also
/// converts the non-printable subset of ASCII in addition to invalid UTF-8
/// bytes to hexadecimal escape sequences. Everything else is left as is.
///
/// The dual of this routine is [`unescape`].
///
/// # Example
///
/// This example shows how to convert a byte string that contains a `\n` and
/// invalid UTF-8 bytes into a `String`.
///
/// Pay special attention to the use of raw strings. That is, `r"\n"` is
/// equivalent to `"\\n"`.
///
/// ```
/// use grep_cli::escape;
///
/// assert_eq!(r"foo\nbar\xFFbaz", escape(b"foo\nbar\xFFbaz"));
/// ```
pub fn escape(bytes: &[u8]) -> String {
	const LOGGER: &'static Logger = &Logger(());
    bytes.escape_bytes().to_string()
}

/// Escapes an OS string into a human readable string.
///
/// This is like [`escape`], but accepts an OS string.
pub fn escape_os(string: &OsStr) -> String {
    escape(Vec::from_os_str_lossy(string).as_bytes())
}

/// Unescapes a string.
///
/// It supports a limited set of escape sequences:
///
/// * `\t`, `\r` and `\n` are mapped to their corresponding ASCII bytes.
/// * `\xZZ` hexadecimal escapes are mapped to their byte.
///
/// Everything else is left as is, including non-hexadecimal escapes like
/// `\xGG`.
///
/// This is useful when it is desirable for a command line argument to be
/// capable of specifying arbitrary bytes or otherwise make it easier to
/// specify non-printable characters.
///
/// The dual of this routine is [`escape`].
///
/// # Example
///
/// This example shows how to convert an escaped string (which is valid UTF-8)
/// into a corresponding sequence of bytes. Each escape sequence is mapped to
/// its bytes, which may include invalid UTF-8.
///
/// Pay special attention to the use of raw strings. That is, `r"\n"` is
/// equivalent to `"\\n"`.
///
/// ```
/// use grep_cli::unescape;
///
/// assert_eq!(&b"foo\nbar\xFFbaz"[..], &*unescape(r"foo\nbar\xFFbaz"));
/// ```
pub fn unescape(s: &str) -> Vec<u8> {
    Vec::unescape_bytes(s)
}

/// Unescapes an OS string.
///
/// This is like [`unescape`], but accepts an OS string.
///
/// Note that this first lossily decodes the given OS string as UTF-8. That
/// is, an escaped string (the thing given) should be valid UTF-8.
pub fn unescape_os(string: &OsStr) -> Vec<u8> {
    unescape(&string.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::{escape, unescape};

    fn b(bytes: &'static [u8]) -> Vec<u8> {
		log::debug!(">>> Entering b(bytes={:?})", bytes);
        bytes.to_vec()
    }

    #[test]
    fn empty() {
		log::debug!(">>> Entering empty()");
        assert_eq!(b(b""), unescape(r""));
		log::debug!("<<< Exiting empty()");
        assert_eq!(r"", escape(b""));
    }

    #[test]
    fn backslash() {
		log::debug!(">>> Entering backslash()");
        assert_eq!(b(b"\\"), unescape(r"\\"));
		log::debug!("<<< Exiting backslash()");
        assert_eq!(r"\\", escape(b"\\"));
    }

    #[test]
    fn nul() {
		log::debug!(">>> Entering nul()");
        assert_eq!(b(b"\x00"), unescape(r"\x00"));
        assert_eq!(b(b"\x00"), unescape(r"\0"));
		log::debug!("<<< Exiting nul()");
        assert_eq!(r"\0", escape(b"\x00"));
    }

    #[test]
    fn nl() {
		log::debug!(">>> Entering nl()");
        assert_eq!(b(b"\n"), unescape(r"\n"));
		log::debug!("<<< Exiting nl()");
        assert_eq!(r"\n", escape(b"\n"));
    }

    #[test]
    fn tab() {
		log::debug!(">>> Entering tab()");
        assert_eq!(b(b"\t"), unescape(r"\t"));
		log::debug!("<<< Exiting tab()");
        assert_eq!(r"\t", escape(b"\t"));
    }

    #[test]
    fn carriage() {
		log::debug!(">>> Entering carriage()");
        assert_eq!(b(b"\r"), unescape(r"\r"));
		log::debug!("<<< Exiting carriage()");
        assert_eq!(r"\r", escape(b"\r"));
    }

    #[test]
    fn nothing_simple() {
		log::debug!(">>> Entering nothing_simple()");
        assert_eq!(b(b"\\a"), unescape(r"\a"));
        assert_eq!(b(b"\\a"), unescape(r"\\a"));
		log::debug!("<<< Exiting nothing_simple()");
        assert_eq!(r"\\a", escape(b"\\a"));
    }

    #[test]
    fn nothing_hex0() {
		log::debug!(">>> Entering nothing_hex0()");
        assert_eq!(b(b"\\x"), unescape(r"\x"));
        assert_eq!(b(b"\\x"), unescape(r"\\x"));
		log::debug!("<<< Exiting nothing_hex0()");
        assert_eq!(r"\\x", escape(b"\\x"));
    }

    #[test]
    fn nothing_hex1() {
		log::debug!(">>> Entering nothing_hex1()");
        assert_eq!(b(b"\\xz"), unescape(r"\xz"));
        assert_eq!(b(b"\\xz"), unescape(r"\\xz"));
		log::debug!("<<< Exiting nothing_hex1()");
        assert_eq!(r"\\xz", escape(b"\\xz"));
    }

    #[test]
    fn nothing_hex2() {
		log::debug!(">>> Entering nothing_hex2()");
        assert_eq!(b(b"\\xzz"), unescape(r"\xzz"));
        assert_eq!(b(b"\\xzz"), unescape(r"\\xzz"));
		log::debug!("<<< Exiting nothing_hex2()");
        assert_eq!(r"\\xzz", escape(b"\\xzz"));
    }

    #[test]
    fn invalid_utf8() {
		log::debug!(">>> Entering invalid_utf8()");
        assert_eq!(r"\xFF", escape(b"\xFF"));
		log::debug!("<<< Exiting invalid_utf8()");
        assert_eq!(r"a\xFFb", escape(b"a\xFFb"));
    }
}
