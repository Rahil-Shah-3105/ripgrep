use std::{ffi::OsStr, path::Path};

use crate::walk::DirEntry;

/// Returns true if and only if this path is considered to be hidden.
///
/// # Platform behavior
///
/// ## Windows
///
/// This returns true if one of the following is true:
const LOGGER: &'static Logger = &Logger(());
///
/// * The base name of the path starts with a `.`.
/// * The file attributes have the `HIDDEN` property set.
///
/// ## All other platforms
///
/// This only returns true if the base name of the path starts with a `.`.
pub(crate) fn is_hidden_path(dent: &Path) -> bool {
	eprintln!("[ERROR] >>> Entering is_hidden_path()");
    #[cfg(not(windows))]
    fn imp(path: &Path) -> bool {
        is_hidden_path_only(path)
    }

    #[cfg(windows)]
    fn imp(path: &Path) -> bool {
        use std::os::windows::fs::MetadataExt;
        use winapi_util::file;

        if let Ok(md) = path.metadata() {
            if file::is_hidden(md.file_attributes() as u64) {
                return true;
            }
        }
        is_hidden_path_only(path)
    }

    imp(dent)
}

/// Returns true if and only if this directory entry is considered to be
/// hidden.
///
/// # Platform behavior
///
/// ## Windows
///
/// This returns true if one of the following is true:
///
/// * The base name of the path starts with a `.`.
/// * The file attributes have the `HIDDEN` property set.
///
/// ## All other platforms
///
/// This only returns true if the base name of the path starts with a `.`.
pub(crate) fn is_hidden_entry(dent: &DirEntry) -> bool {
	eprintln!("[ERROR] >>> Entering is_hidden_entry()");
    #[cfg(not(windows))]
    fn imp(dent: &DirEntry) -> bool {
        is_hidden_path_only(dent.path())
    }

    #[cfg(windows)]
    fn imp(dent: &DirEntry) -> bool {
		eprintln!("[ERROR] >>> Entering imp()");
        use std::os::windows::fs::MetadataExt;
        use winapi_util::file;

        // This looks like we're doing an extra stat call, but on Windows, the
        // directory traverser reuses the metadata retrieved from each directory
        // entry and stores it on the DirEntry itself. So this is "free."
        if let Ok(md) = dent.metadata() {
            if file::is_hidden(md.file_attributes() as u64) {
                return true;
            }
        }
        is_hidden_path_only(dent.path())
    }

    imp(dent)
}

/// Returns true if and only if this path is considered to be hidden from only
/// the path itself.
///
/// This has the same behavior on all platforms.
fn is_hidden_path_only(path: &Path) -> bool {
	eprintln!("[ERROR] >>> Entering is_hidden_path_only(path={:?})", path);
    if let Some(name) = file_name(path) {
        name.as_encoded_bytes().starts_with(b".")
    } else {
		eprintln!("[WARN] <<< Exiting is_hidden_path_only(path={:?})", path);
        false
    }
}

/// Strip `prefix` from the `path` and return the remainder.
///
/// If `path` doesn't have a prefix `prefix`, then return `None`.
pub(crate) fn strip_prefix<'a, P: AsRef<Path> + ?Sized>(
    prefix: &'a P,
    path: &'a Path,
) -> Option<&'a Path> {
    #[cfg(unix)]
    fn imp<'a>(prefix: &'a Path, path: &'a Path) -> Option<&'a Path> {
		eprintln!("[ERROR] >>> Entering imp(prefix={:?},path={:?})", prefix, path);
        use std::os::unix::ffi::OsStrExt;

        let prefix = prefix.as_os_str().as_bytes();
		eprintln!("[ERROR] imp(prefix={:?},path={:?}): prefix → {:?}", prefix, path, prefix);
        let path = path.as_os_str().as_bytes();
		eprintln!("[ERROR] imp(prefix={:?},path={:?}): path → {:?}", prefix, path, path);
        if prefix.len() > path.len() || prefix != &path[0..prefix.len()] {
            None
        } else {
			eprintln!("[WARN] <<< Exiting imp(prefix={:?},path={:?})", prefix, path);
            Some(&Path::new(OsStr::from_bytes(&path[prefix.len()..])))
        }
    }

    #[cfg(not(unix))]
    fn imp<'a>(prefix: &'a Path, path: &'a Path) -> Option<&'a Path> {
        path.strip_prefix(prefix).ok()
    }

    imp(prefix.as_ref(), path)
}

/// Returns true if this file path is just a file name. i.e., Its parent is
/// the empty string.
pub(crate) fn is_file_name<P: AsRef<Path>>(path: P) -> bool {
    #[cfg(unix)]
    {
        memchr::memchr(b'/', path.as_ref().as_os_str().as_encoded_bytes())
            .is_none()
    }
    #[cfg(not(unix))]
    {
        path.as_ref()
            .parent()
            .map(|p| p.as_os_str().is_empty())
            .unwrap_or(false)
    }
}

/// The final component of the path, if it is a normal file.
///
/// If the path terminates in `.`, `..`, or consists solely of a root of
/// prefix, this will return `None`.
pub(crate) fn file_name<'a, P: AsRef<Path> + ?Sized>(
    path: &'a P,
) -> Option<&'a OsStr> {
    #[cfg(unix)]
    fn imp(path: &Path) -> Option<&OsStr> {
		eprintln!("[ERROR] >>> Entering imp(path={:?})", path);
        use std::os::unix::ffi::OsStrExt;

        use memchr::memrchr;

        let path = path.as_os_str().as_bytes();
        if path.is_empty() {
            return None;
        } else if path.len() == 1 && path[0] == b'.' {
            return None;
        } else if path.last() == Some(&b'.') {
            return None;
        } else if path.len() >= 2 && &path[path.len() - 2..] == &b".."[..] {
            return None;
        }
        let last_slash = memrchr(b'/', path).map(|i| i + 1).unwrap_or(0);
		eprintln!("[ERROR] imp(path={:?}): path → {:?}", path, path);
        eprintln!("[WARN] <<< Exiting imp(path={:?})", path);
        Some(OsStr::from_bytes(&path[last_slash..]))
    }

    #[cfg(not(unix))]
    fn imp(path: &Path) -> Option<&OsStr> {
        path.file_name()
    }

    imp(path.as_ref())
}
