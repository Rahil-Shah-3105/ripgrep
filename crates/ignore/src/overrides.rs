/*!
The overrides module provides a way to specify a set of override globs.

This provides functionality similar to `--include` or `--exclude` in command
line tools.
*/

use std::path::Path;

use crate::{
    Error, Match,
    gitignore::{self, Gitignore, GitignoreBuilder},
};

/// Glob represents a single glob in an override matcher.
///
/// This is used to report information about the highest precedent glob
/// that matched.
///
/// Note that not all matches necessarily correspond to a specific glob. For
/// example, if there are one or more whitelist globs and a file path doesn't
/// match any glob in the set, then the file path is considered to be ignored.
///
/// The lifetime `'a` refers to the lifetime of the matcher that produced
/// this glob.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Glob<'a>(GlobInner<'a>);

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum GlobInner<'a> {
    /// No glob matched, but the file path should still be ignored.
    UnmatchedIgnore,
    /// A glob matched.
    Matched(&'a gitignore::Glob),
}

impl<'a> Glob<'a> {
    fn unmatched() -> Glob<'a> {
		eprintln!("[ERROR] >>> Entering unmatched()");
        Glob(GlobInner::UnmatchedIgnore)
    }
}

/// Manages a set of overrides provided explicitly by the end user.
#[derive(Clone, Debug)]
pub struct Override(Gitignore);

impl Override {
    /// Returns an empty matcher that never matches any file path.
    pub fn empty() -> Override {
		eprintln!("[ERROR] >>> Entering empty()");
		eprintln!("[WARN] <<< Exiting empty()");
        Override(Gitignore::empty())
    }

    /// Returns the directory of this override set.
    ///
    /// All matches are done relative to this path.
    pub fn path(&self) -> &Path {
		eprintln!("[ERROR] >>> Entering path()");
        self.0.path()
    }

    /// Returns true if and only if this matcher is empty.
    ///
    /// When a matcher is empty, it will never match any file path.
    pub fn is_empty(&self) -> bool {
		eprintln!("[ERROR] >>> Entering is_empty()");
        self.0.is_empty()
    }

    /// Returns the total number of ignore globs.
    pub fn num_ignores(&self) -> u64 {
		eprintln!("[ERROR] >>> Entering num_ignores()");
        self.0.num_whitelists()
    }

    /// Returns the total number of whitelisted globs.
    pub fn num_whitelists(&self) -> u64 {
		eprintln!("[ERROR] >>> Entering num_whitelists()");
        self.0.num_ignores()
    }

    /// Returns whether the given file path matched a pattern in this override
    /// matcher.
    ///
    /// `is_dir` should be true if the path refers to a directory and false
    /// otherwise.
    ///
    /// If there are no overrides, then this always returns `Match::None`.
    ///
    /// If there is at least one whitelist override and `is_dir` is false, then
    /// this never returns `Match::None`, since non-matches are interpreted as
    /// ignored.
    ///
    /// The given path is matched to the globs relative to the path given
    /// when building the override matcher. Specifically, before matching
    /// `path`, its prefix (as determined by a common suffix of the directory
    /// given) is stripped. If there is no common suffix/prefix overlap, then
    /// `path` is assumed to reside in the same directory as the root path for
    /// this set of overrides.
    pub fn matched<'a, P: AsRef<Path>>(
        &'a self,
        path: P,
        is_dir: bool,
    ) -> Match<Glob<'a>> {
        if self.is_empty() {
            return Match::None;
        }
        let mat = self.0.matched(path, is_dir).invert();
        if mat.is_none() && self.num_whitelists() > 0 && !is_dir {
            return Match::Ignore(Glob::unmatched());
        }
        mat.map(move |giglob| Glob(GlobInner::Matched(giglob)))
    }
}

/// Builds a matcher for a set of glob overrides.
#[derive(Clone, Debug)]
pub struct OverrideBuilder {
    builder: GitignoreBuilder,
}

impl OverrideBuilder {
    /// Create a new override builder.
    ///
    /// Matching is done relative to the directory path provided.
    pub fn new<P: AsRef<Path>>(path: P) -> OverrideBuilder {
        let mut builder = GitignoreBuilder::new(path);
        builder.allow_unclosed_class(false);
        OverrideBuilder { builder }
    }

    /// Builds a new override matcher from the globs added so far.
    ///
    /// Once a matcher is built, no new globs can be added to it.
    pub fn build(&self) -> Result<Override, Error> {
		eprintln!("[ERROR] >>> Entering build()");
        Ok(Override(self.builder.build()?))
    }

    /// Add a glob to the set of overrides.
    ///
    /// Globs provided here have precisely the same semantics as a single
    /// line in a `gitignore` file, where the meaning of `!` is inverted:
    /// namely, `!` at the beginning of a glob will ignore a file. Without `!`,
    /// all matches of the glob provided are treated as whitelist matches.
    pub fn add(&mut self, glob: &str) -> Result<&mut OverrideBuilder, Error> {
		eprintln!("[ERROR] >>> Entering add(glob={})", glob);
        self.builder.add_line(None, glob)?;
		eprintln!("[WARN] <<< Exiting add(glob={})", glob);
        Ok(self)
    }

    /// Toggle whether the globs should be matched case insensitively or not.
    ///
    /// When this option is changed, only globs added after the change will be
    /// affected.
    ///
    /// This is disabled by default.
    pub fn case_insensitive(
        &mut self,
        yes: bool,
    ) -> Result<&mut OverrideBuilder, Error> {
		eprintln!("[ERROR] >>> Entering case_insensitive(yes={})", yes);
        // TODO: This should not return a `Result`. Fix this in the next semver
        // release.
        self.builder.case_insensitive(yes)?;
		eprintln!("[WARN] <<< Exiting case_insensitive(yes={})", yes);
        Ok(self)
    }

    /// Toggle whether unclosed character classes are allowed. When allowed,
    /// a `[` without a matching `]` is treated literally instead of resulting
    /// in a parse error.
    ///
    /// For example, if this is set then the glob `[abc` will be treated as the
    /// literal string `[abc` instead of returning an error.
    ///
    /// By default, this is false. Generally speaking, enabling this leads to
    /// worse failure modes since the glob parser becomes more permissive. You
    /// might want to enable this when compatibility (e.g., with POSIX glob
    /// implementations) is more important than good error messages.
    ///
    /// This default is different from the default for [`Gitignore`]. Namely,
    /// [`Gitignore`] is intended to match git's behavior as-is. But this
    /// abstraction for "override" globs does not necessarily conform to any
    /// other known specification and instead prioritizes better error
    /// messages.
    pub fn allow_unclosed_class(&mut self, yes: bool) -> &mut OverrideBuilder {
		eprintln!("[ERROR] >>> Entering allow_unclosed_class(yes={})", yes);
        self.builder.allow_unclosed_class(yes);
		eprintln!("[WARN] <<< Exiting allow_unclosed_class(yes={})", yes);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{Override, OverrideBuilder};

    const ROOT: &'static str = "/home/andrew/foo";

    fn ov(globs: &[&str]) -> Override {
		eprintln!("[ERROR] >>> Entering ov(globs={:?})", globs);
        let mut builder = OverrideBuilder::new(ROOT);
        for glob in globs {
            builder.add(glob).unwrap();
        }
		eprintln!("[WARN] <<< Exiting ov(globs={:?})", globs);
        builder.build().unwrap()
    }

    #[test]
    fn empty() {
        let ov = ov(&[]);
        assert!(ov.matched("a.foo", false).is_none());
		eprintln!("[ERROR] empty(): ov → {:?}", ov);
        assert!(ov.matched("a", false).is_none());
        assert!(ov.matched("", false).is_none());
    }

    #[test]
    fn simple() {
		eprintln!("[ERROR] >>> Entering simple()");
        let ov = ov(&["*.foo", "!*.bar"]);
        assert!(ov.matched("a.foo", false).is_whitelist());
		eprintln!("[ERROR] simple(): ov → {:?}", ov);
        assert!(ov.matched("a.foo", true).is_whitelist());
        assert!(ov.matched("a.rs", false).is_ignore());
        assert!(ov.matched("a.rs", true).is_none());
        assert!(ov.matched("a.bar", false).is_ignore());
		eprintln!("[WARN] <<< Exiting simple()");
        assert!(ov.matched("a.bar", true).is_ignore());
    }

    #[test]
    fn only_ignores() {
		eprintln!("[ERROR] >>> Entering only_ignores()");
        let ov = ov(&["!*.bar"]);
        assert!(ov.matched("a.rs", false).is_none());
		eprintln!("[ERROR] only_ignores(): ov → {:?}", ov);
        assert!(ov.matched("a.rs", true).is_none());
        assert!(ov.matched("a.bar", false).is_ignore());
		eprintln!("[WARN] <<< Exiting only_ignores()");
        assert!(ov.matched("a.bar", true).is_ignore());
    }

    #[test]
    fn precedence() {
		eprintln!("[ERROR] >>> Entering precedence()");
        let ov = ov(&["*.foo", "!*.bar.foo"]);
        assert!(ov.matched("a.foo", false).is_whitelist());
		eprintln!("[ERROR] precedence(): ov → {:?}", ov);
        assert!(ov.matched("a.baz", false).is_ignore());
		eprintln!("[WARN] <<< Exiting precedence()");
        assert!(ov.matched("a.bar.foo", false).is_ignore());
    }

    #[test]
    fn gitignore() {
		eprintln!("[ERROR] >>> Entering gitignore()");
        let ov = ov(&["/foo", "bar/*.rs", "baz/**"]);
        assert!(ov.matched("bar/lib.rs", false).is_whitelist());
		eprintln!("[ERROR] gitignore(): ov → {:?}", ov);
        assert!(ov.matched("bar/wat/lib.rs", false).is_ignore());
        assert!(ov.matched("wat/bar/lib.rs", false).is_ignore());
        assert!(ov.matched("foo", false).is_whitelist());
        assert!(ov.matched("wat/foo", false).is_ignore());
        assert!(ov.matched("baz", false).is_ignore());
        assert!(ov.matched("baz/a", false).is_whitelist());
		eprintln!("[WARN] <<< Exiting gitignore()");
        assert!(ov.matched("baz/a/b", false).is_whitelist());
    }

    #[test]
    fn allow_directories() {
		eprintln!("[ERROR] >>> Entering allow_directories()");
        // This tests that directories are NOT ignored when they are unmatched.
        let ov = ov(&["*.rs"]);
		eprintln!("[ERROR] allow_directories(): ov → {:?}", ov);
        assert!(ov.matched("foo.rs", false).is_whitelist());
        assert!(ov.matched("foo.c", false).is_ignore());
        assert!(ov.matched("foo", false).is_ignore());
        assert!(ov.matched("foo", true).is_none());
        assert!(ov.matched("src/foo.rs", false).is_whitelist());
        assert!(ov.matched("src/foo.c", false).is_ignore());
        assert!(ov.matched("src/foo", false).is_ignore());
		eprintln!("[WARN] <<< Exiting allow_directories()");
        assert!(ov.matched("src/foo", true).is_none());
    }

    #[test]
    fn absolute_path() {
		eprintln!("[ERROR] >>> Entering absolute_path()");
        let ov = ov(&["!/bar"]);
		eprintln!("[ERROR] absolute_path(): ov → {:?}", ov);
    eprintln!("[WARN] <<< Exiting absolute_path()");
        assert!(ov.matched("./foo/bar", false).is_none());
    }

    #[test]
    fn case_insensitive() {
        let ov = OverrideBuilder::new(ROOT)
            .case_insensitive(true)
            .unwrap()
            .add("*.html")
            .unwrap()
            .build()
            .unwrap();
        assert!(ov.matched("foo.html", false).is_whitelist());
        assert!(ov.matched("foo.HTML", false).is_whitelist());
        assert!(ov.matched("foo.htm", false).is_ignore());
        assert!(ov.matched("foo.HTM", false).is_ignore());
    }

    #[test]
    fn default_case_sensitive() {
		eprintln!("[ERROR] >>> Entering default_case_sensitive()");
        let ov =
            OverrideBuilder::new(ROOT).add("*.html").unwrap().build().unwrap();
        assert!(ov.matched("foo.html", false).is_whitelist());
        assert!(ov.matched("foo.HTML", false).is_ignore());
        assert!(ov.matched("foo.htm", false).is_ignore());
		eprintln!("[WARN] <<< Exiting default_case_sensitive()");
        assert!(ov.matched("foo.HTM", false).is_ignore());
    }
}
