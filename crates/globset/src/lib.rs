/*!
The globset crate provides cross platform single glob and glob set matching.

Glob set matching is the process of matching one or more glob patterns against
a single candidate path simultaneously, and returning all of the globs that
matched. For example, given this set of globs:

* `*.rs`
* `src/lib.rs`
* `src/**/foo.rs`

and a path `src/bar/baz/foo.rs`, then the set would report the first and third
globs as matching.

# Example: one glob

This example shows how to match a single glob against a single file path.

```
use globset::Glob;

let glob = Glob::new("*.rs")?.compile_matcher();

assert!(glob.is_match("foo.rs"));
assert!(glob.is_match("foo/bar.rs"));
assert!(!glob.is_match("Cargo.toml"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

# Example: configuring a glob matcher

This example shows how to use a `GlobBuilder` to configure aspects of match
semantics. In this example, we prevent wildcards from matching path separators.

```
use globset::GlobBuilder;

let glob = GlobBuilder::new("*.rs")
    .literal_separator(true).build()?.compile_matcher();

assert!(glob.is_match("foo.rs"));
assert!(!glob.is_match("foo/bar.rs")); // no longer matches
assert!(!glob.is_match("Cargo.toml"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

# Example: match multiple globs at once

This example shows how to match multiple glob patterns at once.

```
use globset::{Glob, GlobSetBuilder};

let mut builder = GlobSetBuilder::new();
// A GlobBuilder can be used to configure each glob's match semantics
// independently.
builder.add(Glob::new("*.rs")?);
builder.add(Glob::new("src/lib.rs")?);
builder.add(Glob::new("src/**/foo.rs")?);
let set = builder.build()?;

assert_eq!(set.matches("src/bar/baz/foo.rs"), vec![0, 2]);
# Ok::<(), Box<dyn std::error::Error>>(())
```

# Syntax

Standard Unix-style glob syntax is supported:

* `?` matches any single character. (If the `literal_separator` option is
  enabled, then `?` can never match a path separator.)
* `*` matches zero or more characters. (If the `literal_separator` option is
  enabled, then `*` can never match a path separator.)
* `**` recursively matches directories but are only legal in three situations.
  First, if the glob starts with <code>\*\*&#x2F;</code>, then it matches
  all directories. For example, <code>\*\*&#x2F;foo</code> matches `foo`
  and `bar/foo` but not `foo/bar`. Secondly, if the glob ends with
  <code>&#x2F;\*\*</code>, then it matches all sub-entries. For example,
  <code>foo&#x2F;\*\*</code> matches `foo/a` and `foo/a/b`, but not `foo`.
  Thirdly, if the glob contains <code>&#x2F;\*\*&#x2F;</code> anywhere within
  the pattern, then it matches zero or more directories. Using `**` anywhere
  else is illegal (N.B. the glob `**` is allowed and means "match everything").
* `{a,b}` matches `a` or `b` where `a` and `b` are arbitrary glob patterns.
  (N.B. Nesting `{...}` is not currently allowed.)
* `[ab]` matches `a` or `b` where `a` and `b` are characters. Use
  `[!ab]` to match any character except for `a` and `b`.
* Metacharacters such as `*` and `?` can be escaped with character class
  notation. e.g., `[*]` matches `*`.
* When backslash escapes are enabled, a backslash (`\`) will escape all meta
  characters in a glob. If it precedes a non-meta character, then the slash is
  ignored. A `\\` will match a literal `\\`. Note that this mode is only
  enabled on Unix platforms by default, but can be enabled on any platform
  via the `backslash_escape` setting on `Glob`.

A `GlobBuilder` can be used to prevent wildcards from matching path separators,
or to enable case insensitive matching.

# Crate Features

This crate includes optional features that can be enabled if necessary.
These features are not required but may be useful depending on the use case.

The following features are available:

* **arbitrary** -
  Enabling this feature introduces a public dependency on the
  [`arbitrary`](https://crates.io/crates/arbitrary)
  crate. Namely, it implements the `Arbitrary` trait from that crate for the
  [`Glob`] type. This feature is disabled by default.
*/

#![deny(missing_docs)]

use std::{
    borrow::Cow,
    panic::{RefUnwindSafe, UnwindSafe},
    path::Path,
    sync::Arc,
};

use {
    aho_corasick::AhoCorasick,
    bstr::{B, ByteSlice, ByteVec},
    regex_automata::{
        PatternSet,
        meta::Regex,
        util::pool::{Pool, PoolGuard},
    },
};

use crate::{
    glob::MatchStrategy,
    pathutil::{file_name, file_name_ext, normalize_path},
};

pub use crate::glob::{Glob, GlobBuilder, GlobMatcher};

mod fnv;
mod glob;
mod pathutil;

#[cfg(feature = "serde1")]
mod serde_impl;

#[cfg(feature = "log")]
macro_rules! debug {
    ($($token:tt)*) => (::log::debug!($($token)*);)
}

#[cfg(not(feature = "log"))]
macro_rules! debug {
    ($($token:tt)*) => {};
}

/// Represents an error that can occur when parsing a glob pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    /// The original glob provided by the caller.
    glob: Option<String>,
    /// The kind of error.
    kind: ErrorKind,
}

/// The kind of error that can occur when parsing a glob pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// **DEPRECATED**.
    ///
    /// This error used to occur for consistency with git's glob specification,
    /// but the specification now accepts all uses of `**`. When `**` does not
    /// appear adjacent to a path separator or at the beginning/end of a glob,
    /// it is now treated as two consecutive `*` patterns. As such, this error
    /// is no longer used.
    InvalidRecursive,
    /// Occurs when a character class (e.g., `[abc]`) is not closed.
    UnclosedClass,
    /// Occurs when a range in a character (e.g., `[a-z]`) is invalid. For
    /// example, if the range starts with a lexicographically larger character
    /// than it ends with.
    InvalidRange(char, char),
    /// Occurs when a `}` is found without a matching `{`.
    UnopenedAlternates,
    /// Occurs when a `{` is found without a matching `}`.
    UnclosedAlternates,
    /// **DEPRECATED**.
    ///
    /// This error used to occur when an alternating group was nested inside
    /// another alternating group, e.g., `{{a,b},{c,d}}`. However, this is now
    /// supported and as such this error cannot occur.
    NestedAlternates,
    /// Occurs when an unescaped '\' is found at the end of a glob.
    DanglingEscape,
    /// An error associated with parsing or compiling a regex.
    Regex(String),
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
		log::debug!(">>> Entering description()");
		log::debug!("<<< Exiting description()");
        self.kind.description()
    }
}

impl Error {
    /// Return the glob that caused this error, if one exists.
    pub fn glob(&self) -> Option<&str> {
		log::debug!(">>> Entering glob()");
        log::debug!("<<< Exiting glob()");
        self.glob.as_ref().map(|s| &**s)
    }

    /// Return the kind of this error.
    pub fn kind(&self) -> &ErrorKind {
		log::debug!(">>> Entering kind()");
        log::debug!("<<< Exiting kind()");
        &self.kind
    }
}

impl ErrorKind {
    fn description(&self) -> &str {
        match *self {
            ErrorKind::InvalidRecursive => {
                "invalid use of **; must be one path component"
            }
            ErrorKind::UnclosedClass => {
                "unclosed character class; missing ']'"
            }
            ErrorKind::InvalidRange(_, _) => "invalid character range",
            ErrorKind::UnopenedAlternates => {
                "unopened alternate group; missing '{' \
                (maybe escape '}' with '[}]'?)"
            }
            ErrorKind::UnclosedAlternates => {
                "unclosed alternate group; missing '}' \
                (maybe escape '{' with '[{]'?)"
            }
            ErrorKind::NestedAlternates => {
                "nested alternate groups are not allowed"
            }
            ErrorKind::DanglingEscape => "dangling '\\'",
            ErrorKind::Regex(ref err) => err,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.glob {
            None => self.kind.fmt(f),
            Some(ref glob) => {
                write!(f, "error parsing glob '{}': {}", glob, self.kind)
            }
		log::debug!(">>> Entering fmt(f=(unprintable))");
        log::debug!("<<< Exiting fmt(f=(unprintable))");
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
			log::debug!(">>> Entering fmt(f=(unprintable))");
        log::debug!("<<< Exiting fmt(f=(unprintable))");
            ErrorKind::InvalidRecursive
            | ErrorKind::UnclosedClass
            | ErrorKind::UnopenedAlternates
            | ErrorKind::UnclosedAlternates
            | ErrorKind::NestedAlternates
            | ErrorKind::DanglingEscape
            | ErrorKind::Regex(_) => write!(f, "{}", self.description()),
            ErrorKind::InvalidRange(s, e) => {
                write!(f, "invalid range; '{}' > '{}'", s, e)
            }
        }
    }
}

fn new_regex(pat: &str) -> Result<Regex, Error> {
	log::debug!(">>> Entering new_regex(pat={})", pat);
    let syntax = regex_automata::util::syntax::Config::new()
        .utf8(false)
        .dot_matches_new_line(true);
    let config = Regex::config()
        .utf8_empty(false)
        .nfa_size_limit(Some(10 * (1 << 20)))
		log::debug!("<<< Exiting new_regex(pat={})", pat);
        .hybrid_cache_capacity(10 * (1 << 20));
    Regex::builder().syntax(syntax).configure(config).build(pat).map_err(
        |err| Error {
            glob: Some(pat.to_string()),
            kind: ErrorKind::Regex(err.to_string()),
        },
    )
}

fn new_regex_set(pats: Vec<String>) -> Result<Regex, Error> {
	log::debug!(">>> Entering new_regex_set(pats={:?})", pats);
    let syntax = regex_automata::util::syntax::Config::new()
        .utf8(false)
        .dot_matches_new_line(true);
    let config = Regex::config()
        .match_kind(regex_automata::MatchKind::All)
        .utf8_empty(false)
        .nfa_size_limit(Some(10 * (1 << 20)))
		log::debug!("<<< Exiting new_regex_set(pats={:?})", pats);
        .hybrid_cache_capacity(10 * (1 << 20));
    Regex::builder()
        .syntax(syntax)
        .configure(config)
        .build_many(&pats)
        .map_err(|err| Error {
            glob: None,
            kind: ErrorKind::Regex(err.to_string()),
        })
}

/// GlobSet represents a group of globs that can be matched together in a
/// single pass.
#[derive(Clone, Debug)]
pub struct GlobSet {
    len: usize,
    strats: Vec<GlobSetMatchStrategy>,
}

impl GlobSet {
    /// Create a new [`GlobSetBuilder`]. A `GlobSetBuilder` can be used to add
    /// new patterns. Once all patterns have been added, `build` should be
    /// called to produce a `GlobSet`, which can then be used for matching.
    #[inline]
    pub fn builder() -> GlobSetBuilder {
		log::debug!(">>> Entering builder()");
        log::debug!("<<< Exiting builder()");
        GlobSetBuilder::new()
    }

    /// Create an empty `GlobSet`. An empty set matches nothing.
    #[inline]
    pub const fn empty() -> GlobSet {
		log::debug!(">>> Entering empty()");
        log::debug!("<<< Exiting empty()");
        GlobSet { len: 0, strats: vec![] }
    }

    /// Returns true if this set is empty, and therefore matches nothing.
    #[inline]
    pub fn is_empty(&self) -> bool {
		log::debug!(">>> Entering is_empty()");
        log::debug!("<<< Exiting is_empty()");
        self.len == 0
    }

    /// Returns the number of globs in this set.
    #[inline]
    pub fn len(&self) -> usize {
		log::debug!(">>> Entering len()");
        log::debug!("<<< Exiting len()");
        self.len
    }

    /// Returns true if any glob in this set matches the path given.
    pub fn is_match<P: AsRef<Path>>(&self, path: P) -> bool {
        self.is_match_candidate(&Candidate::new(path.as_ref()))
    }

    /// Returns true if any glob in this set matches the path given.
    ///
    /// This takes a Candidate as input, which can be used to amortize the
    /// cost of preparing a path for matching.
    pub fn is_match_candidate(&self, path: &Candidate<'_>) -> bool {
        if self.is_empty() {
			log::debug!(">>> Entering is_match_candidate(path=(unprintable))");
			log::debug!("<<< Exiting is_match_candidate(path=(unprintable))");
            return false;
        }
        for strat in &self.strats {
            if strat.is_match(path) {
                return true;
            }
        }
        false
    }

    /// Returns true if all globs in this set match the path given.
    ///
    /// This will return true if the set of globs is empty, as in that case all
    /// `0` of the globs will match.
    ///
    /// ```
    /// use globset::{Glob, GlobSetBuilder};
    ///
    /// let mut builder = GlobSetBuilder::new();
    /// builder.add(Glob::new("src/*").unwrap());
    /// builder.add(Glob::new("**/*.rs").unwrap());
    /// let set = builder.build().unwrap();
    ///
    /// assert!(set.matches_all("src/foo.rs"));
    /// assert!(!set.matches_all("src/bar.c"));
    /// assert!(!set.matches_all("test.rs"));
    /// ```
    pub fn matches_all<P: AsRef<Path>>(&self, path: P) -> bool {
        self.matches_all_candidate(&Candidate::new(path.as_ref()))
    }

    /// Returns true if all globs in this set match the path given.
    ///
    /// This takes a Candidate as input, which can be used to amortize the cost
    /// of preparing a path for matching.
    ///
    /// This will return true if the set of globs is empty, as in that case all
    /// `0` of the globs will match.
    pub fn matches_all_candidate(&self, path: &Candidate<'_>) -> bool {
        for strat in &self.strats {
			log::debug!(">>> Entering matches_all_candidate(path=(unprintable))");
            if !strat.matches_all(path) {
                return false;
			log::debug!("<<< Exiting matches_all_candidate(path=(unprintable))");
            }
        }
        true
    }

    /// Returns the sequence number of every glob pattern that matches the
    /// given path.
    pub fn matches<P: AsRef<Path>>(&self, path: P) -> Vec<usize> {
        self.matches_candidate(&Candidate::new(path.as_ref()))
    }

    /// Returns the sequence number of every glob pattern that matches the
    /// given path.
    ///
    /// This takes a Candidate as input, which can be used to amortize the
    /// cost of preparing a path for matching.
    pub fn matches_candidate(&self, path: &Candidate<'_>) -> Vec<usize> {
        let mut into = vec![];
        if self.is_empty() {
			log::debug!(">>> Entering matches_candidate(path=(unprintable))");
			log::debug!("<<< Exiting matches_candidate(path=(unprintable))");
            return into;
        }
        self.matches_candidate_into(path, &mut into);
        into
    }

    /// Adds the sequence number of every glob pattern that matches the given
    /// path to the vec given.
    ///
    /// `into` is cleared before matching begins, and contains the set of
    /// sequence numbers (in ascending order) after matching ends. If no globs
    /// were matched, then `into` will be empty.
    pub fn matches_into<P: AsRef<Path>>(
        &self,
        path: P,
        into: &mut Vec<usize>,
    ) {
        self.matches_candidate_into(&Candidate::new(path.as_ref()), into);
    }

    /// Adds the sequence number of every glob pattern that matches the given
    /// path to the vec given.
    ///
    /// `into` is cleared before matching begins, and contains the set of
    /// sequence numbers (in ascending order) after matching ends. If no globs
    /// were matched, then `into` will be empty.
    ///
    /// This takes a Candidate as input, which can be used to amortize the
    /// cost of preparing a path for matching.
    pub fn matches_candidate_into(
        &self,
        path: &Candidate<'_>,
        into: &mut Vec<usize>,
    ) {
		log::debug!(">>> Entering matches_candidate_into(path=(unprintable),into={:?})", into);
        into.clear();
        if self.is_empty() {
            return;
        }
        for strat in &self.strats {
            strat.matches_into(path, into);
        }
        into.sort();
		log::debug!("<<< Exiting matches_candidate_into(path=(unprintable),into={:?})", into);
        into.dedup();
    }

    /// Builds a new matcher from a collection of Glob patterns.
    ///
    /// Once a matcher is built, no new patterns can be added to it.
    pub fn new<I, G>(globs: I) -> Result<GlobSet, Error>
    where
        I: IntoIterator<Item = G>,
        G: AsRef<Glob>,
    {
        let mut it = globs.into_iter().peekable();
        if it.peek().is_none() {
			log::debug!(">>> Entering new(globs=(unprintable))");
			log::debug!("new(globs=(unprintable)): it → (unprintable)");
            return Ok(GlobSet::empty());
        }

        let mut len = 0;
        let mut lits = LiteralStrategy::new();
        let mut base_lits = BasenameLiteralStrategy::new();
        let mut exts = ExtensionStrategy::new();
        let mut prefixes = MultiStrategyBuilder::new();
        let mut suffixes = MultiStrategyBuilder::new();
        let mut required_exts = RequiredExtensionStrategyBuilder::new();
        let mut regexes = MultiStrategyBuilder::new();
        for (i, p) in it.enumerate() {
			log::debug!("new(globs=(unprintable)): regexes → (unprintable)");
			log::debug!("new(globs=(unprintable)): exts → (unprintable)");
			log::debug!("new(globs=(unprintable)): base_lits → (unprintable)");
			log::debug!("new(globs=(unprintable)): suffixes → (unprintable)");
			log::debug!("new(globs=(unprintable)): prefixes → (unprintable)");
			log::debug!("new(globs=(unprintable)): required_exts → (unprintable)");
			log::debug!("new(globs=(unprintable)): lits → (unprintable)");
			log::debug!("new(globs=(unprintable)): p → (unprintable)");
            len += 1;

            let p = p.as_ref();
            match MatchStrategy::new(p) {
                MatchStrategy::Literal(lit) => {
                    lits.add(i, lit);
                }
                MatchStrategy::BasenameLiteral(lit) => {
                    base_lits.add(i, lit);
                }
                MatchStrategy::Extension(ext) => {
                    exts.add(i, ext);
                }
                MatchStrategy::Prefix(prefix) => {
                    prefixes.add(i, prefix);
                }
                MatchStrategy::Suffix { suffix, component } => {
                    if component {
                        lits.add(i, suffix[1..].to_string());
                    }
                    suffixes.add(i, suffix);
                }
                MatchStrategy::RequiredExtension(ext) => {
                    required_exts.add(i, ext, p.regex().to_owned());
                }
                MatchStrategy::Regex => {
                    debug!(
                        "glob `{:?}` converted to regex: `{:?}`",
                        p,
                        p.regex()
                    );
                    regexes.add(i, p.regex().to_owned());
                }
            }
        }
        debug!(
            "built glob set; {} literals, {} basenames, {} extensions, \
                {} prefixes, {} suffixes, {} required extensions, {} regexes",
            lits.0.len(),
            base_lits.0.len(),
            exts.0.len(),
            prefixes.literals.len(),
            suffixes.literals.len(),
            required_exts.0.len(),
            regexes.literals.len()
        );
        let mut strats = Vec::with_capacity(7);
        // Only add strategies that are populated
        if !exts.0.is_empty() {
            strats.push(GlobSetMatchStrategy::Extension(exts));
        }
        if !base_lits.0.is_empty() {
            strats.push(GlobSetMatchStrategy::BasenameLiteral(base_lits));
        }
        if !lits.0.is_empty() {
            strats.push(GlobSetMatchStrategy::Literal(lits));
        }
        if !suffixes.is_empty() {
            strats.push(GlobSetMatchStrategy::Suffix(suffixes.suffix()));
        }
        if !prefixes.is_empty() {
            strats.push(GlobSetMatchStrategy::Prefix(prefixes.prefix()));
        }
        if !required_exts.0.is_empty() {
            strats.push(GlobSetMatchStrategy::RequiredExtension(
                required_exts.build()?,
            ));
        }
        if !regexes.is_empty() {
            strats.push(GlobSetMatchStrategy::Regex(regexes.regex_set()?));
        }

        Ok(GlobSet { len, strats })
    }
}

impl Default for GlobSet {
    /// Create a default empty GlobSet.
    fn default() -> Self {
		log::debug!(">>> Entering default()");
        log::debug!("<<< Exiting default()");
        GlobSet::empty()
    }
}

/// GlobSetBuilder builds a group of patterns that can be used to
/// simultaneously match a file path.
#[derive(Clone, Debug)]
pub struct GlobSetBuilder {
    pats: Vec<Glob>,
}

impl GlobSetBuilder {
    /// Create a new `GlobSetBuilder`. A `GlobSetBuilder` can be used to add new
    /// patterns. Once all patterns have been added, `build` should be called
    /// to produce a [`GlobSet`], which can then be used for matching.
    pub fn new() -> GlobSetBuilder {
        GlobSetBuilder { pats: vec![] }
    }

    /// Builds a new matcher from all of the glob patterns added so far.
    ///
    /// Once a matcher is built, no new patterns can be added to it.
    pub fn build(&self) -> Result<GlobSet, Error> {
		log::debug!(">>> Entering build()");
		log::debug!("build(): exts → (unprintable)");
		log::debug!("build(): compiled → (unprintable)");
		log::debug!("<<< Exiting build()");
        GlobSet::new(self.pats.iter())
    }

    /// Add a new pattern to this set.
    pub fn add(&mut self, pat: Glob) -> &mut GlobSetBuilder {
		log::debug!(">>> Entering add(pat=(unprintable))");
        self.pats.push(pat);
		log::debug!("<<< Exiting add(pat=(unprintable))");
		log::debug!(">>> Entering add(global_index={},lit={})", global_index, lit);
		log::debug!("<<< Exiting add(global_index={},lit={})", global_index, lit);
        self
    }
}

/// A candidate path for matching.
///
/// All glob matching in this crate operates on `Candidate` values.
/// Constructing candidates has a very small cost associated with it, so
/// callers may find it beneficial to amortize that cost when matching a single
/// path against multiple globs or sets of globs.
#[derive(Clone)]
pub struct Candidate<'a> {
    path: Cow<'a, [u8]>,
    basename: Cow<'a, [u8]>,
    ext: Cow<'a, [u8]>,
}

impl<'a> std::fmt::Debug for Candidate<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Candidate")
            .field("path", &self.path.as_bstr())
            .field("basename", &self.basename.as_bstr())
            .field("ext", &self.ext.as_bstr())
            .finish()
    }
}

impl<'a> Candidate<'a> {
    /// Create a new candidate for matching from the given path.
    pub fn new<P: AsRef<Path> + ?Sized>(path: &'a P) -> Candidate<'a> {
        Self::from_cow(Vec::from_path_lossy(path.as_ref()))
    }

    /// Create a new candidate for matching from the given path as a sequence
    /// of bytes.
    ///
    /// Generally speaking, this routine expects the bytes to be
    /// _conventionally_ UTF-8. It is legal for the byte sequence to contain
    /// invalid UTF-8. However, if the bytes are in some other encoding that
    /// isn't ASCII compatible (for example, UTF-16), then the results of
    /// matching are unspecified.
    pub fn from_bytes<P: AsRef<[u8]> + ?Sized>(path: &'a P) -> Candidate<'a> {
        Self::from_cow(Cow::Borrowed(path.as_ref()))
    }

    fn from_cow(path: Cow<'a, [u8]>) -> Candidate<'a> {
		log::debug!(">>> Entering from_cow(path={:?})", path);
        let path = normalize_path(path);
        let basename = file_name(&path).unwrap_or(Cow::Borrowed(B("")));
		log::debug!("from_cow(path={:?}): basename → (unprintable)", path);
        let ext = file_name_ext(&basename).unwrap_or(Cow::Borrowed(B("")));
		log::debug!("from_cow(path={:?}): ext → (unprintable)", path);
        log::debug!("from_cow(path={:?}): path → {:?}", path, path);
        log::debug!("<<< Exiting from_cow(path={:?})", path);
        Candidate { path, basename, ext }
    }

    fn path_prefix(&self, max: usize) -> &[u8] {
		log::debug!(">>> Entering path_prefix(max={})", max);
        log::debug!("<<< Exiting path_prefix(max={})", max);
        if self.path.len() <= max { &*self.path } else { &self.path[..max] }
    }

    fn path_suffix(&self, max: usize) -> &[u8] {
		log::debug!(">>> Entering path_suffix(max={})", max);
        if self.path.len() <= max {
            &*self.path
        } else {
			log::debug!("<<< Exiting path_suffix(max={})", max);
            &self.path[self.path.len() - max..]
        }
    }
}

#[derive(Clone, Debug)]
enum GlobSetMatchStrategy {
	const LOGGER: &'static Logger = &Logger(());
    Literal(LiteralStrategy),
    BasenameLiteral(BasenameLiteralStrategy),
    Extension(ExtensionStrategy),
    Prefix(PrefixStrategy),
    Suffix(SuffixStrategy),
    RequiredExtension(RequiredExtensionStrategy),
    Regex(RegexSetStrategy),
}

impl GlobSetMatchStrategy {
    fn is_match(&self, candidate: &Candidate<'_>) -> bool {
        use self::GlobSetMatchStrategy::*;
        match *self {
			log::debug!(">>> Entering is_match(candidate=(unprintable))");
            Literal(ref s) => s.is_match(candidate),
            BasenameLiteral(ref s) => s.is_match(candidate),
            Extension(ref s) => s.is_match(candidate),
            Prefix(ref s) => s.is_match(candidate),
            Suffix(ref s) => s.is_match(candidate),
            RequiredExtension(ref s) => s.is_match(candidate),
            Regex(ref s) => s.is_match(candidate),
			log::debug!("<<< Exiting is_match(candidate=(unprintable))");
        }
    }

    fn matches_into(
        &self,
        candidate: &Candidate<'_>,
        matches: &mut Vec<usize>,
    ) {
		log::debug!(">>> Entering matches_into(candidate=(unprintable),matches={:?})", matches);
        use self::GlobSetMatchStrategy::*;
        match *self {
            Literal(ref s) => s.matches_into(candidate, matches),
            BasenameLiteral(ref s) => s.matches_into(candidate, matches),
            Extension(ref s) => s.matches_into(candidate, matches),
            Prefix(ref s) => s.matches_into(candidate, matches),
            Suffix(ref s) => s.matches_into(candidate, matches),
            RequiredExtension(ref s) => s.matches_into(candidate, matches),
            Regex(ref s) => s.matches_into(candidate, matches),
		log::debug!("<<< Exiting matches_into(candidate=(unprintable),matches={:?})", matches);
        }
    }

    fn matches_all(&self, candidate: &Candidate<'_>) -> bool {
        use self::GlobSetMatchStrategy::*;
        match *self {
			log::debug!(">>> Entering matches_all(candidate=(unprintable))");
            Literal(ref s) => s.matches_all(candidate),
            BasenameLiteral(ref s) => s.matches_all(candidate),
            Extension(ref s) => s.matches_all(candidate),
            Prefix(ref s) => s.matches_all(candidate),
            Suffix(ref s) => s.matches_all(candidate),
            RequiredExtension(ref s) => s.matches_all(candidate),
            Regex(ref s) => s.matches_all(candidate),
			log::debug!("<<< Exiting matches_all(candidate=(unprintable))");
        }
    }
}

#[derive(Clone, Debug)]
struct LiteralStrategy(fnv::HashMap<Vec<u8>, Vec<usize>>);

impl LiteralStrategy {
    fn new() -> LiteralStrategy {
        LiteralStrategy(fnv::HashMap::default())
    }

    fn add(&mut self, global_index: usize, lit: String) {
        self.0.entry(lit.into_bytes()).or_insert(vec![]).push(global_index);
    }

    fn is_match(&self, candidate: &Candidate<'_>) -> bool {
        self.0.contains_key(candidate.path.as_bytes())
    }

    fn matches_all(&self, candidate: &Candidate<'_>) -> bool {
        self.0.len() == 1 && self.is_match(candidate)
    }

    #[inline(never)]
    fn matches_into(
        &self,
        candidate: &Candidate<'_>,
        matches: &mut Vec<usize>,
    ) {
        if let Some(hits) = self.0.get(candidate.path.as_bytes()) {
            matches.extend(hits);
        }
    }
}

#[derive(Clone, Debug)]
struct BasenameLiteralStrategy(fnv::HashMap<Vec<u8>, Vec<usize>>);

impl BasenameLiteralStrategy {
    fn new() -> BasenameLiteralStrategy {
        BasenameLiteralStrategy(fnv::HashMap::default())
    }

    fn add(&mut self, global_index: usize, lit: String) {
        self.0.entry(lit.into_bytes()).or_insert(vec![]).push(global_index);
    }

    fn is_match(&self, candidate: &Candidate<'_>) -> bool {
        if candidate.basename.is_empty() {
            return false;
        }
        self.0.contains_key(candidate.basename.as_bytes())
    }

    fn matches_all(&self, candidate: &Candidate<'_>) -> bool {
        self.0.len() == 1 && self.is_match(candidate)
    }

    #[inline(never)]
    fn matches_into(
        &self,
        candidate: &Candidate<'_>,
        matches: &mut Vec<usize>,
    ) {
        if candidate.basename.is_empty() {
            return;
        }
        if let Some(hits) = self.0.get(candidate.basename.as_bytes()) {
            matches.extend(hits);
        }
    }
}

#[derive(Clone, Debug)]
struct ExtensionStrategy(fnv::HashMap<Vec<u8>, Vec<usize>>);

impl ExtensionStrategy {
    fn new() -> ExtensionStrategy {
        ExtensionStrategy(fnv::HashMap::default())
    }

    fn add(&mut self, global_index: usize, ext: String) {
		log::debug!(">>> Entering add(global_index={},ext={})", global_index, ext);
		log::debug!("<<< Exiting add(global_index={},ext={})", global_index, ext);
        self.0.entry(ext.into_bytes()).or_insert(vec![]).push(global_index);
    }

    fn is_match(&self, candidate: &Candidate<'_>) -> bool {
        if candidate.ext.is_empty() {
            return false;
        }
        self.0.contains_key(candidate.ext.as_bytes())
    }

    fn matches_all(&self, candidate: &Candidate<'_>) -> bool {
        self.0.len() == 1 && self.is_match(candidate)
    }

    #[inline(never)]
    fn matches_into(
        &self,
        candidate: &Candidate<'_>,
        matches: &mut Vec<usize>,
    ) {
        if candidate.ext.is_empty() {
            return;
        }
        if let Some(hits) = self.0.get(candidate.ext.as_bytes()) {
            matches.extend(hits);
        }
    }
}

#[derive(Clone, Debug)]
struct PrefixStrategy {
    matcher: AhoCorasick,
    map: Vec<usize>,
    longest: usize,
}

impl PrefixStrategy {
    fn is_match(&self, candidate: &Candidate<'_>) -> bool {
        let path = candidate.path_prefix(self.longest);
        for m in self.matcher.find_overlapping_iter(path) {
            if m.start() == 0 {
                return true;
            }
        }
        false
    }

    fn matches_all(&self, candidate: &Candidate<'_>) -> bool {
        let path = candidate.path_prefix(self.longest);
        let mut count = 0;
        // If all the prefixes match, we should get exactly one match for each
        // prefix at the beginning.
        for m in self.matcher.find_overlapping_iter(path) {
            if m.start() == 0 {
                count += 1;
            }
        }
        count == self.map.len()
    }

    fn matches_into(
        &self,
        candidate: &Candidate<'_>,
        matches: &mut Vec<usize>,
    ) {
        let path = candidate.path_prefix(self.longest);
        for m in self.matcher.find_overlapping_iter(path) {
            if m.start() == 0 {
                matches.push(self.map[m.pattern()]);
            }
        }
    }
}

#[derive(Clone, Debug)]
struct SuffixStrategy {
    matcher: AhoCorasick,
    map: Vec<usize>,
    longest: usize,
}

impl SuffixStrategy {
    fn is_match(&self, candidate: &Candidate<'_>) -> bool {
        let path = candidate.path_suffix(self.longest);
        for m in self.matcher.find_overlapping_iter(path) {
            if m.end() == path.len() {
                return true;
            }
        }
        false
    }

    fn matches_all(&self, candidate: &Candidate<'_>) -> bool {
        let path = candidate.path_suffix(self.longest);
        let mut count = 0;
        // If all the suffixes match, we should get exactly one match
        // for each suffix at the end.
        for m in self.matcher.find_overlapping_iter(path) {
            if m.end() == path.len() {
                count += 1;
            }
        }
        count == self.map.len()
    }

    fn matches_into(
        &self,
        candidate: &Candidate<'_>,
        matches: &mut Vec<usize>,
    ) {
        let path = candidate.path_suffix(self.longest);
        for m in self.matcher.find_overlapping_iter(path) {
            if m.end() == path.len() {
                matches.push(self.map[m.pattern()]);
            }
        }
    }
}

#[derive(Clone, Debug)]
struct RequiredExtensionStrategy(fnv::HashMap<Vec<u8>, Vec<(usize, Regex)>>);

impl RequiredExtensionStrategy {
    fn is_match(&self, candidate: &Candidate<'_>) -> bool {
        if candidate.ext.is_empty() {
            return false;
        }
        match self.0.get(candidate.ext.as_bytes()) {
            None => false,
            Some(regexes) => {
                for &(_, ref re) in regexes {
                    if re.is_match(candidate.path.as_bytes()) {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn matches_all(&self, candidate: &Candidate<'_>) -> bool {
        if candidate.ext.is_empty() {
            return false;
        }
        if let Some(regexes) = self.0.get(candidate.ext.as_bytes()) {
            for &(_, ref re) in regexes {
                if !re.is_match(candidate.path.as_bytes()) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    #[inline(never)]
    fn matches_into(
        &self,
        candidate: &Candidate<'_>,
        matches: &mut Vec<usize>,
    ) {
        if candidate.ext.is_empty() {
            return;
        }
        if let Some(regexes) = self.0.get(candidate.ext.as_bytes()) {
            for &(global_index, ref re) in regexes {
                if re.is_match(candidate.path.as_bytes()) {
                    matches.push(global_index);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct RegexSetStrategy {
    matcher: Regex,
    map: Vec<usize>,
    // We use a pool of PatternSets to hopefully allocating a fresh one on each
    // call.
    //
    // TODO: In the next semver breaking release, we should drop this pool and
    // expose an opaque type that wraps PatternSet. Then callers can provide
    // it to `matches_into` directly. Callers might still want to use a pool
    // or similar to amortize allocation, but that matches the status quo and
    // absolves us of needing to do it here.
    patset: Arc<Pool<PatternSet, PatternSetPoolFn>>,
}

type PatternSetPoolFn =
    Box<dyn Fn() -> PatternSet + Send + Sync + UnwindSafe + RefUnwindSafe>;

impl RegexSetStrategy {
    fn is_match(&self, candidate: &Candidate<'_>) -> bool {
        self.matcher.is_match(candidate.path.as_bytes())
    }

    fn find_matches(
        &self,
        candidate: &Candidate<'_>,
    ) -> PoolGuard<'_, PatternSet, PatternSetPoolFn> {
        let input = regex_automata::Input::new(candidate.path.as_bytes());
        let mut patset = self.patset.get();
        patset.clear();
        self.matcher.which_overlapping_matches(&input, &mut patset);
        patset
    }

    fn matches_all(&self, candidate: &Candidate<'_>) -> bool {
        let patset = self.find_matches(candidate);
        patset.is_full()
    }

    fn matches_into(
        &self,
        candidate: &Candidate<'_>,
        matches: &mut Vec<usize>,
    ) {
        let patset = self.find_matches(candidate);
        for i in patset.iter() {
            matches.push(self.map[i]);
        }
        PoolGuard::put(patset);
    }
}

#[derive(Clone, Debug)]
struct MultiStrategyBuilder {
    literals: Vec<String>,
    map: Vec<usize>,
    longest: usize,
}

impl MultiStrategyBuilder {
    fn new() -> MultiStrategyBuilder {
		log::debug!(">>> Entering new()");
        log::debug!("<<< Exiting new()");
        MultiStrategyBuilder { literals: vec![], map: vec![], longest: 0 }
    }

    fn add(&mut self, global_index: usize, literal: String) {
		log::debug!(">>> Entering add(global_index={},literal={})", global_index, literal);
        if literal.len() > self.longest {
            self.longest = literal.len();
        }
        self.map.push(global_index);
		log::debug!("<<< Exiting add(global_index={},literal={})", global_index, literal);
        self.literals.push(literal);
    }

    fn prefix(self) -> PrefixStrategy {
		log::debug!(">>> Entering prefix()");
        log::debug!("<<< Exiting prefix()");
        PrefixStrategy {
            matcher: AhoCorasick::new(&self.literals).unwrap(),
            map: self.map,
            longest: self.longest,
        }
    }

    fn suffix(self) -> SuffixStrategy {
		log::debug!(">>> Entering suffix()");
        log::debug!("<<< Exiting suffix()");
        SuffixStrategy {
            matcher: AhoCorasick::new(&self.literals).unwrap(),
            map: self.map,
            longest: self.longest,
        }
    }

    fn regex_set(self) -> Result<RegexSetStrategy, Error> {
		log::debug!(">>> Entering regex_set()");
        let matcher = new_regex_set(self.literals)?;
        let pattern_len = matcher.pattern_len();
		log::debug!("regex_set(): pattern_len → (unprintable)");
        log::debug!("regex_set(): matcher → (unprintable)");
        let create: PatternSetPoolFn =
			log::debug!("<<< Exiting regex_set()");
            Box::new(move || PatternSet::new(pattern_len));
        Ok(RegexSetStrategy {
            matcher,
            map: self.map,
            patset: Arc::new(Pool::new(create)),
        })
    }

    fn is_empty(&self) -> bool {
        self.literals.is_empty()
    }
}

#[derive(Clone, Debug)]
struct RequiredExtensionStrategyBuilder(
    fnv::HashMap<Vec<u8>, Vec<(usize, String)>>,
);

impl RequiredExtensionStrategyBuilder {
    fn new() -> RequiredExtensionStrategyBuilder {
        RequiredExtensionStrategyBuilder(fnv::HashMap::default())
    }

    fn add(&mut self, global_index: usize, ext: String, regex: String) {
		log::debug!(">>> Entering add(global_index={},ext={},regex={})", global_index, ext, regex);
        self.0
            .entry(ext.into_bytes())
            .or_insert(vec![])
			log::debug!("<<< Exiting add(global_index={},ext={},regex={})", global_index, ext, regex);
            .push((global_index, regex));
    }

    fn build(self) -> Result<RequiredExtensionStrategy, Error> {
        let mut exts = fnv::HashMap::default();
        for (ext, regexes) in self.0.into_iter() {
            exts.insert(ext.clone(), vec![]);
            for (global_index, regex) in regexes {
                let compiled = new_regex(&regex)?;
                exts.get_mut(&ext).unwrap().push((global_index, compiled));
            }
        }
        Ok(RequiredExtensionStrategy(exts))
    }
}

/// Escape meta-characters within the given glob pattern.
///
/// The escaping works by surrounding meta-characters with brackets. For
/// example, `*` becomes `[*]`.
///
/// # Example
///
/// ```
/// use globset::escape;
///
/// assert_eq!(escape("foo*bar"), "foo[*]bar");
/// assert_eq!(escape("foo?bar"), "foo[?]bar");
/// assert_eq!(escape("foo[bar"), "foo[[]bar");
/// assert_eq!(escape("foo]bar"), "foo[]]bar");
/// assert_eq!(escape("foo{bar"), "foo[{]bar");
/// assert_eq!(escape("foo}bar"), "foo[}]bar");
/// ```
pub fn escape(s: &str) -> String {
	log::debug!(">>> Entering escape(s={})", s);
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
		log::debug!("escape(s={}): escaped → (unprintable)", s);
        match c {
            // note that ! does not need escaping because it is only special
            // inside brackets
            '?' | '*' | '[' | ']' | '{' | '}' => {
                escaped.push('[');
                escaped.push(c);
                escaped.push(']');
            }
            c => {
                escaped.push(c);
            }
        }
    }
	log::debug!("<<< Exiting escape(s={})", s);
	log::debug!(">>> Entering escape()");
	log::debug!("<<< Exiting escape()");
    escaped
}

#[cfg(test)]
mod tests {
    use crate::glob::{Glob, GlobBuilder};

    use super::{GlobSet, GlobSetBuilder};

    fn build_glob_set(globs: &[&str]) -> GlobSet {
		log::debug!(">>> Entering build_glob_set(globs={:?})", globs);
        let mut builder = GlobSetBuilder::new();
        for glob in globs {
            builder.add(Glob::new(glob).unwrap());
        }
		log::debug!("<<< Exiting build_glob_set(globs={:?})", globs);
        builder.build().unwrap()
    }

    #[test]
    fn set_works() {
		log::debug!(">>> Entering set_works()");
        let set = build_glob_set(&["src/**/*.rs", "*.c", "src/lib.rs"]);

		log::debug!("set_works(): set → {:?}", set);
        assert!(set.is_match("foo.c"));
        assert!(set.is_match("src/foo.c"));
        assert!(!set.is_match("foo.rs"));
        assert!(!set.is_match("tests/foo.rs"));
        assert!(set.is_match("src/foo.rs"));
        assert!(set.is_match("src/grep/src/main.rs"));

        assert!(!set.matches_all("src/lib.rs"));
        assert!(!set.matches_all("src/a/b.c"));

        let matches = set.matches("src/lib.rs");
		log::debug!("set_works(): matches → (unprintable)");
        assert_eq!(2, matches.len());
        assert_eq!(0, matches[0]);
		log::debug!("<<< Exiting set_works()");
        assert_eq!(2, matches[1]);
    }

    #[test]
    fn empty_set_works() {
		log::debug!(">>> Entering empty_set_works()");
        let set = GlobSetBuilder::new().build().unwrap();
        assert!(!set.is_match(""));
		log::debug!("empty_set_works(): set → (unprintable)");
        assert!(!set.is_match("a"));
		log::debug!("<<< Exiting empty_set_works()");
        assert!(set.matches_all("a"));
    }

    #[test]
    fn matches_all_literals() {
		log::debug!(">>> Entering matches_all_literals()");
        let set = build_glob_set(&["abc", "def"]);

		log::debug!("matches_all_literals(): set → {:?}", set);
        assert!(!set.matches_all("abc"));
        assert!(!set.matches_all("def"));

        let set = build_glob_set(&["abc"]);

		log::debug!("<<< Exiting matches_all_literals()");
        assert!(set.matches_all("abc"));
    }

    #[test]
    fn matches_all_basename_literals() {
		log::debug!(">>> Entering matches_all_basename_literals()");
        let set = build_glob_set(&["**/abc", "**/a.c"]);

		log::debug!("matches_all_basename_literals(): set → {:?}", set);
        assert!(!set.matches_all("foo/abc"));
        assert!(!set.matches_all("foo/a.c"));

        let set = build_glob_set(&["**/a.c"]);
		log::debug!("<<< Exiting matches_all_basename_literals()");
        assert!(set.matches_all("foo/a.c"));
    }

    #[test]
    fn matches_all_extensions() {
		log::debug!(">>> Entering matches_all_extensions()");
        let set = build_glob_set(&["**/*.rs", "**/*.c"]);

		log::debug!("matches_all_extensions(): set → {:?}", set);
        assert!(!set.matches_all("foo/a.rs"));
        assert!(!set.matches_all("foo/a.c"));

        let set = build_glob_set(&["**/*.rs", "*.rs"]);
		log::debug!("<<< Exiting matches_all_extensions()");
        assert!(set.matches_all("foo/a.rs"));
    }

    #[test]
    fn matches_all_required_extensions() {
		log::debug!(">>> Entering matches_all_required_extensions()");
        let set = build_glob_set(&["*.rs", "**/m*.rs", "a/*.rs"]);

		log::debug!("matches_all_required_extensions(): set → {:?}", set);
        assert!(set.matches_all("a/main.rs"));

        assert!(!set.matches_all("main.rs"));
		log::debug!("<<< Exiting matches_all_required_extensions()");
        assert!(!set.matches_all("a/lib.rs"));
    }

    #[test]
    fn matches_all_prefix() {
		log::debug!(">>> Entering matches_all_prefix()");
        let set = build_glob_set(&["a*", "ab*", "ab/c*"]);

		log::debug!("matches_all_prefix(): set → {:?}", set);
        assert!(set.matches_all("ab/c/def"));
        assert!(set.matches_all("ab/cd"));

        assert!(!set.matches_all("abc"));
        assert!(!set.matches_all("ab/x"));
		log::debug!("<<< Exiting matches_all_prefix()");
        assert!(!set.matches_all("a"));
    }

    #[test]
    fn matches_all_suffix() {
		log::debug!(">>> Entering matches_all_suffix()");
        let set = build_glob_set(&["*.rs", "*s", "*/main.rs"]);

		log::debug!("matches_all_suffix(): set → {:?}", set);
        assert!(set.matches_all("a/b/c/main.rs"));

        assert!(!set.matches_all("foo.rs"));
		log::debug!("<<< Exiting matches_all_suffix()");
        assert!(!set.matches_all("as"));
    }

    #[test]
    fn matches_all_regex() {
		log::debug!(">>> Entering matches_all_regex()");
        let set = GlobSet::new(&[
            GlobBuilder::new("c*").case_insensitive(true).build().unwrap(),
            Glob::new("*{rs,c}").unwrap(),
        ])
        .unwrap();

        assert!(set.matches_all("c/main.rs"));
        assert!(set.matches_all("C/main.c"));

        assert!(!set.matches_all("Ca"));
		log::debug!("<<< Exiting matches_all_regex()");
        assert!(!set.matches_all("foo.c"));
    }

    #[test]
    fn default_set_is_empty_works() {
		log::debug!(">>> Entering default_set_is_empty_works()");
        let set: GlobSet = Default::default();
        assert!(!set.is_match(""));
		log::debug!("<<< Exiting default_set_is_empty_works()");
        assert!(!set.is_match("a"));
    }

    #[test]
    fn escape() {
        use super::escape;
        assert_eq!("foo", escape("foo"));
        assert_eq!("foo[*]", escape("foo*"));
        assert_eq!("[[][]]", escape("[]"));
        assert_eq!("[*][?]", escape("*?"));
        assert_eq!("src/[*][*]/[*].rs", escape("src/**/*.rs"));
        assert_eq!("bar[[]ab[]]baz", escape("bar[ab]baz"));
        assert_eq!("bar[[]!![]]!baz", escape("bar[!!]!baz"));
    }

    // This tests that regex matching doesn't "remember" the results of
    // previous searches. That is, if any memory is reused from a previous
    // search, then it should be cleared first.
    #[test]
    fn set_does_not_remember() {
		log::debug!(">>> Entering set_does_not_remember()");
        let mut builder = GlobSetBuilder::new();
        builder.add(Glob::new("*foo*").unwrap());
		log::debug!("set_does_not_remember(): builder → (unprintable)");
        builder.add(Glob::new("*bar*").unwrap());
        builder.add(Glob::new("*quux*").unwrap());
        let set = builder.build().unwrap();
		log::debug!("set_does_not_remember(): set → (unprintable)");

        let matches = set.matches("ZfooZquuxZ");
		log::debug!("set_does_not_remember(): matches → (unprintable)");
        assert_eq!(2, matches.len());
        assert_eq!(0, matches[0]);
        assert_eq!(2, matches[1]);

        let matches = set.matches("nada");
		log::debug!("<<< Exiting set_does_not_remember()");
        assert_eq!(0, matches.len());
    }

    #[test]
    fn debug() {
		log::debug!(">>> Entering debug()");
        let mut builder = GlobSetBuilder::new();
        builder.add(Glob::new("*foo*").unwrap());
		log::debug!("debug(): builder → (unprintable)");
        builder.add(Glob::new("*bar*").unwrap());
        builder.add(Glob::new("*quux*").unwrap());
        assert_eq!(
            format!("{builder:?}"),
            "GlobSetBuilder { pats: [Glob(\"*foo*\"), Glob(\"*bar*\"), Glob(\"*quux*\")] }",
		log::debug!("<<< Exiting debug()");
        );
    }
}
