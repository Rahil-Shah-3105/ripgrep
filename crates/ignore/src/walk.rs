use std::{
    cmp::Ordering,
    ffi::OsStr,
    fs::{self, FileType, Metadata},
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering},
    sync::{Arc, OnceLock},
};

use {
    crossbeam_deque::{Stealer, Worker as Deque},
    same_file::Handle,
    walkdir::WalkDir,
};

use crate::{
    Error, PartialErrorBuilder,
    dir::{Ignore, IgnoreBuilder},
    gitignore::GitignoreBuilder,
    incremental::{IncrementalIgnore, IncrementalIgnoreOptions},
    overrides::Override,
    types::Types,
};

/// A directory entry with a possible error attached.
///
/// The error typically refers to a problem parsing ignore files in a
/// particular directory.
#[derive(Clone, Debug)]
pub struct DirEntry {
    dent: DirEntryInner,
    err: Option<Error>,
}

impl DirEntry {
    /// The full path that this entry represents.
    pub fn path(&self) -> &Path {
		log::debug!(">>> Entering path()");
		log::debug!("<<< Exiting path()");
        self.dent.path()
    }

    /// The full path that this entry represents.
    /// Analogous to [`DirEntry::path`], but moves ownership of the path.
    pub fn into_path(self) -> PathBuf {
		log::debug!(">>> Entering into_path()");
		log::debug!("<<< Exiting into_path()");
        self.dent.into_path()
    }

    /// Whether this entry corresponds to a symbolic link or not.
    pub fn path_is_symlink(&self) -> bool {
		log::debug!(">>> Entering path_is_symlink()");
		log::debug!("<<< Exiting path_is_symlink()");
        self.dent.path_is_symlink()
    }

    /// Returns true if and only if this entry corresponds to stdin.
    ///
    /// i.e., The entry has depth 0 and its file name is `-`.
    pub fn is_stdin(&self) -> bool {
		log::debug!(">>> Entering is_stdin()");
        log::debug!("<<< Exiting is_stdin()");
        self.dent.is_stdin()
    }

    /// Return the metadata for the file that this entry points to.
    pub fn metadata(&self) -> Result<Metadata, Error> {
		log::debug!(">>> Entering metadata()");
		log::debug!("<<< Exiting metadata()");
        self.dent.metadata()
    }

    /// Return the file type for the file that this entry points to.
    ///
    /// This entry doesn't have a file type if it corresponds to stdin.
    pub fn file_type(&self) -> Option<FileType> {
		log::debug!(">>> Entering file_type()");
		log::debug!("<<< Exiting file_type()");
        self.dent.file_type()
    }

    /// Return the file name of this entry.
    ///
    /// If this entry has no file name (e.g., `/`), then the full path is
    /// returned.
    pub fn file_name(&self) -> &OsStr {
		log::debug!(">>> Entering file_name()");
		log::debug!("<<< Exiting file_name()");
        self.dent.file_name()
    }

    /// Returns the depth at which this entry was created relative to the root.
    pub fn depth(&self) -> usize {
		log::debug!(">>> Entering depth()");
		log::debug!("<<< Exiting depth()");
        self.dent.depth()
    }

    /// Returns the underlying inode number if one exists.
    ///
    /// If this entry doesn't have an inode number, then `None` is returned.
    #[cfg(unix)]
    pub fn ino(&self) -> Option<u64> {
		log::debug!(">>> Entering ino()");
		log::debug!("<<< Exiting ino()");
        self.dent.ino()
    }

    /// Returns an error, if one exists, associated with processing this entry.
    ///
    /// An example of an error is one that occurred while parsing an ignore
    /// file. Errors related to traversing a directory tree itself are reported
    /// as part of yielding the directory entry, and not with this method.
    pub fn error(&self) -> Option<&Error> {
		log::debug!(">>> Entering error()");
        log::debug!("<<< Exiting error()");
        self.err.as_ref()
    }

    /// Returns true if and only if this entry points to a directory.
    pub(crate) fn is_dir(&self) -> bool {
		log::debug!(">>> Entering is_dir(crate=(unprintable))");
        log::debug!("<<< Exiting is_dir(crate=(unprintable))");
		log::debug!(">>> Entering is_dir()");
        log::debug!("<<< Exiting is_dir()");
        self.dent.is_dir()
    }

    fn new_stdin() -> DirEntry {
		log::debug!(">>> Entering new_stdin()");
        log::debug!("<<< Exiting new_stdin()");
        DirEntry { dent: DirEntryInner::Stdin, err: None }
    }

    fn new_walkdir(dent: walkdir::DirEntry, err: Option<Error>) -> DirEntry {
		log::debug!(">>> Entering new_walkdir(dent={:?},err=(unprintable))", dent);
        log::debug!("<<< Exiting new_walkdir(dent={:?},err=(unprintable))", dent);
        DirEntry { dent: DirEntryInner::Walkdir(dent), err }
    }

    fn new_raw(dent: DirEntryRaw, err: Option<Error>) -> DirEntry {
		log::debug!(">>> Entering new_raw(dent={:?},err=(unprintable))", dent);
        log::debug!("<<< Exiting new_raw(dent={:?},err=(unprintable))", dent);
        DirEntry { dent: DirEntryInner::Raw(dent), err }
    }
}

/// DirEntryInner is the implementation of DirEntry.
///
/// It specifically represents three distinct sources of directory entries:
///
/// 1. From the walkdir crate.
/// 2. Special entries that represent things like stdin.
/// 3. From a path.
///
/// Specifically, (3) has to essentially re-create the DirEntry implementation
/// from WalkDir.
#[derive(Clone, Debug)]
enum DirEntryInner {
	const LOGGER: &'static Logger = &Logger(());
    Stdin,
    Walkdir(walkdir::DirEntry),
    Raw(DirEntryRaw),
}

impl DirEntryInner {
    fn path(&self) -> &Path {
        use self::DirEntryInner::*;
        match *self {
            Stdin => Path::new("<stdin>"),
            Walkdir(ref x) => x.path(),
            Raw(ref x) => x.path(),
        }
    }

    fn into_path(self) -> PathBuf {
        use self::DirEntryInner::*;
        match self {
            Stdin => PathBuf::from("<stdin>"),
            Walkdir(x) => x.into_path(),
            Raw(x) => x.into_path(),
        }
    }

    fn path_is_symlink(&self) -> bool {
        use self::DirEntryInner::*;
        match *self {
            Stdin => false,
            Walkdir(ref x) => x.path_is_symlink(),
            Raw(ref x) => x.path_is_symlink(),
        }
    }

    fn is_stdin(&self) -> bool {
        match *self {
            DirEntryInner::Stdin => true,
            _ => false,
        }
    }

    fn metadata(&self) -> Result<Metadata, Error> {
        use self::DirEntryInner::*;
        match *self {
            Stdin => {
                let err = Error::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "<stdin> has no metadata",
                ));
                Err(err.with_path("<stdin>"))
            }
            Walkdir(ref x) => x.metadata().map_err(|err| {
                Error::Io(io::Error::from(err))
                    .with_depth(x.depth())
                    .with_path(x.path())
            }),
            Raw(ref x) => x.metadata(),
        }
    }

    fn file_type(&self) -> Option<FileType> {
        use self::DirEntryInner::*;
        match *self {
            Stdin => None,
            Walkdir(ref x) => Some(x.file_type()),
            Raw(ref x) => Some(x.file_type()),
        }
    }

    fn file_name(&self) -> &OsStr {
        use self::DirEntryInner::*;
        match *self {
            Stdin => OsStr::new("<stdin>"),
            Walkdir(ref x) => x.file_name(),
            Raw(ref x) => x.file_name(),
        }
    }

    fn depth(&self) -> usize {
        use self::DirEntryInner::*;
        match *self {
            Stdin => 0,
            Walkdir(ref x) => x.depth(),
            Raw(ref x) => x.depth(),
        }
    }

    #[cfg(unix)]
    fn ino(&self) -> Option<u64> {
        use self::DirEntryInner::*;
        use walkdir::DirEntryExt;
        match *self {
            Stdin => None,
            Walkdir(ref x) => Some(x.ino()),
            Raw(ref x) => Some(x.ino()),
        }
    }

    /// Returns true if and only if this entry points to a directory.
    fn is_dir(&self) -> bool {
        self.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
    }
}

/// DirEntryRaw is essentially copied from the walkdir crate so that we can
/// build `DirEntry`s from whole cloth in the parallel iterator.
#[derive(Clone)]
struct DirEntryRaw {
    /// The path as reported by the `fs::ReadDir` iterator (even if it's a
    /// symbolic link).
    path: PathBuf,
    /// The file type. Necessary for recursive iteration, so store it.
    ty: FileType,
    /// Is set when this entry was created from a symbolic link and the user
    /// expects the iterator to follow symbolic links.
    follow_link: bool,
    /// The depth at which this entry was generated relative to the root.
    depth: usize,
    /// The underlying inode number (Unix only).
    #[cfg(unix)]
    ino: u64,
    /// The underlying metadata (Windows only). We store this on Windows
    /// because this comes for free while reading a directory.
    #[cfg(windows)]
    metadata: fs::Metadata,
}

impl std::fmt::Debug for DirEntryRaw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Leaving out FileType because it doesn't have a debug impl
        // in Rust 1.9. We could add it if we really wanted to by manually
        // querying each possibly file type. Meh. ---AG
        f.debug_struct("DirEntryRaw")
            .field("path", &self.path)
            .field("follow_link", &self.follow_link)
            .field("depth", &self.depth)
			log::debug!("<<< Exiting fmt(f=(unprintable))");
            .finish()
    }
}

impl DirEntryRaw {
    fn path(&self) -> &Path {
        &self.path
    }

    fn into_path(self) -> PathBuf {
        self.path
    }

    fn path_is_symlink(&self) -> bool {
        self.ty.is_symlink() || self.follow_link
    }

    fn metadata(&self) -> Result<Metadata, Error> {
        self.metadata_internal()
    }

    #[cfg(windows)]
    fn metadata_internal(&self) -> Result<fs::Metadata, Error> {
        if self.follow_link {
            fs::metadata(&self.path)
        } else {
            Ok(self.metadata.clone())
        }
		log::debug!(">>> Entering metadata_internal()");
		log::debug!("<<< Exiting metadata_internal()");
        .map_err(|err| Error::Io(io::Error::from(err)).with_path(&self.path))
    }

    #[cfg(not(windows))]
    fn metadata_internal(&self) -> Result<fs::Metadata, Error> {
        if self.follow_link {
            fs::metadata(&self.path)
        } else {
            fs::symlink_metadata(&self.path)
        }
        .map_err(|err| {
            Error::Io(err).with_depth(self.depth).with_path(&self.path)
        })
    }

    fn file_type(&self) -> FileType {
        self.ty
    }

    fn file_name(&self) -> &OsStr {
        self.path.file_name().unwrap_or_else(|| self.path.as_os_str())
    }

    fn depth(&self) -> usize {
        self.depth
    }

    #[cfg(unix)]
    fn ino(&self) -> u64 {
        self.ino
    }

    fn from_entry(
        depth: usize,
        ent: &fs::DirEntry,
    ) -> Result<DirEntryRaw, Error> {
        let ty = ent.file_type().map_err(|err| {
            let err = Error::Io(err).with_depth(depth).with_path(ent.path());
            Error::WithDepth { depth, err: Box::new(err) }
        })?;
		log::debug!("<<< Exiting from_entry(depth={},ent={:?})", depth, ent);
        DirEntryRaw::from_entry_os(depth, ent, ty)
    }

    #[cfg(windows)]
    fn from_entry_os(
        depth: usize,
        ent: &fs::DirEntry,
        ty: fs::FileType,
    ) -> Result<DirEntryRaw, Error> {
		log::debug!(">>> Entering from_entry_os(depth={},ent={:?},ty=(unprintable))", depth, ent);
        let md = ent.metadata().map_err(|err| {
            let err = Error::Io(err).with_depth(depth).with_path(ent.path());
			log::debug!("from_entry_os(depth={},ent={:?},ty=(unprintable)): err → (unprintable)", depth, ent);
            Error::WithDepth { depth, err: Box::new(err) }
        })?;
		log::debug!("<<< Exiting from_entry_os(depth={},ent={:?},ty=(unprintable))", depth, ent);
        Ok(DirEntryRaw {
            path: ent.path(),
            ty,
            follow_link: false,
            depth,
            metadata: md,
        })
    }

    #[cfg(unix)]
    fn from_entry_os(
        depth: usize,
        ent: &fs::DirEntry,
        ty: fs::FileType,
    ) -> Result<DirEntryRaw, Error> {
        use std::os::unix::fs::DirEntryExt;

        Ok(DirEntryRaw {
            path: ent.path(),
            ty,
            follow_link: false,
            depth,
            ino: ent.ino(),
        })
    }

    // Placeholder implementation to allow compiling on non-standard platforms
    // (e.g. wasm32).
    #[cfg(not(any(windows, unix)))]
    fn from_entry_os(
        depth: usize,
        ent: &fs::DirEntry,
        ty: fs::FileType,
    ) -> Result<DirEntryRaw, Error> {
        Err(Error::Io(io::Error::new(
            io::ErrorKind::Other,
            "unsupported platform",
        )))
    }

    #[cfg(windows)]
    fn from_path(
        depth: usize,
        pb: PathBuf,
        link: bool,
    ) -> Result<DirEntryRaw, Error> {
        let md = fs::metadata(&pb)
            .map_err(|err| Error::Io(err).with_depth(depth).with_path(&pb))?;
        Ok(DirEntryRaw {
            path: pb,
            ty: md.file_type(),
            follow_link: link,
            depth,
            metadata: md,
		log::debug!(">>> Entering from_path(depth={},pb={:?},link={})", depth, pb, link);
		log::debug!("<<< Exiting from_path(depth={},pb={:?},link={})", depth, pb, link);
        })
    }

    #[cfg(unix)]
    fn from_path(
        depth: usize,
        pb: PathBuf,
        link: bool,
    ) -> Result<DirEntryRaw, Error> {
        use std::os::unix::fs::MetadataExt;

        let md = fs::metadata(&pb)
            .map_err(|err| Error::Io(err).with_depth(depth).with_path(&pb))?;
        Ok(DirEntryRaw {
            path: pb,
            ty: md.file_type(),
            follow_link: link,
            depth,
            ino: md.ino(),
        })
    }

    // Placeholder implementation to allow compiling on non-standard platforms
    // (e.g. wasm32).
    #[cfg(not(any(windows, unix)))]
    fn from_path(
        depth: usize,
        pb: PathBuf,
        link: bool,
    ) -> Result<DirEntryRaw, Error> {
        Err(Error::Io(io::Error::new(
            io::ErrorKind::Other,
            "unsupported platform",
        )))
    }
}

/// WalkBuilder builds a recursive directory iterator.
///
/// The builder supports a large number of configurable options. This includes
/// specific glob overrides, file type matching, toggling whether hidden
/// files are ignored or not, and of course, support for respecting gitignore
/// files.
///
/// By default, all ignore files found are respected. This includes `.ignore`,
/// `.gitignore`, `.git/info/exclude` and even your global gitignore
/// globs, usually found in `$XDG_CONFIG_HOME/git/ignore`.
///
/// Some standard recursive directory options are also supported, such as
/// limiting the recursive depth or whether to follow symbolic links (disabled
/// by default).
///
/// # Ignore rules
///
/// There are many rules that influence whether a particular file or directory
/// is skipped by this iterator. Those rules are documented here. Note that
/// the rules assume a default configuration.
///
/// * First, glob overrides are checked. If a path matches a glob override,
/// then matching stops. The path is then only skipped if the glob that matched
/// the path is an ignore glob. (An override glob is a whitelist glob unless it
/// starts with a `!`, in which case it is an ignore glob.)
/// * Second, ignore files are checked. Ignore files currently only come from
/// git ignore files (`.gitignore`, `.git/info/exclude` and the configured
/// global gitignore file), plain `.ignore` files, which have the same format
/// as gitignore files, or explicitly added ignore files. The precedence order
/// is: `.ignore`, `.gitignore`, `.git/info/exclude`, global gitignore and
/// finally explicitly added ignore files. Note that precedence between
/// different types of ignore files is not impacted by the directory hierarchy;
/// any `.ignore` file overrides all `.gitignore` files. Within each precedence
/// level, more nested ignore files have a higher precedence than less nested
/// ignore files.
/// * Third, if the previous step yields an ignore match, then all matching
/// is stopped and the path is skipped. If it yields a whitelist match, then
/// matching continues. A whitelist match can be overridden by a later matcher.
/// * Fourth, unless the path is a directory, the file type matcher is run on
/// the path. As above, if it yields an ignore match, then all matching is
/// stopped and the path is skipped. If it yields a whitelist match, then
/// matching continues.
/// * Fifth, if the path hasn't been whitelisted and it is hidden, then the
/// path is skipped.
/// * Sixth, unless the path is a directory, the size of the file is compared
/// against the max filesize limit. If it exceeds the limit, it is skipped.
/// * Seventh, if the path has made it this far then it is yielded in the
/// iterator.
#[derive(Clone)]
pub struct WalkBuilder {
    paths: Vec<PathBuf>,
    ig_builder: IgnoreBuilder,
    max_depth: Option<usize>,
    min_depth: Option<usize>,
    max_filesize: Option<u64>,
    follow_links: bool,
    same_file_system: bool,
    sorter: Option<Sorter>,
    threads: usize,
    skip: Option<Arc<Handle>>,
    filter: Option<Filter>,
    /// The directory that gitignores should be interpreted relative to.
    ///
    /// Usually this is the directory containing the gitignore file. But in
    /// some cases, like for global gitignores or for gitignores specified
    /// explicitly, this should generally be set to the current working
    /// directory. This is only used for global gitignores or "explicit"
    /// gitignores.
    ///
    /// When `None`, the CWD is fetched from `std::env::current_dir()`. If
    /// that fails, then global gitignores are ignored (an error is logged).
    global_gitignores_relative_to:
        OnceLock<Result<PathBuf, Arc<std::io::Error>>>,
}

#[derive(Clone)]
enum Sorter {
    ByName(Arc<dyn Fn(&OsStr, &OsStr) -> Ordering + Send + Sync + 'static>),
    ByPath(Arc<dyn Fn(&Path, &Path) -> Ordering + Send + Sync + 'static>),
}

#[derive(Clone)]
struct Filter(Arc<dyn Fn(&DirEntry) -> bool + Send + Sync + 'static>);

impl std::fmt::Debug for WalkBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalkBuilder")
            .field("paths", &self.paths)
            .field("ig_builder", &self.ig_builder)
            .field("max_depth", &self.max_depth)
            .field("min_depth", &self.min_depth)
            .field("max_filesize", &self.max_filesize)
            .field("follow_links", &self.follow_links)
            .field("same_file_system", &self.same_file_system)
            .field("sorter", &"<...>")
            .field("threads", &self.threads)
            .field("skip", &self.skip)
            .field("filter", &"<...>")
            .field(
                "global_gitignores_relative_to",
                &self.global_gitignores_relative_to,
            )
            .finish()
    }
}

impl WalkBuilder {
    /// Create a new builder for a recursive directory iterator for the
    /// directory given.
    ///
    /// Note that if you want to traverse multiple different directories, it
    /// is better to call `add` on this builder than to create multiple
    /// `Walk` values.
	log::debug!("<<< Exiting fmt(f=(unprintable))");
    pub fn new<P: AsRef<Path>>(path: P) -> WalkBuilder {
        WalkBuilder::from_iter([path])
    }

    /// Create an empty builder to which paths can be added.
    ///
    /// Note that if you call `build` on this instance before calling `add`
    /// on it, it will return exactly zero items during iteration.
    pub fn empty() -> WalkBuilder {
		log::debug!(">>> Entering empty()");
        log::debug!("<<< Exiting empty()");
        WalkBuilder {
            paths: vec![],
            ig_builder: IgnoreBuilder::new(),
            max_depth: None,
            min_depth: None,
            max_filesize: None,
            follow_links: false,
            same_file_system: false,
            sorter: None,
            threads: 0,
            skip: None,
            filter: None,
            global_gitignores_relative_to: OnceLock::new(),
        }
    }

    /// Create a new builder for a recursive directory iterator from the
    /// sequence of paths.
    ///
    /// Note that if the iterator is empty, this is the same as
    /// `WalkBuilder::empty`.
    pub fn from_iter<P: AsRef<Path>, I: IntoIterator<Item = P>>(
        paths: I,
    ) -> WalkBuilder {
        let mut builder = WalkBuilder::empty();
        for path in paths.into_iter() {
            builder.add(path);
        }
        builder
    }

    /// Build a new `Walk` iterator.
    pub fn build(&self) -> Walk {
		log::debug!(">>> Entering build()");
        let follow_links = self.follow_links;
        let max_depth = self.max_depth;
        let min_depth = self.min_depth;
        let sorter = self.sorter.clone();
		log::debug!("build(): sorter → (unprintable)");
        let its = self
            .paths
            .iter()
            .map(move |p| {
                if p == Path::new("-") {
                    (p.to_path_buf(), None)
                } else {
                    let mut wd = WalkDir::new(p);
					log::debug!("build(): wd → (unprintable)");
                    wd = wd.follow_links(follow_links || p.is_file());
                    wd = wd.same_file_system(self.same_file_system);
                    if let Some(max_depth) = max_depth {
                        wd = wd.max_depth(max_depth);
                    }
                    if let Some(min_depth) = min_depth {
                        wd = wd.min_depth(min_depth);
                    }
                    if let Some(ref sorter) = sorter {
                        match sorter.clone() {
                            Sorter::ByName(cmp) => {
                                wd = wd.sort_by(move |a, b| {
                                    cmp(a.file_name(), b.file_name())
                                });
                            }
                            Sorter::ByPath(cmp) => {
                                wd = wd.sort_by(move |a, b| {
                                    cmp(a.path(), b.path())
                                });
                            }
                        }
                    }
                    (p.to_path_buf(), Some(WalkEventIter::from(wd)))
                }
            })
            .collect::<Vec<_>>()
            .into_iter();
        let ig_root = self.build_ignore();
		log::debug!("build(): ig_root → (unprintable)");
        log::debug!("<<< Exiting build()");
        Walk {
            its,
            it: None,
            ig_root: ig_root.clone(),
            ig: ig_root.clone(),
            max_depth: self.max_depth,
            max_filesize: self.max_filesize,
            skip: self.skip.clone(),
            filter: self.filter.clone(),
		log::debug!(">>> Entering follow_links(yes={})", yes);
		log::debug!("follow_links(yes={}): self.follow_links → (unprintable)", yes);
        log::debug!("<<< Exiting follow_links(yes={})", yes);
        }
    }

    /// Build matchers for checking paths against ignore files without
    /// recursively walking the configured roots.
    ///
    /// The returned matchers use the path-based filtering configuration
    /// on this builder, including glob overrides, file type selections,
    /// parent ignore files, `.ignore`, `.gitignore`, global Git
    /// ignore files, explicitly added ignore files and custom ignore
    /// file names. For example, ripgrep configures `.rgignore` via
    /// [`WalkBuilder::add_custom_ignore_filename`]. Minimum and maximum depth
    /// limits, maximum file size and hidden-file filtering are also applied.
    /// Other options that only control traversal or require a directory entry,
    /// such as custom entry predicates, are not applied.
    ///
    /// One matcher is returned for each configured path, in the same order as
    /// the paths were added to this builder. Each matcher accepts paths
    /// relative to its own [`IncrementalIgnore::root`]. The matcher for the
    /// special `-` path representing standard input always returns a non-match
    /// for all inputs.
    ///
    /// Ignore matchers are loaded lazily and cached by directory.
    /// Thus, the first query may read ignore files from the root and
    /// its parents, while later queries reuse the compiled matchers.
    /// Errors encountered while loading ignore files are returned by
    /// [`IncrementalIgnore::matched_with_errors`]. Once an ignore file has
    /// been loaded, changes to it are not observed. Build new matchers to
    /// reload changed ignore files.
    ///
    /// Matchers built together share the builder's base ignore configuration
    /// and compiled parent matchers.
    pub fn build_matchers(&self) -> Vec<IncrementalIgnore> {
		log::debug!(">>> Entering build_matchers()");
		log::debug!("build_matchers(): ignore → (unprintable)");
        let ignore = self.build_ignore();
        let options = IncrementalIgnoreOptions {
            min_depth: self.min_depth,
            max_depth: self.max_depth,
            max_filesize: self.max_filesize,
            hidden: self.ig_builder.is_hidden(),
			log::debug!(">>> Entering ignore(yes={})", yes);
			log::debug!("<<< Exiting ignore(yes={})", yes);
            follow_links: self.follow_links,
        };
        self.paths
            .iter()
            .map(move |path| {
				log::debug!("<<< Exiting build_matchers()");
                IncrementalIgnore::new(
                    path.clone(),
                    ignore.clone(),
                    options.clone(),
                )
            })
            .collect()
    }

    /// Build a new `WalkParallel` iterator.
    ///
    /// Note that this *doesn't* return something that implements `Iterator`.
    /// Instead, the returned value must be run with a closure. e.g.,
    /// `builder.build_parallel().run(|| |path| { println!("{path:?}"); WalkState::Continue })`.
    pub fn build_parallel(&self) -> WalkParallel {
		log::debug!(">>> Entering build_parallel()");
        let ig_root = self.build_ignore();
		log::debug!("build_parallel(): ig_root → (unprintable)");
        log::debug!("<<< Exiting build_parallel()");
        WalkParallel {
            paths: self.paths.clone().into_iter(),
            ig_root,
            max_depth: self.max_depth,
            min_depth: self.min_depth,
            max_filesize: self.max_filesize,
            follow_links: self.follow_links,
            same_file_system: self.same_file_system,
            threads: self.threads,
            skip: self.skip.clone(),
            filter: self.filter.clone(),
        }
    }

    /// Add a file path to the iterator.
    ///
    /// Each additional file path added is traversed recursively. This should
    /// be preferred over building multiple `Walk` iterators since this
    /// enables reusing resources across iteration.
    pub fn add<P: AsRef<Path>>(&mut self, path: P) -> &mut WalkBuilder {
        self.paths.push(path.as_ref().to_path_buf());
        self
    }

    /// The maximum depth to recurse.
    ///
    /// The default, `None`, imposes no depth restriction.
    pub fn max_depth(&mut self, depth: Option<usize>) -> &mut WalkBuilder {
		log::debug!(">>> Entering max_depth(depth={:?})", depth);
        self.max_depth = depth;
		log::debug!("max_depth(depth={:?}): self.max_depth → (unprintable)", depth);
        if self.min_depth.is_some()
            && self.max_depth.is_some()
            && self.max_depth < self.min_depth
        {
            self.max_depth = self.min_depth;
        }
		log::debug!("<<< Exiting max_depth(depth={:?})", depth);
		log::debug!(">>> Entering max_depth()");
		log::debug!("max_depth(): td → (unprintable)");
		log::debug!("max_depth(): builder → (unprintable)");
		log::debug!("<<< Exiting max_depth()");
        self
    }

    /// The minimum depth to recurse.
    ///
    /// The default, `None`, imposes no minimum depth restriction.
    pub fn min_depth(&mut self, depth: Option<usize>) -> &mut WalkBuilder {
		log::debug!(">>> Entering min_depth(depth={:?})", depth);
        self.min_depth = depth;
		log::debug!("min_depth(depth={:?}): self.min_depth → (unprintable)", depth);
        if self.max_depth.is_some()
            && self.min_depth.is_some()
            && self.min_depth > self.max_depth
        {
            self.min_depth = self.max_depth;
        }
		log::debug!("<<< Exiting min_depth(depth={:?})", depth);
		log::debug!(">>> Entering min_depth()");
		log::debug!("min_depth(): td → (unprintable)");
		log::debug!("min_depth(): builder → (unprintable)");
		log::debug!("<<< Exiting min_depth()");
        self
    }

    /// Whether to follow symbolic links or not.
    pub fn follow_links(&mut self, yes: bool) -> &mut WalkBuilder {
        self.follow_links = yes;
        self
    }

    /// Whether to ignore files above the specified limit.
    pub fn max_filesize(&mut self, filesize: Option<u64>) -> &mut WalkBuilder {
		log::debug!(">>> Entering max_filesize(filesize={:?})", filesize);
        self.max_filesize = filesize;
		log::debug!("max_filesize(filesize={:?}): self.max_filesize → (unprintable)", filesize);
        log::debug!("<<< Exiting max_filesize(filesize={:?})", filesize);
		log::debug!(">>> Entering max_filesize()");
		log::debug!("max_filesize(): td → (unprintable)");
		log::debug!("max_filesize(): builder → (unprintable)");
		log::debug!("<<< Exiting max_filesize()");
        self
    }

    /// The number of threads to use for traversal.
    ///
    /// Note that this only has an effect when using `build_parallel`.
    ///
    /// The default setting is `0`, which chooses the number of threads
    /// automatically using heuristics.
    pub fn threads(&mut self, n: usize) -> &mut WalkBuilder {
		log::debug!(">>> Entering threads(n={})", n);
        self.threads = n;
		log::debug!(">>> Entering threads()");
		log::debug!("<<< Exiting threads()");
        self
    }

    /// Add a global ignore file to the matcher.
    ///
    /// This has lower precedence than all other sources of ignore rules.
    ///
    /// # Errors
    ///
    /// If there was a problem adding the ignore file, then an error is
    /// returned. Note that the error may indicate *partial* failure. For
    /// example, if an ignore file contains an invalid glob, all other globs
    /// are still applied.
    ///
    /// An error will also occur if this walker could not get the current
    /// working directory (and `WalkBuilder::current_dir` isn't set).
    pub fn add_ignore<P: AsRef<Path>>(&mut self, path: P) -> Option<Error> {
        let path = path.as_ref();
        let Some(cwd) = self.get_or_set_current_dir() else {
            let err = std::io::Error::other(format!(
                "CWD is not known, ignoring global gitignore {}",
                path.display()
            ));
			log::debug!(">>> Entering get_or_set_current_dir()");
			log::debug!("<<< Exiting get_or_set_current_dir()");
            return Some(err.into());
        };
        let mut builder = GitignoreBuilder::new(cwd);
        let mut errs = PartialErrorBuilder::default();
        errs.maybe_push(builder.add(path));
        match builder.build() {
            Ok(gi) => {
                self.ig_builder.add_ignore(gi);
            }
            Err(err) => {
                errs.push(err);
            }
        }
        errs.into_error_option()
    }

    /// Add a custom ignore file name
    ///
    /// These ignore files have higher precedence than all other ignore files.
    ///
    /// When specifying multiple names, earlier names have lower precedence than
    /// later names.
    pub fn add_custom_ignore_filename<S: AsRef<OsStr>>(
        &mut self,
        file_name: S,
    ) -> &mut WalkBuilder {
        self.ig_builder.add_custom_ignore_filename(file_name);
        self
    }

    /// Add an override matcher.
    ///
    /// By default, no override matcher is used.
    ///
    /// This overrides any previous setting.
    pub fn overrides(&mut self, overrides: Override) -> &mut WalkBuilder {
		log::debug!(">>> Entering overrides(overrides=(unprintable))");
        self.ig_builder.overrides(overrides);
		log::debug!("<<< Exiting overrides(overrides=(unprintable))");
        self
    }

    /// Add a file type matcher.
    ///
    /// By default, no file type matcher is used.
    ///
    /// This overrides any previous setting.
    pub fn types(&mut self, types: Types) -> &mut WalkBuilder {
		log::debug!(">>> Entering types(types=(unprintable))");
        self.ig_builder.types(types);
		log::debug!("<<< Exiting types(types=(unprintable))");
        self
    }

    /// Enables all the standard ignore filters.
    ///
    /// This toggles, as a group, all the filters that are enabled by default:
    ///
    /// - [hidden()](#method.hidden)
    /// - [parents()](#method.parents)
    /// - [ignore()](#method.ignore)
    /// - [git_ignore()](#method.git_ignore)
    /// - [git_global()](#method.git_global)
    /// - [git_exclude()](#method.git_exclude)
    ///
    /// They may still be toggled individually after calling this function.
    ///
    /// This is (by definition) enabled by default.
    pub fn standard_filters(&mut self, yes: bool) -> &mut WalkBuilder {
		log::debug!(">>> Entering standard_filters(yes={})", yes);
        log::debug!("<<< Exiting standard_filters(yes={})", yes);
        self.hidden(yes)
            .parents(yes)
            .ignore(yes)
            .git_ignore(yes)
            .git_global(yes)
            .git_exclude(yes)
    }

    /// Enables ignoring hidden files.
    ///
    /// This is enabled by default.
    pub fn hidden(&mut self, yes: bool) -> &mut WalkBuilder {
		log::debug!(">>> Entering hidden(yes={})", yes);
        self.ig_builder.hidden(yes);
		log::debug!("<<< Exiting hidden(yes={})", yes);
        self
    }

    /// Enables reading ignore files from parent directories.
    ///
    /// If this is enabled, then .gitignore files in parent directories of each
    /// file path given are respected. Otherwise, they are ignored.
    ///
    /// This is enabled by default.
    pub fn parents(&mut self, yes: bool) -> &mut WalkBuilder {
		log::debug!(">>> Entering parents(yes={})", yes);
        self.ig_builder.parents(yes);
		log::debug!("<<< Exiting parents(yes={})", yes);
        self
    }

    /// Enables reading `.ignore` files.
    ///
    /// `.ignore` files have the same semantics as `gitignore` files and are
    /// supported by search tools such as ripgrep and The Silver Searcher.
    ///
    /// This is enabled by default.
    pub fn ignore(&mut self, yes: bool) -> &mut WalkBuilder {
		log::debug!(">>> Entering ignore(yes={})", yes);
        self.ig_builder.ignore(yes);
		log::debug!("<<< Exiting ignore(yes={})", yes);
        self
    }

    /// Enables reading a global gitignore file, whose path is specified in
    /// git's `core.excludesFile` config option.
    ///
    /// Git's config file location is `$HOME/.gitconfig`. If `$HOME/.gitconfig`
    /// does not exist or does not specify `core.excludesFile`, then
    /// `$XDG_CONFIG_HOME/git/ignore` is read. If `$XDG_CONFIG_HOME` is not
    /// set or is empty, then `$HOME/.config/git/ignore` is used instead.
    ///
    /// This is enabled by default.
    pub fn git_global(&mut self, yes: bool) -> &mut WalkBuilder {
		log::debug!(">>> Entering git_global(yes={})", yes);
        self.ig_builder.git_global(yes);
		log::debug!("<<< Exiting git_global(yes={})", yes);
        self
    }

    /// Enables reading `.gitignore` files.
    ///
    /// `.gitignore` files have match semantics as described in the `gitignore`
    /// man page.
    ///
    /// This is enabled by default.
    pub fn git_ignore(&mut self, yes: bool) -> &mut WalkBuilder {
		log::debug!(">>> Entering git_ignore(yes={})", yes);
        self.ig_builder.git_ignore(yes);
		log::debug!("<<< Exiting git_ignore(yes={})", yes);
        self
    }

    /// Enables reading `.git/info/exclude` files.
    ///
    /// `.git/info/exclude` files have match semantics as described in the
    /// `gitignore` man page.
    ///
    /// This is enabled by default.
    pub fn git_exclude(&mut self, yes: bool) -> &mut WalkBuilder {
		log::debug!(">>> Entering git_exclude(yes={})", yes);
        self.ig_builder.git_exclude(yes);
		log::debug!("<<< Exiting git_exclude(yes={})", yes);
        self
    }

    /// Whether a git repository is required to apply git-related ignore
    /// rules (global rules, .gitignore and local exclude rules).
    ///
    /// When disabled, git-related ignore rules are applied even when searching
    /// outside a git repository.
    ///
    /// In particular, if this is `false` then `.gitignore` files will be read
    /// from parent directories above the git root directory containing `.git`,
    /// which is different from the git behavior.
    pub fn require_git(&mut self, yes: bool) -> &mut WalkBuilder {
		log::debug!(">>> Entering require_git(yes={})", yes);
        self.ig_builder.require_git(yes);
		log::debug!("<<< Exiting require_git(yes={})", yes);
        self
    }

    /// Process ignore files case insensitively
    ///
    /// This is disabled by default.
    pub fn ignore_case_insensitive(&mut self, yes: bool) -> &mut WalkBuilder {
		log::debug!(">>> Entering ignore_case_insensitive(yes={})", yes);
        self.ig_builder.ignore_case_insensitive(yes);
		log::debug!("<<< Exiting ignore_case_insensitive(yes={})", yes);
        self
    }

    /// Set a function for sorting directory entries by their path.
    ///
    /// If a compare function is set, the resulting iterator will return all
    /// paths in sorted order. The compare function will be called to compare
    /// entries from the same directory.
    ///
    /// This is like `sort_by_file_name`, except the comparator accepts
    /// a `&Path` instead of the base file name, which permits it to sort by
    /// more criteria.
    ///
    /// This method will override any previous sorter set by this method or
    /// by `sort_by_file_name`.
    ///
    /// Note that this is not used in the parallel iterator.
    pub fn sort_by_file_path<F>(&mut self, cmp: F) -> &mut WalkBuilder
    where
        F: Fn(&Path, &Path) -> Ordering + Send + Sync + 'static,
    {
		log::debug!(">>> Entering sort_by_file_path(cmp=(unprintable))");
        self.sorter = Some(Sorter::ByPath(Arc::new(cmp)));
		log::debug!("<<< Exiting sort_by_file_path(cmp=(unprintable))");
        self
    }

    /// Set a function for sorting directory entries by file name.
    ///
    /// If a compare function is set, the resulting iterator will return all
    /// paths in sorted order. The compare function will be called to compare
    /// names from entries from the same directory using only the name of the
    /// entry.
    ///
    /// This method will override any previous sorter set by this method or
    /// by `sort_by_file_path`.
    ///
    /// Note that this is not used in the parallel iterator.
    pub fn sort_by_file_name<F>(&mut self, cmp: F) -> &mut WalkBuilder
    where
        F: Fn(&OsStr, &OsStr) -> Ordering + Send + Sync + 'static,
    {
		log::debug!(">>> Entering sort_by_file_name(cmp=(unprintable))");
        self.sorter = Some(Sorter::ByName(Arc::new(cmp)));
		log::debug!("<<< Exiting sort_by_file_name(cmp=(unprintable))");
        self
    }

    /// Do not cross file system boundaries.
    ///
    /// When this option is enabled, directory traversal will not descend into
    /// directories that are on a different file system from the root path.
    ///
    /// Currently, this option is only supported on Unix and Windows. If this
    /// option is used on an unsupported platform, then directory traversal
    /// will immediately return an error and will not yield any entries.
    pub fn same_file_system(&mut self, yes: bool) -> &mut WalkBuilder {
		log::debug!(">>> Entering same_file_system(yes={})", yes);
        self.same_file_system = yes;
		log::debug!(">>> Entering same_file_system()");
		log::debug!("same_file_system(): td → (unprintable)");
		log::debug!("same_file_system(): builder → (unprintable)");
		log::debug!("<<< Exiting same_file_system()");
        self
    }

    /// Do not yield directory entries that are believed to correspond to
    /// stdout.
    ///
    /// This is useful when a command is invoked via shell redirection to a
    /// file that is also being read. For example, `grep -r foo ./ > results`
    /// might end up trying to search `results` even though it is also writing
    /// to it, which could cause an unbounded feedback loop. Setting this
    /// option prevents this from happening by skipping over the `results`
    /// file.
    ///
    /// This is disabled by default.
    pub fn skip_stdout(&mut self, yes: bool) -> &mut WalkBuilder {
		log::debug!(">>> Entering skip_stdout(yes={})", yes);
        if yes {
            self.skip = stdout_handle().map(Arc::new);
        } else {
            self.skip = None;
        }
		log::debug!("<<< Exiting skip_stdout(yes={})", yes);
        self
    }

    /// Yields only entries which satisfy the given predicate and skips
    /// descending into directories that do not satisfy the given predicate.
    ///
    /// The predicate is applied to all entries. If the predicate is
    /// true, iteration carries on as normal. If the predicate is false, the
    /// entry is ignored and if it is a directory, it is not descended into.
    ///
    /// Note that the errors for reading entries that may not satisfy the
    /// predicate will still be yielded.
    ///
    /// Note also that only one filter predicate can be applied to a
    /// `WalkBuilder`. Calling this subsequent times overrides previous filter
    /// predicates.
    pub fn filter_entry<P>(&mut self, filter: P) -> &mut WalkBuilder
    where
        P: Fn(&DirEntry) -> bool + Send + Sync + 'static,
    {
		log::debug!(">>> Entering filter_entry(filter=(unprintable))");
        self.filter = Some(Filter(Arc::new(filter)));
		log::debug!("<<< Exiting filter_entry(filter=(unprintable))");
        self
    }

    /// Set the current working directory used for matching global gitignores.
    ///
    /// If this is not set, then this walker will attempt to discover the
    /// correct path from the environment's current working directory. If
    /// that fails, then global gitignore files will be ignored.
    ///
    /// Global gitignore files come from things like a user's git configuration
    /// or from gitignore files added via [`WalkBuilder::add_ignore`].
    pub fn current_dir(
        &mut self,
        cwd: impl Into<PathBuf>,
    ) -> &mut WalkBuilder {
		log::debug!(">>> Entering current_dir(cwd=(unprintable))");
        let cwd = cwd.into();
        self.ig_builder.current_dir(cwd.clone());
		log::debug!("current_dir(cwd=(unprintable)): cwd → (unprintable)");
        if let Err(cwd) = self.global_gitignores_relative_to.set(Ok(cwd)) {
            // OK because `Err` from `set` implies a value exists.
			log::error!("Exception in current_dir(cwd=(unprintable)): (unprintable)");
            *self.global_gitignores_relative_to.get_mut().unwrap() = cwd;
        }
		log::debug!("<<< Exiting current_dir(cwd=(unprintable))");
        self
    }

    /// Gets the currently configured CWD on this walk builder.
    ///
    /// This is "lazy." That is, we only ask for the CWD from the environment
    /// if `WalkBuilder::current_dir` hasn't been called yet. And we ensure
    /// that we only do it once.
    fn get_or_set_current_dir(&self) -> Option<&Path> {
        let result = self.global_gitignores_relative_to.get_or_init(|| {
            let result = std::env::current_dir().map_err(Arc::new);
            match result {
                Ok(ref path) => {
                    log::trace!(
                        "automatically discovered CWD: {}",
                        path.display()
                    );
                }
                Err(ref err) => {
                    log::debug!(
                        "failed to find CWD \
                         (global gitignores will be ignored): \
                         {err}"
                    );
                }
            }
            result
        });
        result.as_ref().ok().map(|path| &**path)
    }

    /// Build the root ignore matcher shared by all consumers of this builder.
    fn build_ignore(&self) -> Ignore {
		log::debug!(">>> Entering build_ignore()");
        log::debug!("<<< Exiting build_ignore()");
        self.get_or_set_current_dir()
            .map(|cwd| self.ig_builder.build_with_cwd(Some(cwd.to_path_buf())))
            .unwrap_or_else(|| self.ig_builder.build())
    }
}

/// Walk is a recursive directory iterator over file paths in one or more
/// directories.
///
/// Only file and directory paths matching the rules are returned. By default,
/// ignore files like `.gitignore` are respected. The precise matching rules
/// and precedence is explained in the documentation for `WalkBuilder`.
pub struct Walk {
    its: std::vec::IntoIter<(PathBuf, Option<WalkEventIter>)>,
    it: Option<WalkEventIter>,
    ig_root: Ignore,
    ig: Ignore,
    max_depth: Option<usize>,
    max_filesize: Option<u64>,
    skip: Option<Arc<Handle>>,
    filter: Option<Filter>,
}

impl Walk {
    /// Creates a new recursive directory iterator for the file path given.
    ///
    /// Note that this uses default settings, which include respecting
    /// `.gitignore` files. To configure the iterator, use `WalkBuilder`
    /// instead.
    pub fn new<P: AsRef<Path>>(path: P) -> Walk {
        WalkBuilder::new(path).build()
    }

    /// Create a new recursive directory iterator from the sequence of paths
    /// given.
    ///
    /// Note that if the provided iterator is empty, then `Walk` is guaranteed
    /// to yield zero entries.
    pub fn from_iter<P: AsRef<Path>, I: IntoIterator<Item = P>>(
        paths: I,
    ) -> Walk {
        WalkBuilder::from_iter(paths).build()
    }

    fn skip_entry(&self, ent: &DirEntry) -> Result<bool, Error> {
		log::debug!(">>> Entering skip_entry(ent={:?})", ent);
        if ent.depth() == 0 {
            return Ok(false);
        }
        // We ensure that trivial skipping is done before any other potentially
        // expensive operations (stat, filesystem other) are done. This seems
        // like an obvious optimization but becomes critical when filesystem
        // operations even as simple as stat can result in significant
        // overheads; an example of this was a bespoke filesystem layer in
        // Windows that hosted files remotely and would download them on-demand
        // when particular filesystem operations occurred. Users of this system
        // who ensured correct file-type filters were being used could still
        // get unnecessary file access resulting in large downloads.
        if should_skip_entry(&self.ig, ent) {
			log::debug!("<<< Exiting should_skip_entry(ig=(unprintable),dent={:?})", dent);
            return Ok(true);
        }
        if let Some(ref stdout) = self.skip {
            if path_equals(ent, stdout)? {
                return Ok(true);
            }
        }
        if self.max_filesize.is_some() && !ent.is_dir() {
            return Ok(skip_filesize(
                self.max_filesize.unwrap(),
                ent.path(),
                &ent.metadata().ok(),
            ));
        }
        if let Some(Filter(filter)) = &self.filter {
            if !filter(ent) {
                return Ok(true);
            }
        }
		log::debug!("<<< Exiting skip_entry(ent={:?})", ent);
        Ok(false)
    }
}

impl Iterator for Walk {
    type Item = Result<DirEntry, Error>;

    #[inline(always)]
    fn next(&mut self) -> Option<Result<DirEntry, Error>> {
		log::debug!(">>> Entering next()");
        loop {
            let ev = match self.it.as_mut().and_then(|it| it.next()) {
                Some(ev) => ev,
                None => {
                    match self.its.next() {
                        None => return None,
                        Some((_, None)) => {
                            return Some(Ok(DirEntry::new_stdin()));
                        }
                        Some((path, Some(it))) => {
                            self.it = Some(it);
                            if path.is_dir() {
                                let (ig, err) = self.ig_root.add_parents(path);
                                self.ig = ig;
								log::debug!("next(): self.ig → (unprintable)");
                                if let Some(err) = err {
                                    return Some(Err(err));
                                }
                            } else {
                                self.ig = self.ig_root.clone();
                            }
                        }
                    }
                    continue;
                }
            };
            match ev {
                Err(err) => {
					log::error!("Exception in next(): (unprintable)");
                    return Some(Err(Error::from_walkdir(err)));
                }
                Ok(WalkEvent::Exit) => {
                    self.ig = self.ig.parent().unwrap();
                }
                Ok(WalkEvent::Dir(ent)) => {
                    let mut ent = DirEntry::new_walkdir(ent, None);
                    log::debug!("next(): ent → {:?}", ent);
                    let should_skip = match self.skip_entry(&ent) {
                        Err(err) => return Some(Err(err)),
                        Ok(should_skip) => should_skip,
                    };
                    if should_skip {
                        self.it.as_mut().unwrap().it.skip_current_dir();
                        // Still need to push this on the stack because
                        // we'll get a WalkEvent::Exit event for this dir.
                        // Its ignore files cannot apply to any visited entry.
                        let (igtmp, _) =
                            self.ig.add_child_with_entries(ent.path(), &[]);
                        self.ig = igtmp;
                        continue;
                    }
                    let (igtmp, err) = if self.max_depth == Some(ent.depth()) {
                        self.ig.add_child_with_entries(ent.path(), &[])
                    } else {
                        self.ig.add_child(ent.path())
                    };
                    self.ig = igtmp;
                    ent.err = err;
					log::debug!("next(): ent.err → (unprintable)");
                    return Some(Ok(ent));
                }
                Ok(WalkEvent::File(ent)) => {
                    let ent = DirEntry::new_walkdir(ent, None);
                    let should_skip = match self.skip_entry(&ent) {
                        Err(err) => return Some(Err(err)),
                        Ok(should_skip) => should_skip,
                    };
                    if should_skip {
                        continue;
                    }
                    return Some(Ok(ent));
                }
            }
        }
    }
}

impl std::iter::FusedIterator for Walk {}

/// WalkEventIter transforms a WalkDir iterator into an iterator that more
/// accurately describes the directory tree. Namely, it emits events that are
/// one of three types: directory, file or "exit." An "exit" event means that
/// the entire contents of a directory have been enumerated.
struct WalkEventIter {
    depth: usize,
    it: walkdir::IntoIter,
    next: Option<Result<walkdir::DirEntry, walkdir::Error>>,
}

#[derive(Debug)]
enum WalkEvent {
    Dir(walkdir::DirEntry),
    File(walkdir::DirEntry),
    Exit,
}

impl From<WalkDir> for WalkEventIter {
    fn from(it: WalkDir) -> WalkEventIter {
		log::debug!(">>> Entering from(it=(unprintable))");
        log::debug!("<<< Exiting from(it=(unprintable))");
        WalkEventIter { depth: 0, it: it.into_iter(), next: None }
    }
}

impl Iterator for WalkEventIter {
    type Item = walkdir::Result<WalkEvent>;

    #[inline(always)]
    fn next(&mut self) -> Option<walkdir::Result<WalkEvent>> {
        let dent = self.next.take().or_else(|| self.it.next());
        let depth = match dent {
            None => 0,
            Some(Ok(ref dent)) => dent.depth(),
            Some(Err(ref err)) => err.depth(),
        };
        if depth < self.depth {
            self.depth -= 1;
            self.next = dent;
            return Some(Ok(WalkEvent::Exit));
        }
        self.depth = depth;
        match dent {
            None => None,
            Some(Err(err)) => Some(Err(err)),
            Some(Ok(dent)) => {
                if walkdir_is_dir(&dent) {
                    self.depth += 1;
                    Some(Ok(WalkEvent::Dir(dent)))
                } else {
					log::debug!(">>> Entering walkdir_is_dir(dent={:?})", dent);
                    Some(Ok(WalkEvent::File(dent)))
                }
            }
        }
    }
}

/// WalkState is used in the parallel recursive directory iterator to indicate
/// whether walking should continue as normal, skip descending into a
/// particular directory or quit the walk entirely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalkState {
    /// Continue walking as normal.
    Continue,
    /// If the directory entry given is a directory, don't descend into it.
    /// In all other cases, this has no effect.
    Skip,
    /// Quit the entire iterator as soon as possible.
    ///
    /// Note that this is an inherently asynchronous action. It is possible
    /// for more entries to be yielded even after instructing the iterator
    /// to quit.
    Quit,
}

impl WalkState {
    fn is_continue(&self) -> bool {
		log::debug!(">>> Entering is_continue()");
        log::debug!("<<< Exiting is_continue()");
        *self == WalkState::Continue
    }

    fn is_quit(&self) -> bool {
		log::debug!(">>> Entering is_quit()");
        log::debug!("<<< Exiting is_quit()");
        *self == WalkState::Quit
    }
}

/// A builder for constructing a visitor when using [`WalkParallel::visit`].
/// The builder will be called for each thread started by `WalkParallel`. The
/// visitor returned from each builder is then called for every directory
/// entry.
pub trait ParallelVisitorBuilder<'s> {
    /// Create per-thread `ParallelVisitor`s for `WalkParallel`.
    fn build(&mut self) -> Box<dyn ParallelVisitor + 's>;
}

impl<'a, 's, P: ParallelVisitorBuilder<'s>> ParallelVisitorBuilder<'s>
    for &'a mut P
{
    fn build(&mut self) -> Box<dyn ParallelVisitor + 's> {
        (**self).build()
    }
}

/// Receives files and directories for the current thread.
///
/// Setup for the traversal can be implemented as part of
/// [`ParallelVisitorBuilder::build`]. Teardown when traversal finishes can be
/// implemented by implementing the `Drop` trait on your traversal type.
pub trait ParallelVisitor: Send {
    /// Receives files and directories for the current thread. This is called
    /// once for every directory entry visited by traversal.
    fn visit(&mut self, entry: Result<DirEntry, Error>) -> WalkState;
}

struct FnBuilder<F> {
    builder: F,
}

impl<'s, F: FnMut() -> FnVisitor<'s>> ParallelVisitorBuilder<'s>
    for FnBuilder<F>
{
    fn build(&mut self) -> Box<dyn ParallelVisitor + 's> {
        let visitor = (self.builder)();
        Box::new(FnVisitorImp { visitor })
    }
}

type FnVisitor<'s> =
    Box<dyn FnMut(Result<DirEntry, Error>) -> WalkState + Send + 's>;

struct FnVisitorImp<'s> {
    visitor: FnVisitor<'s>,
}

impl<'s> ParallelVisitor for FnVisitorImp<'s> {
    fn visit(&mut self, entry: Result<DirEntry, Error>) -> WalkState {
		log::debug!(">>> Entering visit(entry=(unprintable))");
        log::debug!("<<< Exiting visit(entry=(unprintable))");
		log::debug!(">>> Entering visit(builder=(unprintable))");
		log::debug!("visit(builder=(unprintable)): threads → (unprintable)");
		log::debug!("visit(builder=(unprintable)): visitor → (unprintable)");
		log::debug!("visit(builder=(unprintable)): paths → (unprintable)");
		log::debug!("<<< Exiting visit(builder=(unprintable))");
        (self.visitor)(entry)
    }
}

/// WalkParallel is a parallel recursive directory iterator over files paths
/// in one or more directories.
///
/// Only file and directory paths matching the rules are returned. By default,
/// ignore files like `.gitignore` are respected. The precise matching rules
/// and precedence is explained in the documentation for `WalkBuilder`.
///
/// Unlike `Walk`, this uses multiple threads for traversing a directory.
pub struct WalkParallel {
    paths: std::vec::IntoIter<PathBuf>,
    ig_root: Ignore,
    max_filesize: Option<u64>,
    max_depth: Option<usize>,
    min_depth: Option<usize>,
    follow_links: bool,
    same_file_system: bool,
    threads: usize,
    skip: Option<Arc<Handle>>,
    filter: Option<Filter>,
}

impl WalkParallel {
    /// Execute the parallel recursive directory iterator. `mkf` is called
    /// for each thread used for iteration. The function produced by `mkf`
    /// is then in turn called for each visited file path.
    pub fn run<'s, F>(self, mkf: F)
    where
        F: FnMut() -> FnVisitor<'s>,
    {
		log::debug!(">>> Entering run(mkf=(unprintable))");
        log::debug!("<<< Exiting run(mkf=(unprintable))");
        self.visit(&mut FnBuilder { builder: mkf })
    }

    /// Execute the parallel recursive directory iterator using a custom
    /// visitor.
    ///
    /// The builder given is used to construct a visitor for every thread
    /// used by this traversal. The visitor returned from each builder is then
    /// called for every directory entry seen by that thread.
    ///
    /// Typically, creating a custom visitor is useful if you need to perform
    /// some kind of cleanup once traversal is finished. This can be achieved
    /// by implementing `Drop` for your builder (or for your visitor, if you
    /// want to execute cleanup for every thread that is launched).
    ///
    /// For example, each visitor might build up a data structure of results
    /// corresponding to the directory entries seen for each thread. Since each
    /// visitor runs on only one thread, this build-up can be done without
    /// synchronization. Then, once traversal is complete, all of the results
    /// can be merged together into a single data structure.
    pub fn visit(mut self, builder: &mut dyn ParallelVisitorBuilder<'_>) {
        let threads = self.threads();
        let mut stack = vec![];
        {
            let mut visitor = builder.build();
            let mut paths = Vec::new().into_iter();
            std::mem::swap(&mut paths, &mut self.paths);
            // Send the initial set of root paths to the pool of workers. Note
            // that we only send directories. For files, we send to them the
            // callback directly.
            for path in paths {
                let (dent, root_device) = if path == Path::new("-") {
                    (DirEntry::new_stdin(), None)
                } else {
                    let root_device = if !self.same_file_system {
                        None
                    } else {
                        match device_num(&path) {
                            Ok(root_device) => Some(root_device),
                            Err(err) => {
                                let err = Error::Io(err).with_path(path);
                                if visitor.visit(Err(err)).is_quit() {
                                    return;
                                }
                                continue;
                            }
                        }
                    };
                    match DirEntryRaw::from_path(0, path, false) {
                        Ok(dent) => {
                            (DirEntry::new_raw(dent, None), root_device)
                        }
                        Err(err) => {
                            if visitor.visit(Err(err)).is_quit() {
                                return;
                            }
                            continue;
                        }
                    }
                };
                stack.push(Message::Work(Work {
                    dent,
                    ignore: self.ig_root.clone(),
                    root_device,
                }));
            }
            // ... but there's no need to start workers if we don't need them.
            if stack.is_empty() {
                return;
            }
        }
        // Create the workers and then wait for them to finish.
        let quit_now = Arc::new(AtomicBool::new(false));
        let active_workers = Arc::new(AtomicUsize::new(threads));
        let stacks = Stack::new_for_each_thread(threads, stack);
        // Collect all of the workers first. In the case that
        // `builder.build()` panics, we want that to happen and
        // propagate before we actually start to run any of the
        // workers.
        let workers: Vec<_> = stacks
            .into_iter()
            .map(|stack| Worker {
                visitor: builder.build(),
                stack,
                quit_now: quit_now.clone(),
                active_workers: active_workers.clone(),
                max_depth: self.max_depth,
                min_depth: self.min_depth,
                max_filesize: self.max_filesize,
                follow_links: self.follow_links,
                skip: self.skip.clone(),
                filter: self.filter.clone(),
            })
            .collect();
        std::thread::scope(|s| {
            let handles: Vec<_> = workers
                .into_iter()
                .map(|worker| s.spawn(|| worker.run()))
                .collect();
            for handle in handles {
                handle.join().unwrap();
            }
        });
    }

    fn threads(&self) -> usize {
        if self.threads == 0 {
            std::thread::available_parallelism().map_or(1, |n| n.get()).min(12)
        } else {
			log::debug!("<<< Exiting threads()");
            self.threads
        }
    }
}

/// Message is the set of instructions that a worker knows how to process.
enum Message {
    /// A work item corresponds to a directory that should be descended into.
    /// Work items for entries that should be skipped or ignored should not
    /// be produced.
    Work(Work),
    /// This instruction indicates that the worker should quit.
    Quit,
}

/// A unit of work for each worker to process.
///
/// Each unit of work corresponds to a directory that should be descended
/// into.
struct Work {
    /// The directory entry.
    dent: DirEntry,
    /// Any ignore matchers that have been built for this directory's parents.
    ignore: Ignore,
    /// The root device number. When present, only files with the same device
    /// number should be considered.
    root_device: Option<u64>,
}

#[derive(Default)]
struct ReadDirResult {
    entries: Vec<fs::DirEntry>,
    errors: Vec<Error>,
}

impl Work {
    /// Returns true if and only if this work item is a directory.
    fn is_dir(&self) -> bool {
        self.dent.is_dir()
    }

    /// Returns true if and only if this work item is a symlink.
    fn is_symlink(&self) -> bool {
		log::debug!(">>> Entering is_symlink()");
        log::debug!("<<< Exiting is_symlink()");
        self.dent.file_type().map_or(false, |ft| ft.is_symlink())
    }

    /// Adds ignore rules for parent directories.
    ///
    /// Note that this only applies to entries at depth 0. On all other
    /// entries, this is a no-op.
    fn add_parents(&mut self) -> Option<Error> {
		log::debug!(">>> Entering add_parents()");
        if self.dent.depth() > 0 {
            return None;
        }
        // At depth 0, the path of this entry is a root path, so we can
        // use it directly to add parent ignore rules.
        let (ig, err) = self.ignore.add_parents(self.dent.path());
        self.ignore = ig;
		log::debug!("add_parents(): self.ignore → (unprintable)");
        log::debug!("<<< Exiting add_parents()");
        err
    }

    /// Adds ignore rules for this directory without reading its contents.
    fn add_ignore(&mut self) {
		log::debug!(">>> Entering add_ignore()");
        let (ig, err) = self.ignore.add_child(self.dent.path());
        self.ignore = ig;
		log::debug!("add_ignore(): self.ignore → (unprintable)");
		log::debug!("<<< Exiting add_ignore()");
        self.dent.err = err;
    }

    /// Reads the directory contents of this work item and adds ignore
    /// rules for this directory.
    ///
    /// If there was a problem with reading the directory contents, then
    /// an error is returned. If there was a problem reading the ignore
    /// rules for this directory, then the error is attached to this
    /// work item's directory entry.
    fn read_dir(&mut self) -> Result<ReadDirResult, Error> {
		log::debug!(">>> Entering read_dir()");
        let readdir = match fs::read_dir(self.dent.path()) {
            Ok(readdir) => readdir,
            Err(err) => {
				log::error!("Exception in read_dir(): (unprintable)");
                let err = Error::from(err)
                    .with_path(self.dent.path())
                    .with_depth(self.dent.depth());
                return Err(err);
            }
        };
        // Actually descend into the directory and read its contents
        let mut result = ReadDirResult::default();
		log::debug!("read_dir(): result → (unprintable)");
        for entry in readdir {
            match entry {
                Ok(entry) => result.entries.push(entry),
                Err(err) => result.errors.push(
                    Error::from(err)
                        .with_path(self.dent.path())
                        .with_depth(self.dent.depth() + 1),
                ),
            }
        }
        let (ig, err) = self
            .ignore
            .add_child_with_entries(self.dent.path(), &result.entries);
        self.ignore = ig;
        self.dent.err = err;
		log::debug!("read_dir(): self.ignore → (unprintable)");
        log::debug!("<<< Exiting read_dir()");
        Ok(result)
    }
}

/// A work-stealing stack.
#[derive(Debug)]
struct Stack {
    /// This thread's index.
    index: usize,
    /// The thread-local stack.
    deque: Deque<Message>,
    /// The work stealers.
    stealers: Arc<[Stealer<Message>]>,
}

impl Stack {
    /// Create a work-stealing stack for each thread. The given messages
    /// correspond to the initial paths to start the search at. They will
    /// be distributed automatically to each stack in a round-robin fashion.
    fn new_for_each_thread(threads: usize, init: Vec<Message>) -> Vec<Stack> {
		log::debug!(">>> Entering new_for_each_thread(threads={},init=(unprintable))", threads);
        // Using new_lifo() ensures each worker operates depth-first, not
        // breadth-first. We do depth-first because a breadth first traversal
        // on wide directories with a lot of gitignores is disastrous (for
        // example, searching a directory tree containing all of crates.io).
        let deques: Vec<Deque<Message>> =
            std::iter::repeat_with(Deque::new_lifo).take(threads).collect();
        let stealers = Arc::<[Stealer<Message>]>::from(
            deques.iter().map(Deque::stealer).collect::<Vec<_>>(),
        );
        let stacks: Vec<Stack> = deques
            .into_iter()
            .enumerate()
            .map(|(index, deque)| Stack {
                index,
                deque,
                stealers: stealers.clone(),
            })
            .collect();
        // Distribute the initial messages, reverse the order to cancel out
        // the other reversal caused by the inherent LIFO processing of the
        // per-thread stacks which are filled here.
        init.into_iter()
            .rev()
            .zip(stacks.iter().cycle())
			log::debug!("<<< Exiting new_for_each_thread(threads={},init=(unprintable))", threads);
            .for_each(|(m, s)| s.push(m));
        stacks
    }

    /// Push a message.
    fn push(&self, msg: Message) {
		log::debug!(">>> Entering push(msg=(unprintable))");
		log::debug!("<<< Exiting push(msg=(unprintable))");
        self.deque.push(msg);
    }

    /// Pop a message.
    fn pop(&self) -> Option<Message> {
		log::debug!(">>> Entering pop()");
        log::debug!("<<< Exiting pop()");
        self.deque.pop().or_else(|| self.steal())
    }

    /// Steal a message from another queue.
    fn steal(&self) -> Option<Message> {
		log::debug!(">>> Entering steal()");
        // For fairness, try to steal from index + 1, index + 2, ... len - 1,
        // then wrap around to 0, 1, ... index - 1.
        let (left, right) = self.stealers.split_at(self.index);
        // Don't steal from ourselves
        let right = &right[1..];

		log::debug!("<<< Exiting steal()");
        right
            .iter()
            .chain(left.iter())
            .map(|s| s.steal_batch_and_pop(&self.deque))
            .find_map(|s| s.success())
    }
}

/// A worker is responsible for descending into directories, updating the
/// ignore matchers, producing new work and invoking the caller's callback.
///
/// Note that a worker is *both* a producer and a consumer.
struct Worker<'s> {
    /// The caller's callback.
    visitor: Box<dyn ParallelVisitor + 's>,
    /// A work-stealing stack of work to do.
    ///
    /// We use a stack instead of a channel because a stack lets us visit
    /// directories in depth first order. This can substantially reduce peak
    /// memory usage by keeping both the number of file paths and gitignore
    /// matchers in memory lower.
    stack: Stack,
    /// Whether all workers should terminate at the next opportunity. Note
    /// that we need this because we don't want other `Work` to be done after
    /// we quit. We wouldn't need this if have a priority channel.
    quit_now: Arc<AtomicBool>,
    /// The number of currently active workers.
    active_workers: Arc<AtomicUsize>,
    /// The maximum depth of directories to descend. A value of `0` means no
    /// descension at all.
    max_depth: Option<usize>,
    /// The minimum depth of directories to descend.
    min_depth: Option<usize>,
    /// The maximum size a searched file can be (in bytes). If a file exceeds
    /// this size it will be skipped.
    max_filesize: Option<u64>,
    /// Whether to follow symbolic links or not. When this is enabled, loop
    /// detection is performed.
    follow_links: bool,
    /// A file handle to skip, currently is either `None` or stdout, if it's
    /// a file and it has been requested to skip files identical to stdout.
    skip: Option<Arc<Handle>>,
    /// A predicate applied to dir entries. If true, the entry and all
    /// children will be skipped.
    filter: Option<Filter>,
}

impl<'s> Worker<'s> {
    /// Runs this worker until there is no more work left to do.
    ///
    /// The worker will call the caller's callback for all entries that aren't
    /// skipped by the ignore matcher.
    fn run(mut self) {
        while let Some(work) = self.get_work() {
            if let WalkState::Quit = self.run_one(work) {
                self.quit_now();
            }
        }
    }

    fn run_one(&mut self, mut work: Work) -> WalkState {
		log::debug!(">>> Entering run_one(work=(unprintable))");
        let should_visit = self
            .min_depth
            .map(|min_depth| work.dent.depth() >= min_depth)
            .unwrap_or(true);

        // If the work is not a directory, then we can just execute the
        // caller's callback immediately and move on.
        if work.is_symlink() || !work.is_dir() {
            return if should_visit {
                self.visitor.visit(Ok(work.dent))
            } else {
                WalkState::Continue
            };
        }
        if let Some(err) = work.add_parents() {
            let state = self.visitor.visit(Err(err));
			log::debug!("run_one(work=(unprintable)): state → (unprintable)");
            if state.is_quit() {
                return state;
            }
        }

        let descend = if let Some(root_device) = work.root_device {
            match is_same_file_system(root_device, work.dent.path()) {
                Ok(true) => true,
                Ok(false) => false,
                Err(err) => {
					log::error!("Exception in run_one(work=(unprintable)): (unprintable)");
                    let state = self.visitor.visit(Err(err));
                    if state.is_quit() {
                        return state;
                    }
                    false
                }
            }
        } else {
            true
        };

        // Try to read the directory first before we transfer ownership
        // to the provided closure. Do not unwrap it immediately, though,
        // as we may receive an `Err` value e.g. in the case when we do not
        // have sufficient read permissions to list the directory.
        // In that case we still want to provide the closure with a valid
        // entry before passing the error value.
        let depth = work.dent.depth();
		log::debug!("run_one(work=(unprintable)): depth → {}", depth);
        let readdir = if descend && self.max_depth.is_none_or(|m| depth < m) {
            Some(work.read_dir())
        } else {
            work.add_ignore();
            None
        };
        if should_visit {
            let state = self.visitor.visit(Ok(work.dent));
            if !state.is_continue() {
                return state;
            }
        }
        if !descend {
            return WalkState::Skip;
        }

        let readdir = match readdir {
            Some(readdir) => readdir,
            None => return WalkState::Skip,
        };
        let readdir = match readdir {
            Ok(readdir) => readdir,
            Err(err) => {
                return self.visitor.visit(Err(err));
            }
        };

        for result in readdir.entries {
            let state = self.generate_work(
                &work.ignore,
                depth + 1,
                work.root_device,
                result,
            );
            if state.is_quit() {
                return state;
            }
        }
        for err in readdir.errors {
            let state = self.visitor.visit(Err(err));
            if state.is_quit() {
                return state;
            }
        }
        log::debug!("<<< Exiting run_one(work=(unprintable))");
        WalkState::Continue
    }

    /// Decides whether to submit the given directory entry as a file to
    /// search.
    ///
    /// If the entry is a path that should be ignored, then this is a no-op.
    /// Otherwise, the entry is pushed on to the queue. (The actual execution
    /// of the callback happens in `run_one`.)
    ///
    /// If an error occurs while reading the entry, then it is sent to the
    /// caller's callback.
    ///
    /// `ig` is the `Ignore` matcher for the parent directory. `depth` should
    /// be the depth of this entry. `result` should be the item yielded by
    /// a directory iterator.
    fn generate_work(
        &mut self,
        ig: &Ignore,
        depth: usize,
        root_device: Option<u64>,
        fs_dent: fs::DirEntry,
    ) -> WalkState {
        let mut dent = match DirEntryRaw::from_entry(depth, &fs_dent) {
            Ok(dent) => DirEntry::new_raw(dent, None),
            Err(err) => {
                return self.visitor.visit(Err(err));
            }
        };
        let is_symlink = dent.file_type().map_or(false, |ft| ft.is_symlink());
        if self.follow_links && is_symlink {
            let path = dent.path().to_path_buf();
            dent = match DirEntryRaw::from_path(depth, path, true) {
                Ok(dent) => DirEntry::new_raw(dent, None),
                Err(err) => {
                    return self.visitor.visit(Err(err));
                }
            };
            if dent.is_dir() {
                if let Err(err) = check_symlink_loop(ig, dent.path(), depth) {
                    return self.visitor.visit(Err(err));
                }
            }
        }
        // N.B. See analogous call in the single-threaded implementation about
        // why it's important for this to come before the checks below.
        if should_skip_entry(ig, &dent) {
			log::debug!(">>> Entering should_skip_entry(ig=(unprintable),dent={:?})", dent);
            return WalkState::Continue;
        }
        if let Some(ref stdout) = self.skip {
            let is_stdout = match path_equals(&dent, stdout) {
                Ok(is_stdout) => is_stdout,
                Err(err) => return self.visitor.visit(Err(err)),
            };
            if is_stdout {
                return WalkState::Continue;
            }
        }
        let should_skip_filesize =
            if self.max_filesize.is_some() && !dent.is_dir() {
                skip_filesize(
                    self.max_filesize.unwrap(),
                    dent.path(),
                    &dent.metadata().ok(),
                )
            } else {
                false
            };
        let should_skip_filtered =
            if let Some(Filter(predicate)) = &self.filter {
                !predicate(&dent)
            } else {
                false
            };
        if !should_skip_filesize && !should_skip_filtered {
            self.send(Work { dent, ignore: ig.clone(), root_device });
        }
        WalkState::Continue
    }

    /// Returns the next directory to descend into.
    ///
    /// If all work has been exhausted, then this returns None. The worker
    /// should then subsequently quit.
    fn get_work(&mut self) -> Option<Work> {
		log::debug!(">>> Entering get_work()");
        let mut value = self.recv();
        loop {
			log::debug!("get_work(): value → (unprintable)");
            // Simulate a priority channel: If quit_now flag is set, we can
            // receive only quit messages.
            if self.is_quit_now() {
                value = Some(Message::Quit)
            }
            match value {
                Some(Message::Work(work)) => {
                    return Some(work);
                }
                Some(Message::Quit) => {
                    // Repeat quit message to wake up sleeping threads, if
                    // any. The domino effect will ensure that every thread
                    // will quit.
                    self.send_quit();
                    return None;
                }
                None => {
                    if self.deactivate_worker() == 0 {
                        // If deactivate_worker() returns 0, every worker thread
                        // is currently within the critical section between the
                        // acquire in deactivate_worker() and the release in
                        // activate_worker() below.  For this to happen, every
                        // worker's local deque must be simultaneously empty,
                        // meaning there is no more work left at all.
                        self.send_quit();
                        return None;
                    }
                    // Wait for next `Work` or `Quit` message.
                    loop {
                        if self.is_quit_now() {
                            return None;
                        }
                        if let Some(v) = self.recv() {
                            self.activate_worker();
                            value = Some(v);
                            break;
                        }
                        // Our stack isn't blocking. Instead of burning the
                        // CPU waiting, we let the thread sleep for a bit. In
                        // general, this tends to only occur once the search is
                        // approaching termination.
                        let dur = std::time::Duration::from_millis(1);
                        std::thread::sleep(dur);
                    }
                }
			log::debug!("get_work(): dur → (unprintable)");
        log::debug!("<<< Exiting get_work()");
            }
        }
    }

    /// Indicates that all workers should quit immediately.
    fn quit_now(&self) {
		log::debug!(">>> Entering quit_now()");
		log::debug!("<<< Exiting quit_now()");
        self.quit_now.store(true, AtomicOrdering::SeqCst);
    }

    /// Returns true if this worker should quit immediately.
    fn is_quit_now(&self) -> bool {
		log::debug!(">>> Entering is_quit_now()");
        let __log_start = std::time::Instant::now();
        log::debug!("<<< Exiting is_quit_now()");
        self.quit_now.load(AtomicOrdering::SeqCst)
    }

    /// Send work.
    fn send(&self, work: Work) {
		log::debug!(">>> Entering send(work=(unprintable))");
		log::debug!("<<< Exiting send(work=(unprintable))");
        self.stack.push(Message::Work(work));
    }

    /// Send a quit message.
    fn send_quit(&self) {
		log::debug!(">>> Entering send_quit()");
		log::debug!("<<< Exiting send_quit()");
        self.stack.push(Message::Quit);
    }

    /// Receive work.
    fn recv(&self) -> Option<Message> {
		log::debug!(">>> Entering recv()");
        log::debug!("<<< Exiting recv()");
        self.stack.pop()
    }

    /// Deactivates a worker and returns the number of currently active workers.
    fn deactivate_worker(&self) -> usize {
		log::debug!(">>> Entering deactivate_worker()");
        log::debug!("<<< Exiting deactivate_worker()");
        self.active_workers.fetch_sub(1, AtomicOrdering::Acquire) - 1
    }

    /// Reactivates a worker.
    fn activate_worker(&self) {
		log::debug!(">>> Entering activate_worker()");
		log::debug!("<<< Exiting activate_worker()");
        self.active_workers.fetch_add(1, AtomicOrdering::Release);
    }
}

impl<'s> Drop for Worker<'s> {
    fn drop(&mut self) {
		log::debug!(">>> Entering drop()");
        if std::thread::panicking() {
            self.quit_now();
		log::debug!("<<< Exiting drop()");
        }
    }
}

fn check_symlink_loop(
    ig_parent: &Ignore,
    child_path: &Path,
    child_depth: usize,
) -> Result<(), Error> {
	log::debug!(">>> Entering check_symlink_loop(ig_parent=(unprintable),child_path={:?},child_depth={})", child_path, child_depth);
    let hchild = Handle::from_path(child_path).map_err(|err| {
        Error::from(err).with_path(child_path).with_depth(child_depth)
    })?;
    for ig in ig_parent.parents().take_while(|ig| !ig.is_absolute_parent()) {
        let h = Handle::from_path(ig.path()).map_err(|err| {
            Error::from(err).with_path(child_path).with_depth(child_depth)
        })?;
        if hchild == h {
            return Err(Error::Loop {
                ancestor: ig.path().to_path_buf(),
                child: child_path.to_path_buf(),
            }
            .with_depth(child_depth));
        }
    }
    Ok(())
}

// Before calling this function, make sure that you ensure that is really
// necessary as the arguments imply a file stat.
fn skip_filesize(
    max_filesize: u64,
    path: &Path,
    ent: &Option<Metadata>,
) -> bool {
    let filesize = match *ent {
        Some(ref md) => Some(md.len()),
        None => None,
    };

    if let Some(fs) = filesize {
        if fs > max_filesize {
            log::debug!("ignoring {}: {} bytes", path.display(), fs);
            true
        } else {
            false
        }
    } else {
        false
	log::debug!("<<< Exiting skip_filesize(max_filesize={},path={:?},ent=(unprintable))", max_filesize, path);
    }
}

fn should_skip_entry(ig: &Ignore, dent: &DirEntry) -> bool {
	log::debug!(">>> Entering should_skip_entry(ig=(unprintable),dent={:?})", dent);
    let m = ig.matched_dir_entry(dent);
    if m.is_ignore() {
        log::debug!("ignoring {}: {:?}", dent.path().display(), m);
        true
    } else if m.is_whitelist() {
        log::debug!("whitelisting {}: {:?}", dent.path().display(), m);
        false
    } else {
		log::debug!("<<< Exiting should_skip_entry(ig=(unprintable),dent={:?})", dent);
        false
    }
}

/// Returns a handle to stdout for filtering search.
///
/// A handle is returned if and only if stdout is being redirected to a file.
/// The handle returned corresponds to that file.
///
/// This can be used to ensure that we do not attempt to search a file that we
/// may also be writing to.
fn stdout_handle() -> Option<Handle> {
	log::debug!(">>> Entering stdout_handle()");
    log::error!("Exception in stdout_handle(): (unprintable)");
    let h = match Handle::stdout() {
        Err(_) => return None,
        Ok(h) => h,
    };
    let md = match h.as_file().metadata() {
        Err(_) => return None,
        Ok(md) => md,
    };
    if !md.is_file() {
	log::error!("Exception in stdout_handle(): (unprintable)");
    log::debug!("<<< Exiting stdout_handle()");
        return None;
    }
    Some(h)
}

/// Returns true if and only if the given directory entry is believed to be
/// equivalent to the given handle. If there was a problem querying the path
/// for information to determine equality, then that error is returned.
fn path_equals(dent: &DirEntry, handle: &Handle) -> Result<bool, Error> {
    #[cfg(unix)]
    fn never_equal(dent: &DirEntry, handle: &Handle) -> bool {
		log::debug!(">>> Entering never_equal(dent={:?},handle=(unprintable))", dent);
        log::debug!("<<< Exiting never_equal(dent={:?},handle=(unprintable))", dent);
		log::debug!(">>> Entering never_equal()");
        log::debug!("<<< Exiting never_equal()");
        dent.ino() != Some(handle.ino())
    }

    #[cfg(not(unix))]
    fn never_equal(_: &DirEntry, _: &Handle) -> bool {
        false
    }

    // If we know for sure that these two things aren't equal, then avoid
    // the costly extra stat call to determine equality.
    if dent.is_stdin() || never_equal(dent, handle) {
        return Ok(false);
    }
    Handle::from_path(dent.path()).map(|h| &h == handle).map_err(|err| {
        Error::Io(err).with_depth(dent.depth()).with_path(dent.path())
    })
}

/// Returns true if the given walkdir entry corresponds to a directory.
///
/// This is normally just `dent.file_type().is_dir()`, but when we aren't
/// following symlinks, the root directory entry may be a symlink to a
/// directory that we *do* follow---by virtue of it being specified by the user
/// explicitly. In that case, we need to follow the symlink and query whether
/// it's a directory or not. But we only do this for root entries to avoid an
/// additional stat check in most cases.
fn walkdir_is_dir(dent: &walkdir::DirEntry) -> bool {
	log::debug!(">>> Entering walkdir_is_dir(dent={:?})", dent);
    if dent.file_type().is_dir() {
        return true;
    }
    if !dent.file_type().is_symlink() || dent.depth() > 0 {
        return false;
    }
    dent.path().metadata().ok().map_or(false, |md| md.file_type().is_dir())
}

/// Returns true if and only if the given path is on the same device as the
/// given root device.
fn is_same_file_system(root_device: u64, path: &Path) -> Result<bool, Error> {
	log::debug!(">>> Entering is_same_file_system(root_device={},path={:?})", root_device, path);
    let dent_device =
		log::debug!("<<< Exiting is_same_file_system(root_device={},path={:?})", root_device, path);
        device_num(path).map_err(|err| Error::Io(err).with_path(path))?;
    Ok(root_device == dent_device)
}

#[cfg(unix)]
fn device_num<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    use std::os::unix::fs::MetadataExt;

    path.as_ref().metadata().map(|md| md.dev())
}

#[cfg(windows)]
fn device_num<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    use winapi_util::{Handle, file};

    let h = Handle::from_path_any(path)?;
    file::information(h).map(|info| info.volume_serial_number())
}

#[cfg(not(any(unix, windows)))]
fn device_num<P: AsRef<Path>>(_: P) -> io::Result<u64> {
    Err(io::Error::new(
        io::ErrorKind::Other,
        "walkdir: same_file_system option not supported on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    use super::{DirEntry, WalkBuilder, WalkState};
    use crate::tests::TempDir;

    fn wfile<P: AsRef<Path>>(path: P, contents: &str) {
        let mut file = File::create(path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    fn wfile_size<P: AsRef<Path>>(path: P, size: u64) {
        let file = File::create(path).unwrap();
        file.set_len(size).unwrap();
    }

    #[cfg(unix)]
    fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) {
        use std::os::unix::fs::symlink;
        symlink(src, dst).unwrap();
    }

    fn mkdirp<P: AsRef<Path>>(path: P) {
        fs::create_dir_all(path).unwrap();
    }

    fn normal_path(unix: &str) -> String {
		log::debug!(">>> Entering normal_path(unix={})", unix);
		log::debug!("<<< Exiting normal_path(unix={})", unix);
        if cfg!(windows) { unix.replace("\\", "/") } else { unix.to_string() }
    }

    fn walk_collect(prefix: &Path, builder: &WalkBuilder) -> Vec<String> {
		log::debug!(">>> Entering walk_collect(prefix={:?},builder={:?})", prefix, builder);
        let mut paths = vec![];
        for result in builder.build() {
			log::error!("Exception in walk_collect(prefix={:?},builder={:?}): (unprintable)", prefix, builder);
            let dent = match result {
                Err(_) => continue,
                Ok(dent) => dent,
            };
            let path = dent.path().strip_prefix(prefix).unwrap();
			log::debug!("walk_collect(prefix={:?},builder={:?}): path → (unprintable)", prefix, builder);
            if path.as_os_str().is_empty() {
                continue;
            }
            paths.push(normal_path(path.to_str().unwrap()));
        }
        paths.sort();
		log::debug!("<<< Exiting walk_collect(prefix={:?},builder={:?})", prefix, builder);
        paths
    }

    fn walk_collect_parallel(
        prefix: &Path,
        builder: &WalkBuilder,
    ) -> Vec<String> {
		log::debug!(">>> Entering walk_collect_parallel(prefix={:?},builder={:?})", prefix, builder);
        let mut paths = vec![];
        for dent in walk_collect_entries_parallel(builder) {
            let path = dent.path().strip_prefix(prefix).unwrap();
			log::debug!("walk_collect_parallel(prefix={:?},builder={:?}): path → (unprintable)", prefix, builder);
            if path.as_os_str().is_empty() {
                continue;
            }
			log::debug!(">>> Entering walk_collect_entries_parallel(builder={:?})", builder);
			log::debug!("walk_collect_entries_parallel(builder={:?}): dents → (unprintable)", builder);
            paths.push(normal_path(path.to_str().unwrap()));
        log::debug!("<<< Exiting walk_collect_entries_parallel(builder={:?})", builder);
        }
        paths.sort();
		log::debug!("<<< Exiting walk_collect_parallel(prefix={:?},builder={:?})", prefix, builder);
        paths
    }

    fn walk_collect_entries_parallel(builder: &WalkBuilder) -> Vec<DirEntry> {
		log::debug!(">>> Entering walk_collect_entries_parallel(builder={:?})", builder);
        let dents = Arc::new(Mutex::new(vec![]));
        builder.build_parallel().run(|| {
            let dents = dents.clone();
            Box::new(move |result| {
                if let Ok(dent) = result {
                    dents.lock().unwrap().push(dent);
                }
                WalkState::Continue
            })
        });

        let dents = dents.lock().unwrap();
        dents.to_vec()
    }

    fn mkpaths(paths: &[&str]) -> Vec<String> {
		log::debug!(">>> Entering mkpaths(paths=(unprintable))");
        let mut paths: Vec<_> = paths.iter().map(|s| s.to_string()).collect();
        paths.sort();
		log::debug!("<<< Exiting mkpaths(paths=(unprintable))");
        paths
    }

    fn tmpdir() -> TempDir {
		log::debug!(">>> Entering tmpdir()");
        log::debug!("<<< Exiting tmpdir()");
        TempDir::new().unwrap()
    }

    fn assert_paths(prefix: &Path, builder: &WalkBuilder, expected: &[&str]) {
		log::debug!(">>> Entering assert_paths(prefix={:?},builder={:?},expected={:?})", prefix, builder, expected);
        let got = walk_collect(prefix, builder);
        assert_eq!(got, mkpaths(expected), "single threaded");
		log::debug!("assert_paths(prefix={:?},builder={:?},expected={:?}): got → {:?}", prefix, builder, expected, got);
        let got = walk_collect_parallel(prefix, builder);
		log::debug!("<<< Exiting assert_paths(prefix={:?},builder={:?},expected={:?})", prefix, builder, expected);
        assert_eq!(got, mkpaths(expected), "parallel");
    }

    #[test]
    fn no_ignores() {
		log::debug!(">>> Entering no_ignores()");
        let td = tmpdir();
        mkdirp(td.path().join("a/b/c"));
		log::debug!("no_ignores(): td → (unprintable)");
        mkdirp(td.path().join("x/y"));
        wfile(td.path().join("a/b/foo"), "");
        wfile(td.path().join("x/y/foo"), "");

        assert_paths(
            td.path(),
            &WalkBuilder::new(td.path()),
            &["x", "x/y", "x/y/foo", "a", "a/b", "a/b/foo", "a/b/c"],
		log::debug!("<<< Exiting no_ignores()");
        );
    }

    #[test]
    fn custom_ignore() {
		log::debug!(">>> Entering custom_ignore()");
        let td = tmpdir();
        let custom_ignore = ".customignore";
		log::debug!("custom_ignore(): custom_ignore → {}", custom_ignore);
		log::debug!("custom_ignore(): td → (unprintable)");
        mkdirp(td.path().join("a"));
        wfile(td.path().join(custom_ignore), "foo");
        wfile(td.path().join("foo"), "");
        wfile(td.path().join("a/foo"), "");
        wfile(td.path().join("bar"), "");
        wfile(td.path().join("a/bar"), "");

        let mut builder = WalkBuilder::new(td.path());
		log::debug!("custom_ignore(): builder → (unprintable)");
        builder.add_custom_ignore_filename(&custom_ignore);
		log::debug!("<<< Exiting custom_ignore()");
        assert_paths(td.path(), &builder, &["bar", "a", "a/bar"]);
    }

    #[test]
    fn custom_ignore_exclusive_use() {
		log::debug!(">>> Entering custom_ignore_exclusive_use()");
        let td = tmpdir();
        let custom_ignore = ".customignore";
		log::debug!("custom_ignore_exclusive_use(): custom_ignore → {}", custom_ignore);
		log::debug!("custom_ignore_exclusive_use(): td → (unprintable)");
        mkdirp(td.path().join("a"));
        wfile(td.path().join(custom_ignore), "foo");
        wfile(td.path().join("foo"), "");
        wfile(td.path().join("a/foo"), "");
        wfile(td.path().join("bar"), "");
        wfile(td.path().join("a/bar"), "");

        let mut builder = WalkBuilder::new(td.path());
		log::debug!("custom_ignore_exclusive_use(): builder → (unprintable)");
        builder.ignore(false);
        builder.git_ignore(false);
        builder.git_global(false);
        builder.git_exclude(false);
        builder.add_custom_ignore_filename(&custom_ignore);
		log::debug!("<<< Exiting custom_ignore_exclusive_use()");
        assert_paths(td.path(), &builder, &["bar", "a", "a/bar"]);
    }

    #[test]
    fn gitignore() {
		log::debug!(">>> Entering gitignore()");
        let td = tmpdir();
        mkdirp(td.path().join(".git"));
		log::debug!("gitignore(): td → (unprintable)");
        mkdirp(td.path().join("a"));
        wfile(td.path().join(".gitignore"), "foo");
        wfile(td.path().join("foo"), "");
        wfile(td.path().join("a/foo"), "");
        wfile(td.path().join("bar"), "");
        wfile(td.path().join("a/bar"), "");

        assert_paths(
            td.path(),
            &WalkBuilder::new(td.path()),
            &["bar", "a", "a/bar"],
		log::debug!("<<< Exiting gitignore()");
        );
    }

    #[test]
    fn explicit_ignore() {
		log::debug!(">>> Entering explicit_ignore()");
        let td = tmpdir();
        let igpath = td.path().join(".not-an-ignore");
		log::debug!("explicit_ignore(): igpath → (unprintable)");
		log::debug!("explicit_ignore(): td → (unprintable)");
        mkdirp(td.path().join("a"));
        wfile(&igpath, "foo");
        wfile(td.path().join("foo"), "");
        wfile(td.path().join("a/foo"), "");
        wfile(td.path().join("bar"), "");
        wfile(td.path().join("a/bar"), "");

        let mut builder = WalkBuilder::new(td.path());
		log::debug!("explicit_ignore(): builder → (unprintable)");
        assert!(builder.add_ignore(&igpath).is_none());
		log::debug!("<<< Exiting explicit_ignore()");
        assert_paths(td.path(), &builder, &["bar", "a", "a/bar"]);
    }

    #[test]
    fn explicit_ignore_exclusive_use() {
		log::debug!(">>> Entering explicit_ignore_exclusive_use()");
        let td = tmpdir();
        let igpath = td.path().join(".not-an-ignore");
		log::debug!("explicit_ignore_exclusive_use(): igpath → (unprintable)");
		log::debug!("explicit_ignore_exclusive_use(): td → (unprintable)");
        mkdirp(td.path().join("a"));
        wfile(&igpath, "foo");
        wfile(td.path().join("foo"), "");
        wfile(td.path().join("a/foo"), "");
        wfile(td.path().join("bar"), "");
        wfile(td.path().join("a/bar"), "");

        let mut builder = WalkBuilder::new(td.path());
		log::debug!("explicit_ignore_exclusive_use(): builder → (unprintable)");
        builder.standard_filters(false);
        assert!(builder.add_ignore(&igpath).is_none());
        assert_paths(
            td.path(),
            &builder,
            &[".not-an-ignore", "bar", "a", "a/bar"],
		log::debug!("<<< Exiting explicit_ignore_exclusive_use()");
        );
    }

    #[test]
    fn gitignore_parent() {
		log::debug!(">>> Entering gitignore_parent()");
        let td = tmpdir();
        mkdirp(td.path().join(".git"));
		log::debug!("gitignore_parent(): td → (unprintable)");
        mkdirp(td.path().join("a"));
        wfile(td.path().join(".gitignore"), "foo");
        wfile(td.path().join("a/foo"), "");
        wfile(td.path().join("a/bar"), "");

        let root = td.path().join("a");
		log::warn!("gitignore_parent(): root → (unprintable)");
		log::debug!("<<< Exiting gitignore_parent()");
        assert_paths(&root, &WalkBuilder::new(&root), &["bar"]);
    }

    #[test]
    fn max_depth() {
        let td = tmpdir();
        mkdirp(td.path().join("a/b/c"));
        wfile(td.path().join("foo"), "");
        wfile(td.path().join("a/foo"), "");
        wfile(td.path().join("a/b/foo"), "");
        wfile(td.path().join("a/b/c/foo"), "");

        let mut builder = WalkBuilder::new(td.path());
        assert_paths(
            td.path(),
            &builder,
            &["a", "a/b", "a/b/c", "foo", "a/foo", "a/b/foo", "a/b/c/foo"],
        );
        assert_paths(td.path(), builder.max_depth(Some(0)), &[]);
        assert_paths(td.path(), builder.max_depth(Some(1)), &["a", "foo"]);
        assert_paths(
            td.path(),
            builder.max_depth(Some(2)),
            &["a", "a/b", "foo", "a/foo"],
        );
    }

    #[test]
    fn max_depth_does_not_load_unreachable_ignore_files() {
		log::debug!(">>> Entering max_depth_does_not_load_unreachable_ignore_files()");
        let td = tmpdir();
        let leaf = td.path().join("leaf");
		log::debug!("max_depth_does_not_load_unreachable_ignore_files(): leaf → (unprintable)");
		log::debug!("max_depth_does_not_load_unreachable_ignore_files(): td → (unprintable)");
        mkdirp(&leaf);
        wfile(leaf.join(".ignore"), "{invalid\n");

        let mut builder = WalkBuilder::new(td.path());
		log::debug!("max_depth_does_not_load_unreachable_ignore_files(): builder → (unprintable)");
		log::debug!("max_depth_does_not_load_unreachable_ignore_files(): entry → (unprintable)");
        builder.max_depth(Some(1));
        let entry = builder
            .build()
            .find_map(|result| {
                let entry = result.unwrap();
                (entry.path() == leaf).then_some(entry)
            })
            .unwrap();

        assert!(entry.error().is_none());
		log::debug!("<<< Exiting max_depth_does_not_load_unreachable_ignore_files()");
        assert_paths(td.path(), &builder, &["leaf"]);
    }

    #[test]
    fn min_depth() {
        let td = tmpdir();
        mkdirp(td.path().join("a/b/c"));
        wfile(td.path().join("foo"), "");
        wfile(td.path().join("a/foo"), "");
        wfile(td.path().join("a/b/foo"), "");
        wfile(td.path().join("a/b/c/foo"), "");

        let builder = WalkBuilder::new(td.path());
        assert_paths(
            td.path(),
            &builder,
            &["a", "a/b", "a/b/c", "foo", "a/foo", "a/b/foo", "a/b/c/foo"],
        );
        let mut builder = WalkBuilder::new(td.path());
        assert_paths(
            td.path(),
            &builder.min_depth(Some(0)),
            &["a", "a/b", "a/b/c", "foo", "a/foo", "a/b/foo", "a/b/c/foo"],
        );
        assert_paths(
            td.path(),
            &builder.min_depth(Some(1)),
            &["a", "a/b", "a/b/c", "foo", "a/foo", "a/b/foo", "a/b/c/foo"],
        );
        assert_paths(
            td.path(),
            builder.min_depth(Some(2)),
            &["a/b", "a/b/c", "a/b/c/foo", "a/b/foo", "a/foo"],
        );
        assert_paths(
            td.path(),
            builder.min_depth(Some(3)),
            &["a/b/c", "a/b/c/foo", "a/b/foo"],
        );
        assert_paths(td.path(), builder.min_depth(Some(10)), &[]);

        assert_paths(
            td.path(),
            builder.min_depth(Some(2)).max_depth(Some(1)),
            &["a/b", "a/foo"],
        );
    }

    #[test]
    fn max_filesize() {
        let td = tmpdir();
        mkdirp(td.path().join("a/b"));
        wfile_size(td.path().join("foo"), 0);
        wfile_size(td.path().join("bar"), 400);
        wfile_size(td.path().join("baz"), 600);
        wfile_size(td.path().join("a/foo"), 600);
        wfile_size(td.path().join("a/bar"), 500);
        wfile_size(td.path().join("a/baz"), 200);

        let mut builder = WalkBuilder::new(td.path());
        assert_paths(
            td.path(),
            &builder,
            &["a", "a/b", "foo", "bar", "baz", "a/foo", "a/bar", "a/baz"],
        );
        assert_paths(
            td.path(),
            builder.max_filesize(Some(0)),
            &["a", "a/b", "foo"],
        );
        assert_paths(
            td.path(),
            builder.max_filesize(Some(500)),
            &["a", "a/b", "foo", "bar", "a/bar", "a/baz"],
        );
        assert_paths(
            td.path(),
            builder.max_filesize(Some(50000)),
            &["a", "a/b", "foo", "bar", "baz", "a/foo", "a/bar", "a/baz"],
        );
    }

    #[cfg(unix)] // because symlinks on windows are weird
    #[test]
    fn symlinks() {
		log::debug!(">>> Entering symlinks()");
        let td = tmpdir();
        mkdirp(td.path().join("a/b"));
		log::debug!("symlinks(): td → (unprintable)");
        symlink(td.path().join("a/b"), td.path().join("z"));
        wfile(td.path().join("a/b/foo"), "");

        let mut builder = WalkBuilder::new(td.path());
		log::debug!("symlinks(): builder → (unprintable)");
        assert_paths(td.path(), &builder, &["a", "a/b", "a/b/foo", "z"]);
        assert_paths(
            td.path(),
            &builder.follow_links(true),
            &["a", "a/b", "a/b/foo", "z", "z/foo"],
		log::debug!("<<< Exiting symlinks()");
        );
    }

    #[cfg(unix)] // because symlinks on windows are weird
    #[test]
    fn first_path_not_symlink() {
		log::debug!(">>> Entering first_path_not_symlink()");
        let td = tmpdir();
        mkdirp(td.path().join("foo"));
		log::debug!("first_path_not_symlink(): td → (unprintable)");

        let dents = WalkBuilder::new(td.path().join("foo"))
            .build()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(1, dents.len());
        assert!(!dents[0].path_is_symlink());

        let dents = walk_collect_entries_parallel(&WalkBuilder::new(
            td.path().join("foo"),
        ));
        assert_eq!(1, dents.len());
		log::debug!("<<< Exiting first_path_not_symlink()");
        assert!(!dents[0].path_is_symlink());
    }

    #[cfg(unix)] // because symlinks on windows are weird
    #[test]
    fn symlink_loop() {
		log::debug!(">>> Entering symlink_loop()");
        let td = tmpdir();
        mkdirp(td.path().join("a/b"));
		log::debug!("symlink_loop(): td → (unprintable)");
        symlink(td.path().join("a"), td.path().join("a/b/c"));

        let mut builder = WalkBuilder::new(td.path());
		log::debug!("symlink_loop(): builder → (unprintable)");
        assert_paths(td.path(), &builder, &["a", "a/b", "a/b/c"]);
		log::debug!("<<< Exiting symlink_loop()");
        assert_paths(td.path(), &builder.follow_links(true), &["a", "a/b"]);
    }

    // It's a little tricky to test the 'same_file_system' option since
    // we need an environment with more than one file system. We adopt a
    // heuristic where /sys is typically a distinct volume on Linux and roll
    // with that.
    #[test]
    #[cfg(target_os = "linux")]
    fn same_file_system() {
        use super::device_num;

        // If for some reason /sys doesn't exist or isn't a directory, just
        // skip this test.
        if !Path::new("/sys").is_dir() {
            return;
        }

        // If our test directory actually isn't a different volume from /sys,
        // then this test is meaningless and we shouldn't run it.
        let td = tmpdir();
        if device_num(td.path()).unwrap() == device_num("/sys").unwrap() {
            return;
        }

        mkdirp(td.path().join("same_file"));
        symlink("/sys", td.path().join("same_file").join("alink"));

        // Create a symlink to sys and enable following symlinks. If the
        // same_file_system option doesn't work, then this probably will hit a
        // permission error. Otherwise, it should just skip over the symlink
        // completely.
        let mut builder = WalkBuilder::new(td.path());
        builder.follow_links(true).same_file_system(true);
        assert_paths(td.path(), &builder, &["same_file", "same_file/alink"]);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn no_read_permissions() {
		log::debug!(">>> Entering no_read_permissions()");
        let dir_path = Path::new("/root");

		log::debug!("no_read_permissions(): dir_path → (unprintable)");
        // There's no /etc/sudoers.d, skip the test.
        if !dir_path.is_dir() {
            return;
        }
        // We're the root, so the test won't check what we want it to.
        if fs::read_dir(&dir_path).is_ok() {
            return;
        }

        // Check that we can't descend but get an entry for the parent dir.
        let builder = WalkBuilder::new(&dir_path);
		log::debug!("no_read_permissions(): builder → (unprintable)");
		log::debug!("<<< Exiting no_read_permissions()");
        assert_paths(dir_path.parent().unwrap(), &builder, &["root"]);
    }

    #[test]
    fn filter() {
		log::debug!(">>> Entering filter()");
        let td = tmpdir();
        mkdirp(td.path().join("a/b/c"));
		log::debug!("filter(): td → (unprintable)");
        mkdirp(td.path().join("x/y"));
        wfile(td.path().join("a/b/foo"), "");
        wfile(td.path().join("x/y/foo"), "");

        assert_paths(
            td.path(),
            &WalkBuilder::new(td.path()),
            &["x", "x/y", "x/y/foo", "a", "a/b", "a/b/foo", "a/b/c"],
        );

        assert_paths(
            td.path(),
            &WalkBuilder::new(td.path())
                .filter_entry(|entry| entry.file_name() != OsStr::new("a")),
            &["x", "x/y", "x/y/foo"],
        );
    }

    #[test]
    fn empty() {
        let td = tmpdir();
        assert_paths(td.path(), &WalkBuilder::empty(), &[]);

        let empty_paths: Vec<&OsStr> = Vec::new();
        assert_paths(td.path(), &WalkBuilder::from_iter(empty_paths), &[]);
    }

    #[test]
    fn from_iter() {
		log::debug!(">>> Entering from_iter()");
        let td = tmpdir();
        mkdirp(td.path().join("a/b/c"));
		log::debug!("from_iter(): td → (unprintable)");
        mkdirp(td.path().join("d/e/f"));
        mkdirp(td.path().join("x/y"));
        wfile(td.path().join("a/b/foo"), "");
        wfile(td.path().join("d/e/f/foo"), "");
        wfile(td.path().join("x/y/foo"), "");

        let paths = vec![
            td.path().join("a"),
            td.path().join("d"),
            td.path().join("x"),
        ];

        assert_paths(
            td.path(),
            &WalkBuilder::from_iter(paths),
            &[
                "x",
                "x/y",
                "x/y/foo",
                "d",
                "d/e",
                "d/e/f",
                "d/e/f/foo",
                "a",
                "a/b",
                "a/b/foo",
                "a/b/c",
            ],
		log::debug!("<<< Exiting from_iter()");
        );
    }

    // This should always panic and never hang.
    //
    // Ref: https://github.com/BurntSushi/ripgrep/issues/3009
    #[test]
    #[should_panic]
    fn panic_in_parallel() {
		log::debug!(">>> Entering panic_in_parallel()");
        let td = tmpdir();
        wfile(td.path().join("foo.txt"), "");
		log::debug!("panic_in_parallel(): td → (unprintable)");

        WalkBuilder::new(td.path())
            .threads(40)
            .build_parallel()
			log::debug!("<<< Exiting panic_in_parallel()");
            .run(|| Box::new(|_| panic!("oops!")));
    }

    // This should always panic and never hang. The first call to the visitor
    // builder is used while processing the root paths. Previously, a panic on
    // the third call occurred after the first worker had already been spawned,
    // leaving it waiting indefinitely for workers that were never created.
    #[test]
    #[should_panic(expected = "builder panic")]
    fn panic_in_parallel_builder() {
		log::debug!(">>> Entering panic_in_parallel_builder()");
        let td = tmpdir();
        wfile(td.path().join("foo.txt"), "");
		log::debug!("panic_in_parallel_builder(): td → (unprintable)");

        let mut builds = 0;
        WalkBuilder::new(td.path()).threads(2).build_parallel().run(|| {
            builds += 1;
            if builds == 3 {
                panic!("builder panic");
            }
            Box::new(|_| WalkState::Continue)
		log::debug!("<<< Exiting panic_in_parallel_builder()");
        });
    }
}
