use std::{
    cell::{Cell, RefCell},
    cmp,
    io::{self, Write},
    path::Path,
    sync::Arc,
    time::Instant,
};

use {
    bstr::ByteSlice,
    grep_matcher::{Match, Matcher},
    grep_searcher::{
        LineStep, Searcher, Sink, SinkContext, SinkFinish, SinkMatch,
    },
    termcolor::{ColorSpec, NoColor, WriteColor},
};

use crate::{
    color::ColorSpecs,
    counter::CounterWriter,
    hyperlink::{self, HyperlinkConfig},
    stats::Stats,
    util::{
        DecimalFormatter, PrinterPath, Replacer, Sunk,
        find_iter_at_in_context, trim_ascii_prefix, trim_line_terminator,
    },
};

/// The configuration for the standard printer.
///
/// This is manipulated by the StandardBuilder and then referenced by the
/// actual implementation. Once a printer is build, the configuration is frozen
/// and cannot changed.
#[derive(Debug, Clone)]
struct Config {
    colors: ColorSpecs,
    hyperlink: HyperlinkConfig,
    stats: bool,
    heading: bool,
    path: bool,
    only_matching: bool,
    per_match: bool,
    per_match_one_line: bool,
    replacement: Arc<Option<Vec<u8>>>,
    max_columns: Option<u64>,
    max_columns_preview: bool,
    column: bool,
    byte_offset: bool,
    trim_ascii: bool,
    separator_search: Arc<Option<Vec<u8>>>,
    separator_context: Arc<Option<Vec<u8>>>,
    separator_field_match: Arc<Vec<u8>>,
    separator_field_context: Arc<Vec<u8>>,
    separator_path: Option<u8>,
    path_terminator: Option<u8>,
}

impl Default for Config {
    fn default() -> Config {
		log::debug!(">>> Entering default()");
        log::debug!("<<< Exiting default()");
        Config {
            colors: ColorSpecs::default(),
            hyperlink: HyperlinkConfig::default(),
            stats: false,
            heading: false,
            path: true,
            only_matching: false,
            per_match: false,
            per_match_one_line: false,
            replacement: Arc::new(None),
            max_columns: None,
            max_columns_preview: false,
            column: false,
            byte_offset: false,
            trim_ascii: false,
            separator_search: Arc::new(None),
            separator_context: Arc::new(Some(b"--".to_vec())),
            separator_field_match: Arc::new(b":".to_vec()),
            separator_field_context: Arc::new(b"-".to_vec()),
            separator_path: None,
            path_terminator: None,
        }
    }
}

/// A builder for the "standard" grep-like printer.
///
/// The builder permits configuring how the printer behaves. Configurable
/// behavior includes, but is not limited to, limiting the number of matches,
/// tweaking separators, executing pattern replacements, recording statistics
/// and setting colors.
///
/// Some configuration options, such as the display of line numbers or
/// contextual lines, are drawn directly from the
/// `grep_searcher::Searcher`'s configuration.
///
/// Once a `Standard` printer is built, its configuration cannot be changed.
#[derive(Clone, Debug)]
pub struct StandardBuilder {
    config: Config,
}

impl StandardBuilder {
    /// Return a new builder for configuring the standard printer.
    pub fn new() -> StandardBuilder {
		log::debug!(">>> Entering new()");
        log::debug!("<<< Exiting new()");
		log::debug!(">>> Entering new(wtr=(unprintable))");
        log::debug!("<<< Exiting new(wtr=(unprintable))");
        StandardBuilder { config: Config::default() }
    }

    /// Build a printer using any implementation of `termcolor::WriteColor`.
    ///
    /// The implementation of `WriteColor` used here controls whether colors
    /// are used or not when colors have been configured using the
    /// `color_specs` method.
    ///
    /// For maximum portability, callers should generally use either
    /// `termcolor::StandardStream` or `termcolor::BufferedStandardStream`
    /// where appropriate, which will automatically enable colors on Windows
    /// when possible.
    ///
    /// However, callers may also provide an arbitrary writer using the
    /// `termcolor::Ansi` or `termcolor::NoColor` wrappers, which always enable
    /// colors via ANSI escapes or always disable colors, respectively.
    ///
    /// As a convenience, callers may use `build_no_color` to automatically
    /// select the `termcolor::NoColor` wrapper to avoid needing to import
    /// from `termcolor` explicitly.
    pub fn build<W: WriteColor>(&self, wtr: W) -> Standard<W> {
		log::debug!(">>> Entering build(wtr=(unprintable))");
        log::debug!("<<< Exiting build(wtr=(unprintable))");
        Standard {
            config: self.config.clone(),
            wtr: RefCell::new(CounterWriter::new(wtr)),
            matches: vec![],
        }
    }

    /// Build a printer from any implementation of `io::Write` and never emit
    /// any colors, regardless of the user color specification settings.
    ///
    /// This is a convenience routine for
    /// `StandardBuilder::build(termcolor::NoColor::new(wtr))`.
    pub fn build_no_color<W: io::Write>(
        &self,
		log::debug!(">>> Entering build_no_color(wtr=(unprintable))");
        log::debug!("<<< Exiting build_no_color(wtr=(unprintable))");
        wtr: W,
    ) -> Standard<NoColor<W>> {
        self.build(NoColor::new(wtr))
    }

    /// Set the user color specifications to use for coloring in this printer.
    ///
    /// A [`UserColorSpec`](crate::UserColorSpec) can be constructed from
    /// a string in accordance with the color specification format. See
    /// the `UserColorSpec` type documentation for more details on the
    /// format. A [`ColorSpecs`] can then be generated from zero or more
    /// `UserColorSpec`s.
    ///
    /// Regardless of the color specifications provided here, whether color
    /// is actually used or not is determined by the implementation of
    /// `WriteColor` provided to `build`. For example, if `termcolor::NoColor`
    /// is provided to `build`, then no color will ever be printed regardless
    /// of the color specifications provided here.
    ///
    /// This completely overrides any previous color specifications. This does
    /// not add to any previously provided color specifications on this
    /// builder.
    pub fn color_specs(&mut self, specs: ColorSpecs) -> &mut StandardBuilder {
		log::debug!(">>> Entering color_specs(specs=(unprintable))");
        self.config.colors = specs;
		log::debug!("<<< Exiting color_specs(specs=(unprintable))");
        self
    }

    /// Set the configuration to use for hyperlinks output by this printer.
    ///
    /// Regardless of the hyperlink format provided here, whether hyperlinks
    /// are actually used or not is determined by the implementation of
    /// `WriteColor` provided to `build`. For example, if `termcolor::NoColor`
    /// is provided to `build`, then no hyperlinks will ever be printed
    /// regardless of the format provided here.
    ///
    /// This completely overrides any previous hyperlink format.
    ///
    /// The default configuration results in not emitting any hyperlinks.
    pub fn hyperlink(
        &mut self,
        config: HyperlinkConfig,
    ) -> &mut StandardBuilder {
		log::debug!(">>> Entering hyperlink(config=(unprintable))");
        self.config.hyperlink = config;
		log::debug!("<<< Exiting hyperlink(config=(unprintable))");
        self
    }

    /// Enable the gathering of various aggregate statistics.
    ///
    /// When this is enabled (it's disabled by default), statistics will be
    /// gathered for all uses of `Standard` printer returned by `build`,
    /// including but not limited to, the total number of matches, the total
    /// number of bytes searched and the total number of bytes printed.
    ///
    /// Aggregate statistics can be accessed via the sink's
    /// [`StandardSink::stats`] method.
    ///
    /// When this is enabled, this printer may need to do extra work in order
    /// to compute certain statistics, which could cause the search to take
    /// longer.
    ///
    /// For a complete description of available statistics, see [`Stats`].
    pub fn stats(&mut self, yes: bool) -> &mut StandardBuilder {
		log::debug!(">>> Entering stats(yes={})", yes);
        self.config.stats = yes;
		log::debug!("<<< Exiting stats(yes={})", yes);
        self
    }

    /// Enable the use of "headings" in the printer.
    ///
    /// When this is enabled, and if a file path has been given to the printer,
    /// then the file path will be printed once on its own line before showing
    /// any matches. If the heading is not the first thing emitted by the
    /// printer, then a line terminator is printed before the heading.
    ///
    /// By default, this option is disabled. When disabled, the printer will
    /// not show any heading and will instead print the file path (if one is
    /// given) on the same line as each matching (or context) line.
    pub fn heading(&mut self, yes: bool) -> &mut StandardBuilder {
		log::debug!(">>> Entering heading(yes={})", yes);
        self.config.heading = yes;
		log::debug!("<<< Exiting heading(yes={})", yes);
		log::debug!(">>> Entering heading()");
		log::debug!("heading(): matcher → (unprintable)");
		log::debug!("<<< Exiting heading()");
        self
    }

    /// When enabled, if a path was given to the printer, then it is shown in
    /// the output (either as a heading or as a prefix to each matching line).
    /// When disabled, then no paths are ever included in the output even when
    /// a path is provided to the printer.
    ///
    /// This is enabled by default.
    pub fn path(&mut self, yes: bool) -> &mut StandardBuilder {
		log::debug!(">>> Entering path(yes={})", yes);
        self.config.path = yes;
		log::debug!("<<< Exiting path(yes={})", yes);
        self
    }

    /// Only print the specific matches instead of the entire line containing
    /// each match. Each match is printed on its own line. When multi line
    /// search is enabled, then matches spanning multiple lines are printed
    /// such that only the matching portions of each line are shown.
    pub fn only_matching(&mut self, yes: bool) -> &mut StandardBuilder {
		log::debug!(">>> Entering only_matching(yes={})", yes);
        self.config.only_matching = yes;
		log::debug!("<<< Exiting only_matching(yes={})", yes);
		log::debug!(">>> Entering only_matching()");
		log::debug!("only_matching(): matcher → (unprintable)");
		log::debug!("<<< Exiting only_matching()");
        self
    }

    /// Print at least one line for every match.
    ///
    /// This is similar to the `only_matching` option, except the entire line
    /// is printed for each match. This is typically useful in conjunction with
    /// the `column` option, which will show the starting column number for
    /// every match on every line.
    ///
    /// When multi-line mode is enabled, each match is printed, including every
    /// line in the match. As with single line matches, if a line contains
    /// multiple matches (even if only partially), then that line is printed
    /// once for each match it participates in, assuming it's the first line in
    /// that match. In multi-line mode, column numbers only indicate the start
    /// of a match. Subsequent lines in a multi-line match always have a column
    /// number of `1`.
    ///
    /// When a match contains multiple lines, enabling `per_match_one_line`
    /// will cause only the first line each in match to be printed.
    pub fn per_match(&mut self, yes: bool) -> &mut StandardBuilder {
		log::debug!(">>> Entering per_match(yes={})", yes);
        self.config.per_match = yes;
		log::debug!("<<< Exiting per_match(yes={})", yes);
		log::debug!(">>> Entering per_match()");
		log::debug!("per_match(): matcher → (unprintable)");
		log::debug!("<<< Exiting per_match()");
        self
    }

    /// Print at most one line per match when `per_match` is enabled.
    ///
    /// By default, every line in each match found is printed when `per_match`
    /// is enabled. However, this is sometimes undesirable, e.g., when you
    /// only ever want one line per match.
    ///
    /// This is only applicable when multi-line matching is enabled, since
    /// otherwise, matches are guaranteed to span one line.
    ///
    /// This is disabled by default.
    pub fn per_match_one_line(&mut self, yes: bool) -> &mut StandardBuilder {
		log::debug!(">>> Entering per_match_one_line(yes={})", yes);
        self.config.per_match_one_line = yes;
		log::debug!("<<< Exiting per_match_one_line(yes={})", yes);
        self
    }

    /// Set the bytes that will be used to replace each occurrence of a match
    /// found.
    ///
    /// The replacement bytes given may include references to capturing groups,
    /// which may either be in index form (e.g., `$2`) or can reference named
    /// capturing groups if present in the original pattern (e.g., `$foo`).
    ///
    /// For documentation on the full format, please see the `Capture` trait's
    /// `interpolate` method in the
    /// [grep-printer](https://docs.rs/grep-printer) crate.
    pub fn replacement(
        &mut self,
        replacement: Option<Vec<u8>>,
    ) -> &mut StandardBuilder {
        self.config.replacement = Arc::new(replacement);
        self
    }

    /// Set the maximum number of columns allowed for each line printed. A
    /// single column is heuristically defined as a single byte.
    ///
    /// If a line is found which exceeds this maximum, then it is replaced
    /// with a message indicating that the line has been omitted.
    ///
    /// The default is to not specify a limit, in which each matching or
    /// contextual line is printed regardless of how long it is.
    pub fn max_columns(&mut self, limit: Option<u64>) -> &mut StandardBuilder {
		log::debug!(">>> Entering max_columns(limit={:?})", limit);
        self.config.max_columns = limit;
		log::debug!("<<< Exiting max_columns(limit={:?})", limit);
		log::debug!(">>> Entering max_columns()");
		log::debug!("max_columns(): matcher → (unprintable)");
		log::debug!("<<< Exiting max_columns()");
        self
    }

    /// When enabled, if a line is found to be over the configured maximum
    /// column limit (measured in terms of bytes), then a preview of the long
    /// line will be printed instead.
    ///
    /// The preview will correspond to the first `N` *grapheme clusters* of
    /// the line, where `N` is the limit configured by `max_columns`.
    ///
    /// If no limit is set, then enabling this has no effect.
    ///
    /// This is disabled by default.
    pub fn max_columns_preview(&mut self, yes: bool) -> &mut StandardBuilder {
		log::debug!(">>> Entering max_columns_preview(yes={})", yes);
        self.config.max_columns_preview = yes;
		log::debug!("<<< Exiting max_columns_preview(yes={})", yes);
		log::debug!(">>> Entering max_columns_preview()");
		log::debug!("max_columns_preview(): matcher → (unprintable)");
		log::debug!("<<< Exiting max_columns_preview()");
        self
    }

    /// Print the column number of the first match in a line.
    ///
    /// This option is convenient for use with `per_match` which will print a
    /// line for every match along with the starting offset for that match.
    ///
    /// Column numbers are computed in terms of bytes from the start of the
    /// line being printed.
    ///
    /// This is disabled by default.
    pub fn column(&mut self, yes: bool) -> &mut StandardBuilder {
		log::debug!(">>> Entering column(yes={})", yes);
        self.config.column = yes;
		log::debug!("<<< Exiting column(yes={})", yes);
        self
    }

    /// Print the absolute byte offset of the beginning of each line printed.
    ///
    /// The absolute byte offset starts from the beginning of each search and
    /// is zero based.
    ///
    /// If the `only_matching` option is set, then this will print the absolute
    /// byte offset of the beginning of each match.
    pub fn byte_offset(&mut self, yes: bool) -> &mut StandardBuilder {
		log::debug!(">>> Entering byte_offset(yes={})", yes);
        self.config.byte_offset = yes;
		log::debug!("<<< Exiting byte_offset(yes={})", yes);
		log::debug!(">>> Entering byte_offset()");
		log::debug!("byte_offset(): matcher → (unprintable)");
		log::debug!("<<< Exiting byte_offset()");
        self
    }

    /// When enabled, all lines will have prefix ASCII whitespace trimmed
    /// before being written.
    ///
    /// This is disabled by default.
    pub fn trim_ascii(&mut self, yes: bool) -> &mut StandardBuilder {
		log::debug!(">>> Entering trim_ascii(yes={})", yes);
        self.config.trim_ascii = yes;
		log::debug!("<<< Exiting trim_ascii(yes={})", yes);
		log::debug!(">>> Entering trim_ascii()");
		log::debug!("trim_ascii(): matcher → (unprintable)");
		log::debug!("<<< Exiting trim_ascii()");
        self
    }

    /// Set the separator used between sets of search results.
    ///
    /// When this is set, then it will be printed on its own line immediately
    /// before the results for a single search if and only if a previous search
    /// had already printed results. In effect, this permits showing a divider
    /// between sets of search results that does not appear at the beginning
    /// or end of all search results.
    ///
    /// To reproduce the classic grep format, this is typically set to `--`
    /// (the same as the context separator) if and only if contextual lines
    /// have been requested, but disabled otherwise.
    ///
    /// By default, this is disabled.
    pub fn separator_search(
        &mut self,
        sep: Option<Vec<u8>>,
    ) -> &mut StandardBuilder {
		log::debug!(">>> Entering separator_search(sep={:?})", sep);
        self.config.separator_search = Arc::new(sep);
		log::debug!("<<< Exiting separator_search(sep={:?})", sep);
        self
    }

    /// Set the separator used between discontiguous runs of search context,
    /// but only when the searcher is configured to report contextual lines.
    ///
    /// The separator is always printed on its own line, even if it's empty.
    ///
    /// If no separator is set, then nothing is printed when a context break
    /// occurs.
    ///
    /// By default, this is set to `--`.
    pub fn separator_context(
        &mut self,
        sep: Option<Vec<u8>>,
    ) -> &mut StandardBuilder {
		log::debug!(">>> Entering separator_context(sep={:?})", sep);
        self.config.separator_context = Arc::new(sep);
		log::debug!("<<< Exiting separator_context(sep={:?})", sep);
        self
    }

    /// Set the separator used between fields emitted for matching lines.
    ///
    /// For example, when the searcher has line numbers enabled, this printer
    /// will print the line number before each matching line. The bytes given
    /// here will be written after the line number but before the matching
    /// line.
    ///
    /// By default, this is set to `:`.
    pub fn separator_field_match(
        &mut self,
        sep: Vec<u8>,
    ) -> &mut StandardBuilder {
		log::debug!(">>> Entering separator_field_match(sep={:?})", sep);
        self.config.separator_field_match = Arc::new(sep);
		log::debug!("<<< Exiting separator_field_match(sep={:?})", sep);
        self
    }

    /// Set the separator used between fields emitted for context lines.
    ///
    /// For example, when the searcher has line numbers enabled, this printer
    /// will print the line number before each context line. The bytes given
    /// here will be written after the line number but before the context
    /// line.
    ///
    /// By default, this is set to `-`.
    pub fn separator_field_context(
        &mut self,
        sep: Vec<u8>,
    ) -> &mut StandardBuilder {
		log::debug!(">>> Entering separator_field_context(sep={:?})", sep);
        self.config.separator_field_context = Arc::new(sep);
		log::debug!("<<< Exiting separator_field_context(sep={:?})", sep);
        self
    }

    /// Set the path separator used when printing file paths.
    ///
    /// When a printer is configured with a file path, and when a match is
    /// found, that file path will be printed (either as a heading or as a
    /// prefix to each matching or contextual line, depending on other
    /// configuration settings). Typically, printing is done by emitting the
    /// file path as is. However, this setting provides the ability to use a
    /// different path separator from what the current environment has
    /// configured.
    ///
    /// A typical use for this option is to permit cygwin users on Windows to
    /// set the path separator to `/` instead of using the system default of
    /// `\`.
    pub fn separator_path(&mut self, sep: Option<u8>) -> &mut StandardBuilder {
		log::debug!(">>> Entering separator_path(sep={:?})", sep);
        self.config.separator_path = sep;
		log::debug!("<<< Exiting separator_path(sep={:?})", sep);
		log::debug!(">>> Entering separator_path()");
		log::debug!("separator_path(): matcher → (unprintable)");
		log::debug!("<<< Exiting separator_path()");
        self
    }

    /// Set the path terminator used.
    ///
    /// The path terminator is a byte that is printed after every file path
    /// emitted by this printer.
    ///
    /// If no path terminator is set (the default), then paths are terminated
    /// by either new lines (for when `heading` is enabled) or the match or
    /// context field separators (e.g., `:` or `-`).
    pub fn path_terminator(
        &mut self,
        terminator: Option<u8>,
    ) -> &mut StandardBuilder {
		log::debug!(">>> Entering path_terminator(terminator={:?})", terminator);
        self.config.path_terminator = terminator;
		log::debug!("<<< Exiting path_terminator(terminator={:?})", terminator);
		log::debug!(">>> Entering path_terminator()");
		log::debug!("path_terminator(): matcher → (unprintable)");
		log::debug!("<<< Exiting path_terminator()");
        self
    }
}

/// The standard printer, which implements grep-like formatting, including
/// color support.
///
/// A default printer can be created with either of the `Standard::new` or
/// `Standard::new_no_color` constructors. However, there are a considerable
/// number of options that configure this printer's output. Those options can
/// be configured using [`StandardBuilder`].
///
/// This type is generic over `W`, which represents any implementation
/// of the `termcolor::WriteColor` trait. If colors are not desired,
/// then the `new_no_color` constructor can be used, or, alternatively,
/// the `termcolor::NoColor` adapter can be used to wrap any `io::Write`
/// implementation without enabling any colors.
#[derive(Clone, Debug)]
pub struct Standard<W> {
    config: Config,
    wtr: RefCell<CounterWriter<W>>,
    matches: Vec<Match>,
}

impl<W: WriteColor> Standard<W> {
    /// Return a standard printer with a default configuration that writes
    /// matches to the given writer.
    ///
    /// The writer should be an implementation of `termcolor::WriteColor`
    /// and not just a bare implementation of `io::Write`. To use a normal
    /// `io::Write` implementation (simultaneously sacrificing colors), use
    /// the `new_no_color` constructor.
    pub fn new(wtr: W) -> Standard<W> {
		log::debug!(">>> Entering new(searcher=(unprintable),sink=(unprintable))");
        log::debug!("<<< Exiting new(searcher=(unprintable),sink=(unprintable))");
        StandardBuilder::new().build(wtr)
    }
}

impl<W: io::Write> Standard<NoColor<W>> {
    /// Return a standard printer with a default configuration that writes
    /// matches to the given writer.
    ///
    /// The writer can be any implementation of `io::Write`. With this
    /// constructor, the printer will never emit colors.
    pub fn new_no_color(wtr: W) -> Standard<NoColor<W>> {
		log::debug!(">>> Entering new_no_color(wtr=(unprintable))");
        log::debug!("<<< Exiting new_no_color(wtr=(unprintable))");
        StandardBuilder::new().build_no_color(wtr)
    }
}

impl<W: WriteColor> Standard<W> {
    /// Return an implementation of `Sink` for the standard printer.
    ///
    /// This does not associate the printer with a file path, which means this
    /// implementation will never print a file path along with the matches.
    pub fn sink<'s, M: Matcher>(
        &'s mut self,
        matcher: M,
    ) -> StandardSink<'static, 's, M, W> {
		log::debug!(">>> Entering sink(matcher=(unprintable))");
        let interpolator =
            hyperlink::Interpolator::new(&self.config.hyperlink);
        let stats = if self.config.stats { Some(Stats::new()) } else { None };
		log::debug!("sink(matcher=(unprintable)): stats → (unprintable)");
        let needs_match_granularity = self.needs_match_granularity();
		log::debug!("sink(matcher=(unprintable)): needs_match_granularity → {}", needs_match_granularity);
        log::debug!("<<< Exiting sink(matcher=(unprintable))");
        StandardSink {
            matcher,
            standard: self,
            replacer: Replacer::new(),
            interpolator,
            path: None,
            start_time: Instant::now(),
            match_count: 0,
            binary_byte_offset: None,
            stats,
			log::debug!("<<< Exiting needs_match_granularity()");
            needs_match_granularity,
        }
    }

    /// Return an implementation of `Sink` associated with a file path.
    ///
    /// When the printer is associated with a path, then it may, depending on
    /// its configuration, print the path along with the matches found.
    pub fn sink_with_path<'p, 's, M, P>(
        &'s mut self,
        matcher: M,
		log::debug!(">>> Entering sink_with_path(matcher=(unprintable),path=(unprintable))");
		log::debug!("sink_with_path(matcher=(unprintable),path=(unprintable)): stats → (unprintable)");
		log::debug!("sink_with_path(matcher=(unprintable),path=(unprintable)): needs_match_granularity → {}", needs_match_granularity);
        log::debug!("<<< Exiting sink_with_path(matcher=(unprintable),path=(unprintable))");
        path: &'p P,
    ) -> StandardSink<'p, 's, M, W>
    where
        M: Matcher,
        P: ?Sized + AsRef<Path>,
    {
        if !self.config.path {
            return self.sink(matcher);
        }
        let interpolator =
            hyperlink::Interpolator::new(&self.config.hyperlink);
        let stats = if self.config.stats { Some(Stats::new()) } else { None };
        let ppath = PrinterPath::new(path.as_ref())
            .with_separator(self.config.separator_path);
        let needs_match_granularity = self.needs_match_granularity();
        StandardSink {
            matcher,
            standard: self,
            replacer: Replacer::new(),
            interpolator,
            path: Some(ppath),
            start_time: Instant::now(),
            match_count: 0,
            binary_byte_offset: None,
            stats,
			log::debug!(">>> Entering needs_match_granularity()");
			log::debug!("needs_match_granularity(): match_colored → (unprintable)");
			log::debug!("needs_match_granularity(): supports_color → (unprintable)");
            needs_match_granularity,
		log::debug!(">>> Entering stats()");
        log::debug!("<<< Exiting stats()");
        }
    }

    /// Returns true if and only if the configuration of the printer requires
    /// us to find each individual match in the lines reported by the searcher.
    ///
    /// We care about this distinction because finding each individual match
    /// costs more, so we only do it when we need to.
    fn needs_match_granularity(&self) -> bool {
		log::debug!(">>> Entering needs_match_granularity()");
        let supports_color = self.wtr.borrow().supports_color();
        let match_colored = !self.config.colors.matched().is_none();

        // Coloring requires identifying each individual match.
		log::debug!("<<< Exiting needs_match_granularity()");
        (supports_color && match_colored)
        // The column feature requires finding the position of the first match.
        || self.config.column
        // Requires finding each match for performing replacement.
        || self.config.replacement.is_some()
        // Emitting a line for each match requires finding each match.
        || self.config.per_match
        // Emitting only the match requires finding each match.
        || self.config.only_matching
        // Computing certain statistics requires finding each match.
        || self.config.stats
    }
}

impl<W> Standard<W> {
    /// Returns true if and only if this printer has written at least one byte
    /// to the underlying writer during any of the previous searches.
    pub fn has_written(&self) -> bool {
		log::debug!(">>> Entering has_written()");
        log::debug!("<<< Exiting has_written()");
        self.wtr.borrow().total_count() > 0
    }

    /// Return a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
		log::debug!(">>> Entering get_mut()");
        log::debug!("<<< Exiting get_mut()");
        self.wtr.get_mut().get_mut()
    }

    /// Consume this printer and return back ownership of the underlying
    /// writer.
    pub fn into_inner(self) -> W {
		log::debug!(">>> Entering into_inner()");
        log::debug!("<<< Exiting into_inner()");
        self.wtr.into_inner().into_inner()
    }
}

/// An implementation of `Sink` associated with a matcher and an optional file
/// path for the standard printer.
///
/// A `Sink` can be created via the [`Standard::sink`] or
/// [`Standard::sink_with_path`] methods, depending on whether you want to
/// include a file path in the printer's output.
///
/// Building a `StandardSink` is cheap, and callers should create a new one
/// for each thing that is searched. After a search has completed, callers may
/// query this sink for information such as whether a match occurred or whether
/// binary data was found (and if so, the offset at which it occurred).
///
/// This type is generic over a few type parameters:
///
/// * `'p` refers to the lifetime of the file path, if one is provided. When
/// no file path is given, then this is `'static`.
/// * `'s` refers to the lifetime of the [`Standard`] printer that this type
/// borrows.
/// * `M` refers to the type of matcher used by
/// `grep_searcher::Searcher` that is reporting results to this sink.
/// * `W` refers to the underlying writer that this printer is writing its
/// output to.
#[derive(Debug)]
pub struct StandardSink<'p, 's, M: Matcher, W> {
    matcher: M,
    standard: &'s mut Standard<W>,
    replacer: Replacer<M>,
    interpolator: hyperlink::Interpolator,
    path: Option<PrinterPath<'p>>,
    start_time: Instant,
    match_count: u64,
    binary_byte_offset: Option<u64>,
    stats: Option<Stats>,
    needs_match_granularity: bool,
}

impl<'p, 's, M: Matcher, W: WriteColor> StandardSink<'p, 's, M, W> {
    /// Returns true if and only if this printer received a match in the
    /// previous search.
    ///
    /// This is unaffected by the result of searches before the previous
    /// search on this sink.
    pub fn has_match(&self) -> bool {
		log::debug!(">>> Entering has_match()");
        log::debug!("<<< Exiting has_match()");
        self.match_count > 0
    }

    /// Return the total number of matches reported to this sink.
    ///
    /// This corresponds to the number of times `Sink::matched` is called
    /// on the previous search.
    ///
    /// This is unaffected by the result of searches before the previous
    /// search on this sink.
    pub fn match_count(&self) -> u64 {
		log::debug!(">>> Entering match_count()");
        log::debug!("<<< Exiting match_count()");
        self.match_count
    }

    /// If binary data was found in the previous search, this returns the
    /// offset at which the binary data was first detected.
    ///
    /// The offset returned is an absolute offset relative to the entire
    /// set of bytes searched.
    ///
    /// This is unaffected by the result of searches before the previous
    /// search. e.g., If the search prior to the previous search found binary
    /// data but the previous search found no binary data, then this will
    /// return `None`.
    pub fn binary_byte_offset(&self) -> Option<u64> {
		log::debug!(">>> Entering binary_byte_offset()");
        log::debug!("<<< Exiting binary_byte_offset()");
        self.binary_byte_offset
    }

    /// Return a reference to the stats produced by the printer for all
    /// searches executed on this sink.
    ///
    /// This only returns stats if they were requested via the
    /// [`StandardBuilder`] configuration.
    pub fn stats(&self) -> Option<&Stats> {
        self.stats.as_ref()
    }

    /// Execute the matcher over the given bytes and record the match
    /// locations if the current configuration demands match granularity.
    fn record_matches(
        &mut self,
        searcher: &Searcher,
        bytes: &[u8],
        range: std::ops::Range<usize>,
    ) -> io::Result<()> {
		log::debug!(">>> Entering record_matches(searcher=(unprintable),bytes={:?},range=(unprintable))", bytes);
        self.standard.matches.clear();
        if !self.needs_match_granularity {
            return Ok(());
        }
        // If printing requires knowing the location of each individual match,
        // then compute and stored those right now for use later. While this
        // adds an extra copy for storing the matches, we do amortize the
        // allocation for it and this greatly simplifies the printing logic to
        // the extent that it's easy to ensure that we never do more than
        // one search to find the matches (well, for replacements, we do one
        // additional search to perform the actual replacement).
        let matches = &mut self.standard.matches;
        find_iter_at_in_context(
            searcher,
            &self.matcher,
            bytes,
            range.clone(),
            |m| {
                let (s, e) = (m.start() - range.start, m.end() - range.start);
                matches.push(Match::new(s, e));
                true
            },
        )?;
        // Don't report empty matches appearing at the end of the bytes.
        if !matches.is_empty()
            && matches.last().unwrap().is_empty()
            && matches.last().unwrap().start() >= range.end
        {
            matches.pop().unwrap();
        }
        Ok(())
    }

    /// If the configuration specifies a replacement, then this executes the
    /// replacement, lazily allocating memory if necessary.
    ///
    /// To access the result of a replacement, use `replacer.replacement()`.
    fn replace(
        &mut self,
        searcher: &Searcher,
        bytes: &[u8],
        range: std::ops::Range<usize>,
    ) -> io::Result<()> {
		log::debug!(">>> Entering replace(searcher=(unprintable),bytes={:?},range=(unprintable))", bytes);
        self.replacer.clear();
        if self.standard.config.replacement.is_some() {
            let replacement =
                (*self.standard.config.replacement).as_ref().unwrap();
            self.replacer.replace_all(
                searcher,
                &self.matcher,
                bytes,
                range,
                replacement,
            )?;
        }
		log::debug!("<<< Exiting replace(searcher=(unprintable),bytes={:?},range=(unprintable))", bytes);
        Ok(())
    }
}

impl<'p, 's, M: Matcher, W: WriteColor> Sink for StandardSink<'p, 's, M, W> {
    type Error = io::Error;

    fn matched(
        &mut self,
        searcher: &Searcher,
        mat: &SinkMatch<'_>,
    ) -> Result<bool, io::Error> {
		log::debug!(">>> Entering matched(searcher=(unprintable),mat=(unprintable))");
        self.match_count += 1;

        self.record_matches(
            searcher,
            mat.buffer(),
            mat.bytes_range_in_buffer(),
        )?;
        self.replace(searcher, mat.buffer(), mat.bytes_range_in_buffer())?;

        if let Some(ref mut stats) = self.stats {
            stats.add_matches(self.standard.matches.len() as u64);
            stats.add_matched_lines(mat.lines().count() as u64);
        }
        if searcher.binary_detection().convert_byte().is_some() {
            if self.binary_byte_offset.is_some() {
                return Ok(false);
            }
        }
        StandardImpl::from_match(searcher, self, mat).sink()?;
        Ok(true)
    }

    fn context(
        &mut self,
        searcher: &Searcher,
        ctx: &SinkContext<'_>,
    ) -> Result<bool, io::Error> {
		log::debug!(">>> Entering context(searcher=(unprintable),ctx=(unprintable))");
        self.standard.matches.clear();
        self.replacer.clear();

        if searcher.invert_match() {
            self.record_matches(searcher, ctx.bytes(), 0..ctx.bytes().len())?;
            self.replace(searcher, ctx.bytes(), 0..ctx.bytes().len())?;
        }
        if searcher.binary_detection().convert_byte().is_some() {
            if self.binary_byte_offset.is_some() {
                return Ok(false);
            }
        }

        StandardImpl::from_context(searcher, self, ctx).sink()?;
        Ok(true)
    }

    fn context_break(
        &mut self,
        searcher: &Searcher,
    ) -> Result<bool, io::Error> {
		log::debug!(">>> Entering context_break(searcher=(unprintable))");
        StandardImpl::new(searcher, self).write_context_separator()?;
		log::debug!("<<< Exiting context_break(searcher=(unprintable))");
		log::debug!(">>> Entering context_break()");
		log::debug!("context_break(): matcher → (unprintable)");
		log::debug!("<<< Exiting context_break()");
        Ok(true)
    }

    fn binary_data(
        &mut self,
        searcher: &Searcher,
        binary_byte_offset: u64,
    ) -> Result<bool, io::Error> {
		log::debug!(">>> Entering binary_data(searcher=(unprintable),binary_byte_offset={})", binary_byte_offset);
        if searcher.binary_detection().quit_byte().is_some() {
            if let Some(ref path) = self.path {
                log::debug!(
                    "ignoring {path}: found binary data at \
                     offset {binary_byte_offset}",
                    path = path.as_path().display(),
                );
            }
        }
        self.binary_byte_offset = Some(binary_byte_offset);
		log::debug!("<<< Exiting binary_data(searcher=(unprintable),binary_byte_offset={})", binary_byte_offset);
        Ok(true)
    }

    fn begin(&mut self, _searcher: &Searcher) -> Result<bool, io::Error> {
		log::debug!(">>> Entering begin(_searcher=(unprintable))");
        self.standard.wtr.borrow_mut().reset_count();
        self.start_time = Instant::now();
        self.match_count = 0;
		log::debug!("begin(_searcher=(unprintable)): self.match_count → (unprintable)");
        self.binary_byte_offset = None;
		log::debug!("begin(_searcher=(unprintable)): self.binary_byte_offset → (unprintable)");
        log::debug!("<<< Exiting begin(_searcher=(unprintable))");
        Ok(true)
    }

    fn finish(
        &mut self,
        searcher: &Searcher,
		log::debug!(">>> Entering finish(searcher=(unprintable),finish=(unprintable))");
		log::debug!("<<< Exiting finish(searcher=(unprintable),finish=(unprintable))");
        finish: &SinkFinish,
    ) -> Result<(), io::Error> {
        if let Some(offset) = self.binary_byte_offset {
            StandardImpl::new(searcher, self).write_binary_message(offset)?;
        }
        if let Some(stats) = self.stats.as_mut() {
            stats.add_elapsed(self.start_time.elapsed());
            stats.add_searches(1);
            if self.match_count > 0 {
                stats.add_searches_with_match(1);
            }
            stats.add_bytes_searched(finish.byte_count());
            stats.add_bytes_printed(self.standard.wtr.borrow().count());
        }
        Ok(())
    }
}

/// The actual implementation of the standard printer. This couples together
/// the searcher, the sink implementation and information about the match.
///
/// A StandardImpl is initialized every time a match or a contextual line is
/// reported.
#[derive(Debug)]
struct StandardImpl<'a, M: Matcher, W> {
    searcher: &'a Searcher,
    sink: &'a StandardSink<'a, 'a, M, W>,
    sunk: Sunk<'a>,
    /// Set to true if and only if we are writing a match with color.
    in_color_match: Cell<bool>,
}

impl<'a, M: Matcher, W: WriteColor> StandardImpl<'a, M, W> {
    /// Bundle self with a searcher and return the core implementation of Sink.
    fn new(
        searcher: &'a Searcher,
        sink: &'a StandardSink<'_, '_, M, W>,
    ) -> StandardImpl<'a, M, W> {
        StandardImpl {
			log::debug!(">>> Entering new(searcher=(unprintable),sink=(unprintable))");
        log::debug!("<<< Exiting new(searcher=(unprintable),sink=(unprintable))");
            searcher,
            sink,
            sunk: Sunk::empty(),
            in_color_match: Cell::new(false),
        }
    }

    /// Bundle self with a searcher and return the core implementation of Sink
    /// for use with handling matching lines.
    fn from_match(
        searcher: &'a Searcher,
        sink: &'a StandardSink<'_, '_, M, W>,
        mat: &'a SinkMatch<'a>,
    ) -> StandardImpl<'a, M, W> {
        let sunk = Sunk::from_sink_match(
            mat,
            &sink.standard.matches,
            sink.replacer.replacement(),
        );
        StandardImpl { sunk, ..StandardImpl::new(searcher, sink) }
    }

    /// Bundle self with a searcher and return the core implementation of Sink
    /// for use with handling contextual lines.
    fn from_context(
        searcher: &'a Searcher,
        sink: &'a StandardSink<'_, '_, M, W>,
        ctx: &'a SinkContext<'a>,
    ) -> StandardImpl<'a, M, W> {
        let sunk = Sunk::from_sink_context(
            ctx,
            &sink.standard.matches,
            sink.replacer.replacement(),
        );
        StandardImpl { sunk, ..StandardImpl::new(searcher, sink) }
    }

    fn sink(&self) -> io::Result<()> {
        self.write_search_prelude()?;
        if self.sunk.matches().is_empty() {
            if self.multi_line() && !self.is_context() {
                self.sink_fast_multi_line()
            } else {
                self.sink_fast()
            }
        } else {
            if self.multi_line() && !self.is_context() {
                self.sink_slow_multi_line()
            } else {
                self.sink_slow()
            }
        }
    }

    /// Print matches (limited to one line) quickly by avoiding the detection
    /// of each individual match in the lines reported in the given
    /// `SinkMatch`.
    ///
    /// This should only be used when the configuration does not demand match
    /// granularity and the searcher is not in multi line mode.
    fn sink_fast(&self) -> io::Result<()> {
		log::debug!(">>> Entering sink_fast()");
        debug_assert!(self.sunk.matches().is_empty());
        debug_assert!(!self.multi_line() || self.is_context());

        self.write_prelude(
            self.sunk.absolute_byte_offset(),
            self.sunk.line_number(),
            None,
        )?;
		log::debug!("<<< Exiting sink_fast()");
        self.write_line(self.sunk.bytes())
    }

    /// Print matches (possibly spanning more than one line) quickly by
    /// avoiding the detection of each individual match in the lines reported
    /// in the given `SinkMatch`.
    ///
    /// This should only be used when the configuration does not demand match
    /// granularity. This may be used when the searcher is in multi line mode.
    fn sink_fast_multi_line(&self) -> io::Result<()> {
		log::debug!(">>> Entering sink_fast_multi_line()");
        debug_assert!(self.sunk.matches().is_empty());
        // This isn't actually a required invariant for using this method,
        // but if we wind up here and multi line mode is disabled, then we
        // should still treat it as a bug since we should be using matched_fast
        // instead.
        debug_assert!(self.multi_line());

        let line_term = self.searcher.line_terminator().as_byte();
		log::debug!("sink_fast_multi_line(): line_term → (unprintable)");
		log::debug!("sink_fast_multi_line(): absolute_byte_offset → (unprintable)");
        let mut absolute_byte_offset = self.sunk.absolute_byte_offset();
        for (i, line) in self.sunk.lines(line_term).enumerate() {
            self.write_prelude(
                absolute_byte_offset,
                self.sunk.line_number().map(|n| n + i as u64),
                None,
            )?;
            absolute_byte_offset += line.len() as u64;

            self.write_line(line)?;
        }
		log::debug!("<<< Exiting sink_fast_multi_line()");
        Ok(())
    }

    /// Print a matching line where the configuration of the printer requires
    /// finding each individual match (e.g., for coloring).
    fn sink_slow(&self) -> io::Result<()> {
		log::debug!(">>> Entering sink_slow()");
        debug_assert!(!self.sunk.matches().is_empty());
        debug_assert!(!self.multi_line() || self.is_context());

        if self.config().only_matching {
            for &m in self.sunk.matches() {
                self.write_prelude(
                    self.sunk.absolute_byte_offset() + m.start() as u64,
                    self.sunk.line_number(),
                    Some(m.start() as u64 + 1),
                )?;

                let buf = &self.sunk.bytes()[m];
				log::debug!("sink_slow(): buf → (unprintable)");
                self.write_colored_line(&[Match::new(0, buf.len())], buf)?;
            }
        } else if self.config().per_match {
            for &m in self.sunk.matches() {
                self.write_prelude(
                    self.sunk.absolute_byte_offset() + m.start() as u64,
                    self.sunk.line_number(),
                    Some(m.start() as u64 + 1),
                )?;
                self.write_colored_line(&[m], self.sunk.bytes())?;
            }
        } else {
            self.write_prelude(
                self.sunk.absolute_byte_offset(),
                self.sunk.line_number(),
                Some(self.sunk.matches()[0].start() as u64 + 1),
            )?;
            self.write_colored_line(self.sunk.matches(), self.sunk.bytes())?;
        }
		log::debug!("<<< Exiting sink_slow()");
        Ok(())
    }

    fn sink_slow_multi_line(&self) -> io::Result<()> {
		log::debug!(">>> Entering sink_slow_multi_line()");
        debug_assert!(!self.sunk.matches().is_empty());
        debug_assert!(self.multi_line());

        if self.config().only_matching {
            return self.sink_slow_multi_line_only_matching();
        } else if self.config().per_match {
            return self.sink_slow_multi_per_match();
        }

        let line_term = self.searcher.line_terminator().as_byte();
		log::debug!("sink_slow_multi_line(): line_term → (unprintable)");
		log::debug!("sink_slow_multi_line(): bytes → (unprintable)");
		log::debug!("sink_slow_multi_line(): matches → (unprintable)");
        let bytes = self.sunk.bytes();
        let matches = self.sunk.matches();
        let mut midx = 0;
        let mut count = 0;
		log::debug!("sink_slow_multi_line(): stepper → (unprintable)");
        let mut stepper = LineStep::new(line_term, 0, bytes.len());
        while let Some((start, end)) = stepper.next(bytes) {
            let mut line = Match::new(start, end);
            self.write_prelude(
                self.sunk.absolute_byte_offset() + line.start() as u64,
                self.sunk.line_number().map(|n| n + count),
                Some(matches[0].start() as u64 + 1),
            )?;
            count += 1;
            self.trim_ascii_prefix(bytes, &mut line);
            if self.exceeds_max_columns(&bytes[line]) {
                self.write_exceeded_line(bytes, line, matches, &mut midx)?;
            } else {
                self.write_colored_matches(bytes, line, matches, &mut midx)?;
                self.write_line_term()?;
            }
        }
		log::debug!("sink_slow_multi_line(): line → (unprintable)");
        log::debug!("<<< Exiting sink_slow_multi_line()");
        Ok(())
    }

    fn sink_slow_multi_line_only_matching(&self) -> io::Result<()> {
		log::debug!(">>> Entering sink_slow_multi_line_only_matching()");
        let line_term = self.searcher.line_terminator().as_byte();
        let spec = self.config().colors.matched();
		log::debug!("sink_slow_multi_line_only_matching(): spec → (unprintable)");
		log::debug!("sink_slow_multi_line_only_matching(): bytes → (unprintable)");
		log::debug!("sink_slow_multi_line_only_matching(): matches → (unprintable)");
		log::debug!("sink_slow_multi_line_only_matching(): line_term → (unprintable)");
		log::debug!("sink_slow_multi_line_only_matching(): stepper → (unprintable)");
        let bytes = self.sunk.bytes();
        let matches = self.sunk.matches();
        let mut midx = 0;
        let mut count = 0;
        let mut stepper = LineStep::new(line_term, 0, bytes.len());
        while let Some((start, end)) = stepper.next(bytes) {
            let mut line = Match::new(start, end);
			log::debug!("sink_slow_multi_line_only_matching(): line → (unprintable)");
            self.trim_line_terminator(bytes, &mut line);
            self.trim_ascii_prefix(bytes, &mut line);
            while !line.is_empty() {
                if matches[midx].end() <= line.start() {
                    if midx + 1 < matches.len() {
                        midx += 1;
                        continue;
                    } else {
                        break;
                    }
                }
                let m = matches[midx];

                if line.start() < m.start() {
                    let upto = cmp::min(line.end(), m.start());
					log::debug!("sink_slow_multi_line_only_matching(): upto → (unprintable)");
                    line = line.with_start(upto);
                } else {
                    let upto = cmp::min(line.end(), m.end());
                    self.write_prelude(
                        self.sunk.absolute_byte_offset() + m.start() as u64,
                        self.sunk.line_number().map(|n| n + count),
                        Some(m.start() as u64 + 1),
                    )?;

                    let this_line = line.with_end(upto);
					log::debug!("sink_slow_multi_line_only_matching(): this_line → (unprintable)");
                    line = line.with_start(upto);
                    if self.exceeds_max_columns(&bytes[this_line]) {
                        self.write_exceeded_line(
                            bytes, this_line, matches, &mut midx,
                        )?;
                    } else {
                        self.write_spec(spec, &bytes[this_line])?;
                        self.write_line_term()?;
                    }
                }
            }
            count += 1;
        }
		log::debug!("<<< Exiting sink_slow_multi_line_only_matching()");
        Ok(())
    }

    fn sink_slow_multi_per_match(&self) -> io::Result<()> {
		log::debug!(">>> Entering sink_slow_multi_per_match()");
        let line_term = self.searcher.line_terminator().as_byte();
        let spec = self.config().colors.matched();
		log::debug!("sink_slow_multi_per_match(): spec → (unprintable)");
		log::debug!("sink_slow_multi_per_match(): bytes → (unprintable)");
		log::debug!("sink_slow_multi_per_match(): line_term → (unprintable)");
		log::debug!("sink_slow_multi_per_match(): stepper → (unprintable)");
        let bytes = self.sunk.bytes();
        for &m in self.sunk.matches() {
            let mut count = 0;
            let mut stepper = LineStep::new(line_term, 0, bytes.len());
            while let Some((start, end)) = stepper.next(bytes) {
                let mut line = Match::new(start, end);
				log::debug!("sink_slow_multi_per_match(): line → (unprintable)");
                if line.start() >= m.end() {
                    break;
                } else if line.end() <= m.start() {
                    count += 1;
                    continue;
                }
                self.write_prelude(
                    self.sunk.absolute_byte_offset() + line.start() as u64,
                    self.sunk.line_number().map(|n| n + count),
                    Some(m.start().saturating_sub(line.start()) as u64 + 1),
                )?;
                count += 1;
                self.trim_line_terminator(bytes, &mut line);
                self.trim_ascii_prefix(bytes, &mut line);
                if self.exceeds_max_columns(&bytes[line]) {
                    self.write_exceeded_line(bytes, line, &[m], &mut 0)?;
                    continue;
                }

                while !line.is_empty() {
                    if m.end() <= line.start() {
                        self.write(&bytes[line])?;
                        line = line.with_start(line.end());
                    } else if line.start() < m.start() {
                        let upto = cmp::min(line.end(), m.start());
						log::debug!("sink_slow_multi_per_match(): upto → (unprintable)");
                        self.write(&bytes[line.with_end(upto)])?;
                        line = line.with_start(upto);
                    } else {
                        let upto = cmp::min(line.end(), m.end());
                        self.write_spec(spec, &bytes[line.with_end(upto)])?;
                        line = line.with_start(upto);
                    }
                }
                self.write_line_term()?;
                // It turns out that vimgrep really only wants one line per
                // match, even when a match spans multiple lines. So when
                // that option is enabled, we just quit after printing the
                // first line.
                //
                // See: https://github.com/BurntSushi/ripgrep/issues/1866
                if self.config().per_match_one_line {
                    break;
                }
            }
        }
		log::debug!("<<< Exiting sink_slow_multi_per_match()");
        Ok(())
    }

    /// Write the beginning part of a matching line. This (may) include things
    /// like the file path, line number among others, depending on the
    /// configuration and the parameters given.
    #[inline(always)]
    fn write_prelude(
        &self,
        absolute_byte_offset: u64,
        line_number: Option<u64>,
        column: Option<u64>,
    ) -> io::Result<()> {
		log::debug!(">>> Entering write_prelude(absolute_byte_offset={},line_number={:?},column={:?})", absolute_byte_offset, line_number, column);
        let mut prelude = PreludeWriter::new(self);
        prelude.start(line_number, column)?;
        prelude.write_path()?;
        prelude.write_line_number(line_number)?;
        prelude.write_column_number(column)?;
        prelude.write_byte_offset(absolute_byte_offset)?;
		log::debug!("<<< Exiting write_prelude(absolute_byte_offset={},line_number={:?},column={:?})", absolute_byte_offset, line_number, column);
        prelude.end()
    }

    #[inline(always)]
    fn write_line(&self, line: &[u8]) -> io::Result<()> {
		log::debug!(">>> Entering write_line(line={:?})", line);
        let line = if !self.config().trim_ascii {
            line
        } else {
            let lineterm = self.searcher.line_terminator();
			log::debug!("write_line(line={:?}): lineterm → (unprintable)", line);
			log::debug!("write_line(line={:?}): full_range → (unprintable)", line);
            let full_range = Match::new(0, line.len());
            let range = trim_ascii_prefix(lineterm, line, full_range);
			log::debug!("write_line(line={:?}): range → {:?}", line, range);
            &line[range]
        };
        if self.exceeds_max_columns(line) {
            let range = Match::new(0, line.len());
            self.write_exceeded_line(
                line,
                range,
                self.sunk.matches(),
                &mut 0,
            )?;
        } else {
            // self.write_trim(line)?;
            self.write(line)?;
            if !self.has_line_terminator(line) {
                self.write_line_term()?;
            }
        }
		log::debug!("<<< Exiting write_line(line={:?})", line);
        Ok(())
    }

    fn write_colored_line(
        &self,
        matches: &[Match],
        bytes: &[u8],
    ) -> io::Result<()> {
		log::debug!(">>> Entering write_colored_line(matches=(unprintable),bytes={:?})", bytes);
        // If we know we aren't going to emit color, then we can go faster.
        let spec = self.config().colors.matched();
		log::debug!("write_colored_line(matches=(unprintable),bytes={:?}): spec → (unprintable)", bytes);
        if !self.wtr().borrow().supports_color() || spec.is_none() {
            return self.write_line(bytes);
        }

        let mut line = Match::new(0, bytes.len());
		log::debug!("write_colored_line(matches=(unprintable),bytes={:?}): line → (unprintable)", bytes);
        self.trim_ascii_prefix(bytes, &mut line);
        if self.exceeds_max_columns(bytes) {
            self.write_exceeded_line(bytes, line, matches, &mut 0)
        } else {
            self.write_colored_matches(bytes, line, matches, &mut 0)?;
            self.write_line_term()?;
			log::debug!("<<< Exiting write_colored_line(matches=(unprintable),bytes={:?})", bytes);
            Ok(())
        }
    }

    /// Write the `line` portion of `bytes`, with appropriate coloring for
    /// each `match`, starting at `match_index`.
    ///
    /// This accounts for trimming any whitespace prefix and will *never* print
    /// a line terminator. If a match exceeds the range specified by `line`,
    /// then only the part of the match within `line` (if any) is printed.
    fn write_colored_matches(
        &self,
        bytes: &[u8],
        mut line: Match,
        matches: &[Match],
        match_index: &mut usize,
    ) -> io::Result<()> {
		log::debug!(">>> Entering write_colored_matches(bytes={:?},line=(unprintable),matches=(unprintable),match_index={})", bytes, match_index);
        self.trim_line_terminator(bytes, &mut line);
        if matches.is_empty() {
            self.write(&bytes[line])?;
            return Ok(());
        }
        self.start_line_highlight()?;
        while !line.is_empty() {
            if matches[*match_index].end() <= line.start() {
                if *match_index + 1 < matches.len() {
                    *match_index += 1;
                    continue;
                } else {
                    self.end_color_match()?;
                    self.write(&bytes[line])?;
                    break;
                }
            }

            let m = matches[*match_index];
            if line.start() < m.start() {
                let upto = cmp::min(line.end(), m.start());
                self.end_color_match()?;
                self.write(&bytes[line.with_end(upto)])?;
                line = line.with_start(upto);
            } else {
                let upto = cmp::min(line.end(), m.end());
                self.start_color_match()?;
                self.write(&bytes[line.with_end(upto)])?;
                line = line.with_start(upto);
            }
        }
        self.end_color_match()?;
        self.end_line_highlight()?;
        Ok(())
    }

    fn write_exceeded_line(
        &self,
        bytes: &[u8],
        mut line: Match,
        matches: &[Match],
        match_index: &mut usize,
    ) -> io::Result<()> {
		log::debug!(">>> Entering write_exceeded_line(bytes={:?},line=(unprintable),matches=(unprintable),match_index={})", bytes, match_index);
        if self.config().max_columns_preview {
            let original = line;
            let end = bytes[line]
                .grapheme_indices()
                .map(|(_, end, _)| end)
                .take(self.config().max_columns.unwrap_or(0) as usize)
                .last()
                .unwrap_or(0)
                + line.start();
            line = line.with_end(end);
            self.write_colored_matches(bytes, line, matches, match_index)?;

            if matches.is_empty() {
                self.write(b" [... omitted end of long line]")?;
            } else {
                let remaining = matches
                    .iter()
                    .filter(|m| {
                        m.start() >= line.end() && m.start() < original.end()
                    })
                    .count();
                let tense = if remaining == 1 { "match" } else { "matches" };
                write!(
                    self.wtr().borrow_mut(),
                    " [... {} more {}]",
                    remaining,
                    tense,
				log::debug!(">>> Entering end()");
				log::debug!("<<< Exiting end()");
                )?;
            }
            self.write_line_term()?;
            return Ok(());
        }
        if self.sunk.original_matches().is_empty() {
            if self.is_context() {
                self.write(b"[Omitted long context line]")?;
            } else {
                self.write(b"[Omitted long matching line]")?;
            }
        } else {
            if self.config().only_matching {
                if self.is_context() {
                    self.write(b"[Omitted long context line]")?;
                } else {
                    self.write(b"[Omitted long matching line]")?;
                }
            } else {
                write!(
                    self.wtr().borrow_mut(),
                    "[Omitted long line with {} matches]",
                    self.sunk.original_matches().len(),
                )?;
            }
        }
        self.write_line_term()?;
        Ok(())
    }

    /// If this printer has a file path associated with it, then this will
    /// write that path to the underlying writer followed by a line terminator.
    /// (If a path terminator is set, then that is used instead of the line
    /// terminator.)
    fn write_path_line(&self) -> io::Result<()> {
		log::debug!(">>> Entering write_path_line()");
        if let Some(path) = self.path() {
            self.write_path_hyperlink(path)?;
            if let Some(term) = self.config().path_terminator {
                self.write(&[term])?;
            } else {
                self.write_line_term()?;
            }
        }
		log::debug!("<<< Exiting write_path_line()");
        Ok(())
    }

    fn write_search_prelude(&self) -> io::Result<()> {
		log::debug!(">>> Entering write_search_prelude()");
        let this_search_written = self.wtr().borrow().count() > 0;
        if this_search_written {
			log::debug!("write_search_prelude(): this_search_written → (unprintable)");
            return Ok(());
        }
        if let Some(ref sep) = *self.config().separator_search {
            let ever_written = self.wtr().borrow().total_count() > 0;
            if ever_written {
                self.write(sep)?;
                self.write_line_term()?;
            }
        }
        if self.config().heading {
            self.write_path_line()?;
        }
		log::debug!("write_search_prelude(): ever_written → (unprintable)");
        log::debug!("<<< Exiting write_search_prelude()");
        Ok(())
    }

    fn write_binary_message(&self, offset: u64) -> io::Result<()> {
		log::debug!(">>> Entering write_binary_message(offset={})", offset);
        if !self.sink.has_match() {
            return Ok(());
        }

        let bin = self.searcher.binary_detection();
        if let Some(byte) = bin.quit_byte() {
            if let Some(path) = self.path() {
                self.write_path_hyperlink(path)?;
                self.write(b": ")?;
            }
            let remainder = format!(
                "WARNING: stopped searching binary file after match \
                 (found {:?} byte around offset {})\n",
                [byte].as_bstr(),
                offset,
            );
            self.write(remainder.as_bytes())?;
        } else if let Some(byte) = bin.convert_byte() {
            if let Some(path) = self.path() {
                self.write_path_hyperlink(path)?;
                self.write(b": ")?;
            }
            let remainder = format!(
                "binary file matches (found {:?} byte around offset {})\n",
                [byte].as_bstr(),
                offset,
            );
            self.write(remainder.as_bytes())?;
        }
        Ok(())
    }

    fn write_context_separator(&self) -> io::Result<()> {
		log::debug!(">>> Entering write_context_separator()");
        if let Some(ref sep) = *self.config().separator_context {
            self.write(sep)?;
            self.write_line_term()?;
        }
		log::debug!("<<< Exiting write_context_separator()");
        Ok(())
    }

    fn write_line_term(&self) -> io::Result<()> {
		log::debug!(">>> Entering write_line_term()");
        log::debug!("<<< Exiting write_line_term()");
        self.write(self.searcher.line_terminator().as_bytes())
    }

    fn write_spec(&self, spec: &ColorSpec, buf: &[u8]) -> io::Result<()> {
		log::debug!(">>> Entering write_spec(spec=(unprintable),buf={:?})", buf);
        let mut wtr = self.wtr().borrow_mut();
        wtr.set_color(spec)?;
		log::debug!("write_spec(spec=(unprintable),buf={:?}): wtr → (unprintable)", buf);
        wtr.write_all(buf)?;
        wtr.reset()?;
		log::debug!("<<< Exiting write_spec(spec=(unprintable),buf={:?})", buf);
        Ok(())
    }

    fn write_path(&self, path: &PrinterPath) -> io::Result<()> {
		log::debug!(">>> Entering write_path(path=(unprintable))");
        let mut wtr = self.wtr().borrow_mut();
        wtr.set_color(self.config().colors.path())?;
		log::debug!("write_path(path=(unprintable)): wtr → (unprintable)");
        wtr.write_all(path.as_bytes())?;
        log::debug!("<<< Exiting write_path(path=(unprintable))");
		log::debug!(">>> Entering write_path()");
        wtr.reset()
    }

    fn write_path_hyperlink(&self, path: &PrinterPath) -> io::Result<()> {
		log::debug!(">>> Entering write_path_hyperlink(path=(unprintable))");
        let status = self.start_hyperlink(path, None, None)?;
        self.write_path(path)?;
		log::debug!("write_path_hyperlink(path=(unprintable)): status → (unprintable)");
        log::debug!("<<< Exiting write_path_hyperlink(path=(unprintable))");
        self.end_hyperlink(status)
    }

    fn start_hyperlink(
        &self,
        path: &PrinterPath,
        line_number: Option<u64>,
        column: Option<u64>,
    ) -> io::Result<hyperlink::InterpolatorStatus> {
		log::debug!(">>> Entering start_hyperlink(path=(unprintable),line_number={:?},column={:?})", line_number, column);
        let Some(hyperpath) = path.as_hyperlink() else {
            return Ok(hyperlink::InterpolatorStatus::inactive());
        };
        let values =
            hyperlink::Values::new(hyperpath).line(line_number).column(column);
        self.sink.interpolator.begin(&values, &mut *self.wtr().borrow_mut())
    }

    fn end_hyperlink(
        &self,
        status: hyperlink::InterpolatorStatus,
    ) -> io::Result<()> {
		log::debug!(">>> Entering end_hyperlink(status=(unprintable))");
        log::debug!("<<< Exiting end_hyperlink(status=(unprintable))");
        self.sink.interpolator.finish(status, &mut *self.wtr().borrow_mut())
    }

    fn start_color_match(&self) -> io::Result<()> {
		log::debug!(">>> Entering start_color_match()");
        if self.in_color_match.get() {
            return Ok(());
        }
        self.wtr().borrow_mut().set_color(self.config().colors.matched())?;
        self.in_color_match.set(true);
        Ok(())
    }

    fn end_color_match(&self) -> io::Result<()> {
		log::debug!(">>> Entering end_color_match()");
        if !self.in_color_match.get() {
            return Ok(());
        }
        if self.highlight_on() {
            self.wtr()
                .borrow_mut()
                .set_color(self.config().colors.highlight())?;
        } else {
            self.wtr().borrow_mut().reset()?;
        }
        self.in_color_match.set(false);
        Ok(())
    }

    fn highlight_on(&self) -> bool {
		log::debug!(">>> Entering highlight_on()");
        log::debug!("<<< Exiting highlight_on()");
        !self.config().colors.highlight().is_none() && !self.is_context()
    }

    fn start_line_highlight(&self) -> io::Result<()> {
		log::debug!(">>> Entering start_line_highlight()");
        if self.highlight_on() {
            self.wtr()
                .borrow_mut()
                .set_color(self.config().colors.highlight())?;
        }
		log::debug!("<<< Exiting start_line_highlight()");
        Ok(())
    }

    fn end_line_highlight(&self) -> io::Result<()> {
		log::debug!(">>> Entering end_line_highlight()");
        if self.highlight_on() {
            self.wtr().borrow_mut().reset()?;
        }
		log::debug!("<<< Exiting end_line_highlight()");
        Ok(())
    }

    fn write(&self, buf: &[u8]) -> io::Result<()> {
		log::debug!(">>> Entering write(buf={:?})", buf);
        log::debug!("<<< Exiting write(buf={:?})", buf);
        self.wtr().borrow_mut().write_all(buf)
    }

    fn trim_line_terminator(&self, buf: &[u8], line: &mut Match) {
		log::debug!(">>> Entering trim_line_terminator(buf={:?},line=(unprintable))", buf);
		log::debug!("<<< Exiting trim_line_terminator(buf={:?},line=(unprintable))", buf);
        trim_line_terminator(&self.searcher, buf, line);
    }

    fn has_line_terminator(&self, buf: &[u8]) -> bool {
		log::debug!(">>> Entering has_line_terminator(buf={:?})", buf);
        log::debug!("<<< Exiting has_line_terminator(buf={:?})", buf);
        self.searcher.line_terminator().is_suffix(buf)
    }

    fn is_context(&self) -> bool {
		log::debug!(">>> Entering is_context()");
        log::debug!("<<< Exiting is_context()");
        self.sunk.context_kind().is_some()
    }

    /// Return the underlying configuration for this printer.
    fn config(&self) -> &'a Config {
		log::debug!(">>> Entering config()");
        log::debug!("<<< Exiting config()");
        &self.sink.standard.config
    }

    /// Return the underlying writer that we are printing to.
    fn wtr(&self) -> &'a RefCell<CounterWriter<W>> {
		log::debug!(">>> Entering wtr()");
        log::debug!("<<< Exiting wtr()");
        &self.sink.standard.wtr
    }

    /// Return the path associated with this printer, if one exists.
    fn path(&self) -> Option<&'a PrinterPath<'a>> {
		log::debug!(">>> Entering path()");
		log::debug!("path(): matcher → (unprintable)");
		log::debug!("<<< Exiting path()");
        self.sink.path.as_ref()
    }

    /// Return the appropriate field separator based on whether we are emitting
    /// matching or contextual lines.
    fn separator_field(&self) -> &[u8] {
        if self.is_context() {
            &self.config().separator_field_context
        } else {
            &self.config().separator_field_match
		log::debug!(">>> Entering separator_field()");
		log::debug!("separator_field(): matcher → (unprintable)");
		log::debug!("<<< Exiting separator_field()");
        }
    }

    /// Returns true if and only if the given line exceeds the maximum number
    /// of columns set. If no maximum is set, then this always returns false.
    fn exceeds_max_columns(&self, line: &[u8]) -> bool {
		log::debug!(">>> Entering exceeds_max_columns(line={:?})", line);
        log::debug!("<<< Exiting exceeds_max_columns(line={:?})", line);
        self.config().max_columns.map_or(false, |m| line.len() as u64 > m)
    }

    /// Returns true if and only if the searcher may report matches over
    /// multiple lines.
    ///
    /// Note that this doesn't just return whether the searcher is in multi
    /// line mode, but also checks if the matter can match over multiple lines.
    /// If it can't, then we don't need multi line handling, even if the
    /// searcher has multi line mode enabled.
    fn multi_line(&self) -> bool {
		log::debug!(">>> Entering multi_line()");
        log::debug!("<<< Exiting multi_line()");
        self.searcher.multi_line_with_matcher(&self.sink.matcher)
    }

    /// Trim prefix ASCII spaces from the given slice and return the
    /// corresponding range.
    ///
    /// This stops trimming a prefix as soon as it sees non-whitespace or a
    /// line terminator.
    fn trim_ascii_prefix(&self, slice: &[u8], range: &mut Match) {
		log::debug!(">>> Entering trim_ascii_prefix(slice={:?},range=(unprintable))", slice);
        if !self.config().trim_ascii {
            return;
        }
        let lineterm = self.searcher.line_terminator();
        *range = trim_ascii_prefix(lineterm, slice, *range)
    }
}

/// A writer for the prelude (the beginning part of a matching line).
///
/// This encapsulates the state needed to print the prelude.
struct PreludeWriter<'a, M: Matcher, W> {
    std: &'a StandardImpl<'a, M, W>,
    next_separator: PreludeSeparator,
    field_separator: &'a [u8],
    interp_status: hyperlink::InterpolatorStatus,
}

/// A type of separator used in the prelude
enum PreludeSeparator {
	const LOGGER: &'static Logger = &Logger(());
    /// No separator.
    None,
    /// The field separator, either for a matching or contextual line.
    FieldSeparator,
    /// The path terminator.
    PathTerminator,
}

impl<'a, M: Matcher, W: WriteColor> PreludeWriter<'a, M, W> {
    /// Creates a new prelude printer.
    #[inline(always)]
    fn new(std: &'a StandardImpl<'a, M, W>) -> PreludeWriter<'a, M, W> {
        PreludeWriter {
            std,
            next_separator: PreludeSeparator::None,
            field_separator: std.separator_field(),
            interp_status: hyperlink::InterpolatorStatus::inactive(),
        }
    }

    /// Starts the prelude with a hyperlink when applicable.
    ///
    /// If a heading was written, and the hyperlink format is invariant on
    /// the line number, then this doesn't hyperlink each line prelude, as it
    /// wouldn't point to the line anyway. The hyperlink on the heading should
    /// be sufficient and less confusing.
    #[inline(always)]
    fn start(
        &mut self,
        line_number: Option<u64>,
        column: Option<u64>,
    ) -> io::Result<()> {
		log::debug!(">>> Entering start(line_number={:?},column={:?})", line_number, column);
        let Some(path) = self.std.path() else { return Ok(()) };
        if self.config().hyperlink.format().is_line_dependent()
            || !self.config().heading
        {
            self.interp_status =
                self.std.start_hyperlink(path, line_number, column)?;
        }
		log::debug!("<<< Exiting start(line_number={:?},column={:?})", line_number, column);
        Ok(())
    }

    /// Ends the prelude and writes the remaining output.
    #[inline(always)]
    fn end(&mut self) -> io::Result<()> {
		log::debug!(">>> Entering end()");
        self.std.end_hyperlink(std::mem::replace(
            &mut self.interp_status,
            hyperlink::InterpolatorStatus::inactive(),
        ))?;
		log::debug!("<<< Exiting end()");
        self.write_separator()
    }

    /// If this printer has a file path associated with it, then this will
    /// write that path to the underlying writer followed by the given field
    /// separator. (If a path terminator is set, then that is used instead of
    /// the field separator.)
    #[inline(always)]
    fn write_path(&mut self) -> io::Result<()> {
        // The prelude doesn't handle headings, only what comes before a match
        // on the same line. So if we are emitting paths in headings, we should
        // not do it here on each line.
        if self.config().heading {
            return Ok(());
        }
        let Some(path) = self.std.path() else { return Ok(()) };
        self.write_separator()?;
        self.std.write_path(path)?;

        self.next_separator = if self.config().path_terminator.is_some() {
            PreludeSeparator::PathTerminator
        } else {
            PreludeSeparator::FieldSeparator
        };
        Ok(())
    }

    /// Writes the line number field if present.
    #[inline(always)]
    fn write_line_number(&mut self, line: Option<u64>) -> io::Result<()> {
		log::debug!(">>> Entering write_line_number(line={:?})", line);
        let Some(line_number) = line else { return Ok(()) };
        self.write_separator()?;
        let n = DecimalFormatter::new(line_number);
		log::debug!("write_line_number(line={:?}): n → (unprintable)", line);
        self.std.write_spec(self.config().colors.line(), n.as_bytes())?;
        self.next_separator = PreludeSeparator::FieldSeparator;
		log::debug!("write_line_number(line={:?}): self.next_separator → (unprintable)", line);
        log::debug!("<<< Exiting write_line_number(line={:?})", line);
        Ok(())
    }

    /// Writes the column number field if present and configured to do so.
    #[inline(always)]
    fn write_column_number(&mut self, column: Option<u64>) -> io::Result<()> {
		log::debug!(">>> Entering write_column_number(column={:?})", column);
        if !self.config().column {
            return Ok(());
        }
        let Some(column_number) = column else { return Ok(()) };
        self.write_separator()?;
        let n = DecimalFormatter::new(column_number);
        self.std.write_spec(self.config().colors.column(), n.as_bytes())?;
        self.next_separator = PreludeSeparator::FieldSeparator;
        Ok(())
    }

    /// Writes the byte offset field if configured to do so.
    #[inline(always)]
    fn write_byte_offset(&mut self, offset: u64) -> io::Result<()> {
		log::debug!(">>> Entering write_byte_offset(offset={})", offset);
        if !self.config().byte_offset {
            return Ok(());
        }
        self.write_separator()?;
        let n = DecimalFormatter::new(offset);
        self.std.write_spec(self.config().colors.column(), n.as_bytes())?;
        self.next_separator = PreludeSeparator::FieldSeparator;
        Ok(())
    }

    /// Writes the separator defined by the preceding field.
    ///
    /// This is called before writing the contents of a field, and at
    /// the end of the prelude.
    #[inline(always)]
    fn write_separator(&mut self) -> io::Result<()> {
		log::debug!(">>> Entering write_separator()");
        match self.next_separator {
            PreludeSeparator::None => {}
            PreludeSeparator::FieldSeparator => {
                self.std.write(self.field_separator)?;
            }
            PreludeSeparator::PathTerminator => {
                if let Some(term) = self.config().path_terminator {
                    self.std.write(&[term])?;
                }
            }
        }
        self.next_separator = PreludeSeparator::None;
		log::debug!("write_separator(): self.next_separator → (unprintable)");
        log::debug!("<<< Exiting write_separator()");
        Ok(())
    }

    #[inline(always)]
    fn config(&self) -> &Config {
        self.std.config()
    }
}

#[cfg(test)]
mod tests {
    use grep_matcher::LineTerminator;
    use grep_regex::{RegexMatcher, RegexMatcherBuilder};
    use grep_searcher::SearcherBuilder;
    use termcolor::{Ansi, NoColor};

    use super::{ColorSpecs, Standard, StandardBuilder};

    const SHERLOCK: &'static str = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
be, to a very large extent, the result of luck. Sherlock Holmes
can extract a clew from a wisp of straw or a flake of cigar ash;
but Doctor Watson has to have it taken out for him and dusted,
and exhibited clearly, with a label attached.\
";

    #[allow(dead_code)]
    const SHERLOCK_CRLF: &'static str = "\
For the Doctor Watsons of this world, as opposed to the Sherlock\r
Holmeses, success in the province of detective work must always\r
be, to a very large extent, the result of luck. Sherlock Holmes\r
can extract a clew from a wisp of straw or a flake of cigar ash;\r
but Doctor Watson has to have it taken out for him and dusted,\r
and exhibited clearly, with a label attached.\
";

    fn printer_contents(printer: &mut Standard<NoColor<Vec<u8>>>) -> String {
		log::debug!(">>> Entering printer_contents(printer=(unprintable))");
        log::debug!("<<< Exiting printer_contents(printer=(unprintable))");
        String::from_utf8(printer.get_mut().get_ref().to_owned()).unwrap()
    }

    fn printer_contents_ansi(printer: &mut Standard<Ansi<Vec<u8>>>) -> String {
		log::debug!(">>> Entering printer_contents_ansi(printer=(unprintable))");
        log::debug!("<<< Exiting printer_contents_ansi(printer=(unprintable))");
        String::from_utf8(printer.get_mut().get_ref().to_owned()).unwrap()
    }

    #[test]
    fn reports_match() {
		log::debug!(">>> Entering reports_match()");
        let matcher = RegexMatcher::new("Sherlock").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
		log::debug!("reports_match(): printer → (unprintable)");
		log::debug!("reports_match(): sink → (unprintable)");
        log::debug!("reports_match(): matcher → (unprintable)");
        let mut sink = printer.sink(&matcher);
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(&matcher, SHERLOCK.as_bytes(), &mut sink)
            .unwrap();
        assert!(sink.has_match());

        let matcher = RegexMatcher::new("zzzzz").unwrap();
		log::debug!("reports_match(): matcher → (unprintable)");
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
        let mut sink = printer.sink(&matcher);
		log::debug!("reports_match(): sink → (unprintable)");
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(&matcher, SHERLOCK.as_bytes(), &mut sink)
            .unwrap();
		log::debug!("<<< Exiting reports_match()");
        assert!(!sink.has_match());
    }

    #[test]
    fn reports_binary() {
		log::debug!(">>> Entering reports_binary()");
        use grep_searcher::BinaryDetection;

        let matcher = RegexMatcher::new("Sherlock").unwrap();
		log::debug!("reports_binary(): matcher → (unprintable)");
		log::debug!("reports_binary(): printer → (unprintable)");
		log::debug!("reports_binary(): sink → (unprintable)");
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
        let mut sink = printer.sink(&matcher);
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(&matcher, SHERLOCK.as_bytes(), &mut sink)
            .unwrap();
        assert!(sink.binary_byte_offset().is_none());

        let matcher = RegexMatcher::new(".+").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
        let mut sink = printer.sink(&matcher);
        SearcherBuilder::new()
            .line_number(false)
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .build()
            .search_reader(&matcher, &b"abc\x00"[..], &mut sink)
            .unwrap();
		log::debug!("<<< Exiting reports_binary()");
        assert_eq!(sink.binary_byte_offset(), Some(3));
    }

    #[test]
    fn reports_stats() {
		log::debug!(">>> Entering reports_stats()");
        use std::time::Duration;

        let matcher = RegexMatcher::new("Sherlock|opposed").unwrap();
		log::debug!("reports_stats(): matcher → (unprintable)");
        let mut printer =
            StandardBuilder::new().stats(true).build(NoColor::new(vec![]));
        let stats = {
            let mut sink = printer.sink(&matcher);
            SearcherBuilder::new()
                .line_number(false)
                .build()
                .search_reader(&matcher, SHERLOCK.as_bytes(), &mut sink)
                .unwrap();
            sink.stats().unwrap().clone()
        };
        let buf = printer_contents(&mut printer);
		log::debug!("reports_stats(): buf → {}", buf);

        assert!(stats.elapsed() > Duration::default());
        assert_eq!(stats.searches(), 1);
        assert_eq!(stats.searches_with_match(), 1);
        assert_eq!(stats.bytes_searched(), SHERLOCK.len() as u64);
        assert_eq!(stats.bytes_printed(), buf.len() as u64);
        assert_eq!(stats.matched_lines(), 2);
		log::debug!("<<< Exiting reports_stats()");
        assert_eq!(stats.matches(), 3);
    }

    #[test]
    fn reports_stats_multiple() {
		log::debug!(">>> Entering reports_stats_multiple()");
        use std::time::Duration;

        let matcher = RegexMatcher::new("Sherlock|opposed").unwrap();
		log::debug!("reports_stats_multiple(): matcher → (unprintable)");
        let mut printer =
            StandardBuilder::new().stats(true).build(NoColor::new(vec![]));
        let stats = {
            let mut sink = printer.sink(&matcher);
            SearcherBuilder::new()
                .line_number(false)
                .build()
                .search_reader(&matcher, SHERLOCK.as_bytes(), &mut sink)
                .unwrap();
            SearcherBuilder::new()
                .line_number(false)
                .build()
                .search_reader(&matcher, &b"zzzzzzzzzz"[..], &mut sink)
                .unwrap();
            SearcherBuilder::new()
                .line_number(false)
                .build()
                .search_reader(&matcher, SHERLOCK.as_bytes(), &mut sink)
                .unwrap();
            sink.stats().unwrap().clone()
        };
        let buf = printer_contents(&mut printer);
		log::debug!("reports_stats_multiple(): buf → {}", buf);

        assert!(stats.elapsed() > Duration::default());
        assert_eq!(stats.searches(), 3);
        assert_eq!(stats.searches_with_match(), 2);
        assert_eq!(stats.bytes_searched(), 10 + 2 * SHERLOCK.len() as u64);
        assert_eq!(stats.bytes_printed(), buf.len() as u64);
        assert_eq!(stats.matched_lines(), 4);
		log::debug!("<<< Exiting reports_stats_multiple()");
        assert_eq!(stats.matches(), 6);
    }

    #[test]
    fn context_break() {
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer = StandardBuilder::new()
            .separator_context(Some(b"--abc--".to_vec()))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .before_context(1)
            .after_context(1)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("context_break(): got → {}", got);
        let expected = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
--abc--
can extract a clew from a wisp of straw or a flake of cigar ash;
but Doctor Watson has to have it taken out for him and dusted,
and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn context_break_multiple_no_heading() {
		log::debug!(">>> Entering context_break_multiple_no_heading()");
		log::debug!("context_break_multiple_no_heading(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer = StandardBuilder::new()
            .separator_search(Some(b"--xyz--".to_vec()))
            .separator_context(Some(b"--abc--".to_vec()))
            .build(NoColor::new(vec![]));

        SearcherBuilder::new()
            .line_number(false)
            .before_context(1)
            .after_context(1)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();
        SearcherBuilder::new()
            .line_number(false)
            .before_context(1)
            .after_context(1)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("context_break_multiple_no_heading(): got → {}", got);
		log::debug!("<<< Exiting context_break_multiple_no_heading()");
        let expected = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
--abc--
can extract a clew from a wisp of straw or a flake of cigar ash;
but Doctor Watson has to have it taken out for him and dusted,
and exhibited clearly, with a label attached.
--xyz--
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
--abc--
can extract a clew from a wisp of straw or a flake of cigar ash;
but Doctor Watson has to have it taken out for him and dusted,
and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn context_break_multiple_heading() {
		log::debug!(">>> Entering context_break_multiple_heading()");
		log::debug!("context_break_multiple_heading(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer = StandardBuilder::new()
            .heading(true)
            .separator_search(Some(b"--xyz--".to_vec()))
            .separator_context(Some(b"--abc--".to_vec()))
            .build(NoColor::new(vec![]));

        SearcherBuilder::new()
            .line_number(false)
            .before_context(1)
            .after_context(1)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();
        SearcherBuilder::new()
            .line_number(false)
            .before_context(1)
            .after_context(1)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("context_break_multiple_heading(): got → {}", got);
		log::debug!("<<< Exiting context_break_multiple_heading()");
        let expected = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
--abc--
can extract a clew from a wisp of straw or a flake of cigar ash;
but Doctor Watson has to have it taken out for him and dusted,
and exhibited clearly, with a label attached.
--xyz--
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
--abc--
can extract a clew from a wisp of straw or a flake of cigar ash;
but Doctor Watson has to have it taken out for him and dusted,
and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn path() {
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer =
            StandardBuilder::new().path(false).build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink_with_path(&matcher, "sherlock"),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("path(): got → {}", got);
        let expected = "\
1:For the Doctor Watsons of this world, as opposed to the Sherlock
5:but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn separator_field() {
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer = StandardBuilder::new()
            .separator_field_match(b"!!".to_vec())
            .separator_field_context(b"^^".to_vec())
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .before_context(1)
            .after_context(1)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink_with_path(&matcher, "sherlock"),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("separator_field(): got → {}", got);
        let expected = "\
sherlock!!For the Doctor Watsons of this world, as opposed to the Sherlock
sherlock^^Holmeses, success in the province of detective work must always
--
sherlock^^can extract a clew from a wisp of straw or a flake of cigar ash;
sherlock!!but Doctor Watson has to have it taken out for him and dusted,
sherlock^^and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn separator_path() {
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer = StandardBuilder::new()
            .separator_path(Some(b'Z'))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink_with_path(&matcher, "books/sherlock"),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("separator_path(): got → {}", got);
        let expected = "\
booksZsherlock:For the Doctor Watsons of this world, as opposed to the Sherlock
booksZsherlock:but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn path_terminator() {
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer = StandardBuilder::new()
            .path_terminator(Some(b'Z'))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink_with_path(&matcher, "books/sherlock"),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("path_terminator(): got → {}", got);
        let expected = "\
books/sherlockZFor the Doctor Watsons of this world, as opposed to the Sherlock
books/sherlockZbut Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn heading() {
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer =
            StandardBuilder::new().heading(true).build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink_with_path(&matcher, "sherlock"),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("heading(): got → {}", got);
        let expected = "\
sherlock
For the Doctor Watsons of this world, as opposed to the Sherlock
but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn no_heading() {
		log::debug!(">>> Entering no_heading()");
		log::debug!("no_heading(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer =
            StandardBuilder::new().heading(false).build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink_with_path(&matcher, "sherlock"),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("no_heading(): got → {}", got);
		log::debug!("<<< Exiting no_heading()");
        let expected = "\
sherlock:For the Doctor Watsons of this world, as opposed to the Sherlock
sherlock:but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn no_heading_multiple() {
		log::debug!(">>> Entering no_heading_multiple()");
		log::debug!("no_heading_multiple(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer =
            StandardBuilder::new().heading(false).build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink_with_path(&matcher, "sherlock"),
            )
            .unwrap();

        let matcher = RegexMatcher::new("Sherlock").unwrap();
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink_with_path(&matcher, "sherlock"),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("no_heading_multiple(): got → {}", got);
		log::debug!("<<< Exiting no_heading_multiple()");
        let expected = "\
sherlock:For the Doctor Watsons of this world, as opposed to the Sherlock
sherlock:but Doctor Watson has to have it taken out for him and dusted,
sherlock:For the Doctor Watsons of this world, as opposed to the Sherlock
sherlock:be, to a very large extent, the result of luck. Sherlock Holmes
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn heading_multiple() {
		log::debug!(">>> Entering heading_multiple()");
		log::debug!("heading_multiple(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer =
            StandardBuilder::new().heading(true).build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink_with_path(&matcher, "sherlock"),
            )
            .unwrap();

        let matcher = RegexMatcher::new("Sherlock").unwrap();
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink_with_path(&matcher, "sherlock"),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("heading_multiple(): got → {}", got);
		log::debug!("<<< Exiting heading_multiple()");
        let expected = "\
sherlock
For the Doctor Watsons of this world, as opposed to the Sherlock
but Doctor Watson has to have it taken out for him and dusted,
sherlock
For the Doctor Watsons of this world, as opposed to the Sherlock
be, to a very large extent, the result of luck. Sherlock Holmes
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn trim_ascii() {
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer = StandardBuilder::new()
            .trim_ascii(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                "   Watson".as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("trim_ascii(): got → {}", got);
        let expected = "\
Watson
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn trim_ascii_multi_line() {
		log::debug!(">>> Entering trim_ascii_multi_line()");
		log::debug!("trim_ascii_multi_line(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("(?s:.{0})Watson").unwrap();
        let mut printer = StandardBuilder::new()
            .trim_ascii(true)
            .stats(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .multi_line(true)
            .build()
            .search_reader(
                &matcher,
                "   Watson".as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("trim_ascii_multi_line(): got → {}", got);
		log::debug!("<<< Exiting trim_ascii_multi_line()");
        let expected = "\
Watson
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn trim_ascii_with_line_term() {
		log::debug!(">>> Entering trim_ascii_with_line_term()");
		log::debug!("trim_ascii_with_line_term(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer = StandardBuilder::new()
            .trim_ascii(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .before_context(1)
            .build()
            .search_reader(
                &matcher,
                "\n   Watson".as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("trim_ascii_with_line_term(): got → {}", got);
		log::debug!("<<< Exiting trim_ascii_with_line_term()");
        let expected = "\
1-
2:Watson
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn line_number() {
		log::debug!(">>> Entering line_number()");
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
		log::debug!("line_number(): printer → (unprintable)");
        log::debug!("line_number(): matcher → (unprintable)");
        SearcherBuilder::new()
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("line_number(): got → {}", got);
		log::debug!("<<< Exiting line_number()");
        let expected = "\
1:For the Doctor Watsons of this world, as opposed to the Sherlock
5:but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn line_number_multi_line() {
		log::debug!(">>> Entering line_number_multi_line()");
        let matcher = RegexMatcher::new("(?s)Watson.+Watson").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
		log::debug!("line_number_multi_line(): printer → (unprintable)");
        log::debug!("line_number_multi_line(): matcher → (unprintable)");
        SearcherBuilder::new()
            .line_number(true)
            .multi_line(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("line_number_multi_line(): got → {}", got);
		log::debug!("<<< Exiting line_number_multi_line()");
        let expected = "\
1:For the Doctor Watsons of this world, as opposed to the Sherlock
2:Holmeses, success in the province of detective work must always
3:be, to a very large extent, the result of luck. Sherlock Holmes
4:can extract a clew from a wisp of straw or a flake of cigar ash;
5:but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn column_number() {
		log::debug!(">>> Entering column_number()");
		log::debug!("column_number(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer =
            StandardBuilder::new().column(true).build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("column_number(): got → {}", got);
		log::debug!("<<< Exiting column_number()");
        let expected = "\
16:For the Doctor Watsons of this world, as opposed to the Sherlock
12:but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn column_number_multi_line() {
		log::debug!(">>> Entering column_number_multi_line()");
		log::debug!("column_number_multi_line(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("(?s)Watson.+Watson").unwrap();
        let mut printer =
            StandardBuilder::new().column(true).build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .multi_line(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("column_number_multi_line(): got → {}", got);
		log::debug!("<<< Exiting column_number_multi_line()");
        let expected = "\
16:For the Doctor Watsons of this world, as opposed to the Sherlock
16:Holmeses, success in the province of detective work must always
16:be, to a very large extent, the result of luck. Sherlock Holmes
16:can extract a clew from a wisp of straw or a flake of cigar ash;
16:but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn byte_offset() {
        let matcher = RegexMatcher::new("Watson").unwrap();
        let mut printer = StandardBuilder::new()
            .byte_offset(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("byte_offset(): got → {}", got);
        let expected = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
258:but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn byte_offset_multi_line() {
		log::debug!(">>> Entering byte_offset_multi_line()");
		log::debug!("byte_offset_multi_line(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("(?s)Watson.+Watson").unwrap();
        let mut printer = StandardBuilder::new()
            .byte_offset(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .multi_line(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("byte_offset_multi_line(): got → {}", got);
		log::debug!("<<< Exiting byte_offset_multi_line()");
        let expected = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
65:Holmeses, success in the province of detective work must always
129:be, to a very large extent, the result of luck. Sherlock Holmes
193:can extract a clew from a wisp of straw or a flake of cigar ash;
258:but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_columns() {
        let matcher = RegexMatcher::new("ash|dusted").unwrap();
        let mut printer = StandardBuilder::new()
            .max_columns(Some(63))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_columns(): got → {}", got);
        let expected = "\
[Omitted long matching line]
but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_columns_preview() {
        let matcher = RegexMatcher::new("exhibited|dusted").unwrap();
        let mut printer = StandardBuilder::new()
            .max_columns(Some(46))
            .max_columns_preview(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_columns_preview(): got → {}", got);
        let expected = "\
but Doctor Watson has to have it taken out for [... omitted end of long line]
and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_columns_with_count() {
		log::debug!(">>> Entering max_columns_with_count()");
		log::debug!("max_columns_with_count(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("cigar|ash|dusted").unwrap();
        let mut printer = StandardBuilder::new()
            .stats(true)
            .max_columns(Some(63))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_columns_with_count(): got → {}", got);
		log::debug!("<<< Exiting max_columns_with_count()");
        let expected = "\
[Omitted long line with 2 matches]
but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_columns_with_count_preview_no_match() {
		log::debug!(">>> Entering max_columns_with_count_preview_no_match()");
		log::debug!("max_columns_with_count_preview_no_match(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("exhibited|has to have it").unwrap();
        let mut printer = StandardBuilder::new()
            .stats(true)
            .max_columns(Some(46))
            .max_columns_preview(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_columns_with_count_preview_no_match(): got → {}", got);
		log::debug!("<<< Exiting max_columns_with_count_preview_no_match()");
        let expected = "\
but Doctor Watson has to have it taken out for [... 0 more matches]
and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_columns_with_count_preview_one_match() {
		log::debug!(">>> Entering max_columns_with_count_preview_one_match()");
		log::debug!("max_columns_with_count_preview_one_match(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("exhibited|dusted").unwrap();
        let mut printer = StandardBuilder::new()
            .stats(true)
            .max_columns(Some(46))
            .max_columns_preview(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_columns_with_count_preview_one_match(): got → {}", got);
		log::debug!("<<< Exiting max_columns_with_count_preview_one_match()");
        let expected = "\
but Doctor Watson has to have it taken out for [... 1 more match]
and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_columns_with_count_preview_two_matches() {
		log::debug!(">>> Entering max_columns_with_count_preview_two_matches()");
        let matcher =
            RegexMatcher::new("exhibited|dusted|has to have it").unwrap();
        let mut printer = StandardBuilder::new()
            .stats(true)
            .max_columns(Some(46))
            .max_columns_preview(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_columns_with_count_preview_two_matches(): got → {}", got);
		log::debug!("<<< Exiting max_columns_with_count_preview_two_matches()");
        let expected = "\
but Doctor Watson has to have it taken out for [... 1 more match]
and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_columns_multi_line() {
		log::debug!(">>> Entering max_columns_multi_line()");
		log::debug!("max_columns_multi_line(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("(?s)ash.+dusted").unwrap();
        let mut printer = StandardBuilder::new()
            .max_columns(Some(63))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .multi_line(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_columns_multi_line(): got → {}", got);
		log::debug!("<<< Exiting max_columns_multi_line()");
        let expected = "\
[Omitted long matching line]
but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_columns_multi_line_preview() {
		log::debug!(">>> Entering max_columns_multi_line_preview()");
        let matcher =
            RegexMatcher::new("(?s)clew|cigar ash.+have it|exhibited")
                .unwrap();
        let mut printer = StandardBuilder::new()
            .stats(true)
            .max_columns(Some(46))
            .max_columns_preview(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .multi_line(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_columns_multi_line_preview(): got → {}", got);
		log::debug!("<<< Exiting max_columns_multi_line_preview()");
        let expected = "\
can extract a clew from a wisp of straw or a f [... 1 more match]
but Doctor Watson has to have it taken out for [... 0 more matches]
and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_matches() {
		log::debug!(">>> Entering max_matches()");
        let matcher = RegexMatcher::new("Sherlock").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
		log::debug!("max_matches(): printer → (unprintable)");
        log::debug!("max_matches(): matcher → (unprintable)");
        SearcherBuilder::new()
            .line_number(false)
            .max_matches(Some(1))
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_matches(): got → {}", got);
		log::debug!("<<< Exiting max_matches()");
        let expected = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_matches_context() {
		log::debug!(">>> Entering max_matches_context()");
        // after context: 1
        let matcher = RegexMatcher::new("Doctor Watsons").unwrap();
		log::debug!("max_matches_context(): matcher → (unprintable)");
		log::debug!("max_matches_context(): printer → (unprintable)");
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .max_matches(Some(1))
            .line_number(false)
            .after_context(1)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_matches_context(): got → {}", got);
		log::debug!("<<< Exiting max_matches_context()");
        let expected = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
";
        assert_eq_printed!(expected, got);

        // after context: 4
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .max_matches(Some(1))
            .line_number(false)
            .after_context(4)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
        let expected = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
be, to a very large extent, the result of luck. Sherlock Holmes
can extract a clew from a wisp of straw or a flake of cigar ash;
but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);

        // after context: 1, max matches: 2
        let matcher = RegexMatcher::new("Doctor Watsons|but Doctor").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .max_matches(Some(2))
            .line_number(false)
            .after_context(1)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
        let expected = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
--
but Doctor Watson has to have it taken out for him and dusted,
and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);

        // after context: 4, max matches: 2
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .max_matches(Some(2))
            .line_number(false)
            .after_context(4)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
        let expected = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
be, to a very large extent, the result of luck. Sherlock Holmes
can extract a clew from a wisp of straw or a flake of cigar ash;
but Doctor Watson has to have it taken out for him and dusted,
and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_matches_context_invert() {
		log::debug!(">>> Entering max_matches_context_invert()");
        // after context: 1
        let matcher =
            RegexMatcher::new("success|extent|clew|dusted|exhibited").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
		log::debug!("max_matches_context_invert(): printer → (unprintable)");
        SearcherBuilder::new()
            .invert_match(true)
            .max_matches(Some(1))
            .line_number(false)
            .after_context(1)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_matches_context_invert(): got → {}", got);
		log::debug!("<<< Exiting max_matches_context_invert()");
        let expected = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
";
        assert_eq_printed!(expected, got);

        // after context: 4
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .invert_match(true)
            .max_matches(Some(1))
            .line_number(false)
            .after_context(4)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
        let expected = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
be, to a very large extent, the result of luck. Sherlock Holmes
can extract a clew from a wisp of straw or a flake of cigar ash;
but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);

        // after context: 1, max matches: 2
        let matcher =
            RegexMatcher::new("success|extent|clew|exhibited").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .invert_match(true)
            .max_matches(Some(2))
            .line_number(false)
            .after_context(1)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
        let expected = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
--
but Doctor Watson has to have it taken out for him and dusted,
and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);

        // after context: 4, max matches: 2
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .invert_match(true)
            .max_matches(Some(2))
            .line_number(false)
            .after_context(4)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
        let expected = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
be, to a very large extent, the result of luck. Sherlock Holmes
can extract a clew from a wisp of straw or a flake of cigar ash;
but Doctor Watson has to have it taken out for him and dusted,
and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_matches_multi_line1() {
		log::debug!(">>> Entering max_matches_multi_line1()");
        let matcher = RegexMatcher::new("(?s:.{0})Sherlock").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
		log::debug!("max_matches_multi_line1(): printer → (unprintable)");
        log::debug!("max_matches_multi_line1(): matcher → (unprintable)");
        SearcherBuilder::new()
            .line_number(false)
            .multi_line(true)
            .max_matches(Some(1))
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_matches_multi_line1(): got → {}", got);
		log::debug!("<<< Exiting max_matches_multi_line1()");
        let expected = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_matches_multi_line2() {
		log::debug!(">>> Entering max_matches_multi_line2()");
        let matcher =
            RegexMatcher::new(r"(?s)Watson.+?(Holmeses|clearly)").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
		log::debug!("max_matches_multi_line2(): printer → (unprintable)");
        SearcherBuilder::new()
            .line_number(false)
            .multi_line(true)
            .max_matches(Some(1))
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_matches_multi_line2(): got → {}", got);
		log::debug!("<<< Exiting max_matches_multi_line2()");
        let expected = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_matches_multi_line3() {
		log::debug!(">>> Entering max_matches_multi_line3()");
        let matcher = RegexMatcher::new(r"line 2\nline 3").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
		log::debug!("max_matches_multi_line3(): printer → (unprintable)");
        log::debug!("max_matches_multi_line3(): matcher → (unprintable)");
        SearcherBuilder::new()
            .line_number(false)
            .multi_line(true)
            .max_matches(Some(1))
            .build()
            .search_reader(
                &matcher,
                "line 2\nline 3 x\nline 2\nline 3\n".as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_matches_multi_line3(): got → {}", got);
		log::debug!("<<< Exiting max_matches_multi_line3()");
        let expected = "\
line 2
line 3 x
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn max_matches_multi_line4() {
		log::debug!(">>> Entering max_matches_multi_line4()");
        let matcher =
            RegexMatcher::new(r"line 2\nline 3|x\nline 2\n").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
		log::debug!("max_matches_multi_line4(): printer → (unprintable)");
        SearcherBuilder::new()
            .line_number(false)
            .multi_line(true)
            .max_matches(Some(1))
            .build()
            .search_reader(
                &matcher,
                "line 2\nline 3 x\nline 2\nline 3 x\n".as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("max_matches_multi_line4(): got → {}", got);
		log::debug!("<<< Exiting max_matches_multi_line4()");
        let expected = "\
line 2
line 3 x
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn only_matching() {
        let matcher = RegexMatcher::new("Doctor Watsons|Sherlock").unwrap();
        let mut printer = StandardBuilder::new()
            .only_matching(true)
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("only_matching(): got → {}", got);
        let expected = "\
1:9:Doctor Watsons
1:57:Sherlock
3:49:Sherlock
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn only_matching_multi_line1() {
		log::debug!(">>> Entering only_matching_multi_line1()");
        let matcher =
            RegexMatcher::new(r"(?s:.{0})(Doctor Watsons|Sherlock)").unwrap();
        let mut printer = StandardBuilder::new()
            .only_matching(true)
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("only_matching_multi_line1(): got → {}", got);
		log::debug!("<<< Exiting only_matching_multi_line1()");
        let expected = "\
1:9:Doctor Watsons
1:57:Sherlock
3:49:Sherlock
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn only_matching_multi_line2() {
		log::debug!(">>> Entering only_matching_multi_line2()");
        let matcher =
            RegexMatcher::new(r"(?s)Watson.+?(Holmeses|clearly)").unwrap();
        let mut printer = StandardBuilder::new()
            .only_matching(true)
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("only_matching_multi_line2(): got → {}", got);
		log::debug!("<<< Exiting only_matching_multi_line2()");
        let expected = "\
1:16:Watsons of this world, as opposed to the Sherlock
2:16:Holmeses
5:12:Watson has to have it taken out for him and dusted,
6:12:and exhibited clearly
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn only_matching_max_columns() {
		log::debug!(">>> Entering only_matching_max_columns()");
		log::debug!("only_matching_max_columns(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("Doctor Watsons|Sherlock").unwrap();
        let mut printer = StandardBuilder::new()
            .only_matching(true)
            .max_columns(Some(10))
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("only_matching_max_columns(): got → {}", got);
		log::debug!("<<< Exiting only_matching_max_columns()");
        let expected = "\
1:9:[Omitted long matching line]
1:57:Sherlock
3:49:Sherlock
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn only_matching_max_columns_preview() {
		log::debug!(">>> Entering only_matching_max_columns_preview()");
		log::debug!("only_matching_max_columns_preview(): matcher → (unprintable)");
        let matcher = RegexMatcher::new("Doctor Watsons|Sherlock").unwrap();
        let mut printer = StandardBuilder::new()
            .only_matching(true)
            .max_columns(Some(10))
            .max_columns_preview(true)
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("only_matching_max_columns_preview(): got → {}", got);
		log::debug!("<<< Exiting only_matching_max_columns_preview()");
        let expected = "\
1:9:Doctor Wat [... 0 more matches]
1:57:Sherlock
3:49:Sherlock
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn only_matching_max_columns_multi_line1() {
		log::debug!(">>> Entering only_matching_max_columns_multi_line1()");
        // The `(?s:.{0})` trick fools the matcher into thinking that it
        // can match across multiple lines without actually doing so. This is
        // so we can test multi-line handling in the case of a match on only
        // one line.
        let matcher =
            RegexMatcher::new(r"(?s:.{0})(Doctor Watsons|Sherlock)").unwrap();
        let mut printer = StandardBuilder::new()
            .only_matching(true)
            .max_columns(Some(10))
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("only_matching_max_columns_multi_line1(): got → {}", got);
		log::debug!("<<< Exiting only_matching_max_columns_multi_line1()");
        let expected = "\
1:9:[Omitted long matching line]
1:57:Sherlock
3:49:Sherlock
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn only_matching_max_columns_preview_multi_line1() {
		log::debug!(">>> Entering only_matching_max_columns_preview_multi_line1()");
        // The `(?s:.{0})` trick fools the matcher into thinking that it
        // can match across multiple lines without actually doing so. This is
        // so we can test multi-line handling in the case of a match on only
        // one line.
        let matcher =
            RegexMatcher::new(r"(?s:.{0})(Doctor Watsons|Sherlock)").unwrap();
        let mut printer = StandardBuilder::new()
            .only_matching(true)
            .max_columns(Some(10))
            .max_columns_preview(true)
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("only_matching_max_columns_preview_multi_line1(): got → {}", got);
		log::debug!("<<< Exiting only_matching_max_columns_preview_multi_line1()");
        let expected = "\
1:9:Doctor Wat [... 0 more matches]
1:57:Sherlock
3:49:Sherlock
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn only_matching_max_columns_multi_line2() {
		log::debug!(">>> Entering only_matching_max_columns_multi_line2()");
        let matcher =
            RegexMatcher::new(r"(?s)Watson.+?(Holmeses|clearly)").unwrap();
        let mut printer = StandardBuilder::new()
            .only_matching(true)
            .max_columns(Some(50))
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("only_matching_max_columns_multi_line2(): got → {}", got);
		log::debug!("<<< Exiting only_matching_max_columns_multi_line2()");
        let expected = "\
1:16:Watsons of this world, as opposed to the Sherlock
2:16:Holmeses
5:12:[Omitted long matching line]
6:12:and exhibited clearly
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn only_matching_max_columns_preview_multi_line2() {
		log::debug!(">>> Entering only_matching_max_columns_preview_multi_line2()");
        let matcher =
            RegexMatcher::new(r"(?s)Watson.+?(Holmeses|clearly)").unwrap();
        let mut printer = StandardBuilder::new()
            .only_matching(true)
            .max_columns(Some(50))
            .max_columns_preview(true)
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("only_matching_max_columns_preview_multi_line2(): got → {}", got);
		log::debug!("<<< Exiting only_matching_max_columns_preview_multi_line2()");
        let expected = "\
1:16:Watsons of this world, as opposed to the Sherlock
2:16:Holmeses
5:12:Watson has to have it taken out for him and dusted [... 0 more matches]
6:12:and exhibited clearly
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn per_match() {
        let matcher = RegexMatcher::new("Doctor Watsons|Sherlock").unwrap();
        let mut printer = StandardBuilder::new()
            .per_match(true)
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("per_match(): got → {}", got);
        let expected = "\
1:9:For the Doctor Watsons of this world, as opposed to the Sherlock
1:57:For the Doctor Watsons of this world, as opposed to the Sherlock
3:49:be, to a very large extent, the result of luck. Sherlock Holmes
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn per_match_multi_line1() {
		log::debug!(">>> Entering per_match_multi_line1()");
        let matcher =
            RegexMatcher::new(r"(?s:.{0})(Doctor Watsons|Sherlock)").unwrap();
        let mut printer = StandardBuilder::new()
            .per_match(true)
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("per_match_multi_line1(): got → {}", got);
		log::debug!("<<< Exiting per_match_multi_line1()");
        let expected = "\
1:9:For the Doctor Watsons of this world, as opposed to the Sherlock
1:57:For the Doctor Watsons of this world, as opposed to the Sherlock
3:49:be, to a very large extent, the result of luck. Sherlock Holmes
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn per_match_multi_line2() {
		log::debug!(">>> Entering per_match_multi_line2()");
        let matcher =
            RegexMatcher::new(r"(?s)Watson.+?(Holmeses|clearly)").unwrap();
        let mut printer = StandardBuilder::new()
            .per_match(true)
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("per_match_multi_line2(): got → {}", got);
		log::debug!("<<< Exiting per_match_multi_line2()");
        let expected = "\
1:16:For the Doctor Watsons of this world, as opposed to the Sherlock
2:1:Holmeses, success in the province of detective work must always
5:12:but Doctor Watson has to have it taken out for him and dusted,
6:1:and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn per_match_multi_line3() {
		log::debug!(">>> Entering per_match_multi_line3()");
        let matcher =
            RegexMatcher::new(r"(?s)Watson.+?Holmeses|always.+?be").unwrap();
        let mut printer = StandardBuilder::new()
            .per_match(true)
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("per_match_multi_line3(): got → {}", got);
		log::debug!("<<< Exiting per_match_multi_line3()");
        let expected = "\
1:16:For the Doctor Watsons of this world, as opposed to the Sherlock
2:1:Holmeses, success in the province of detective work must always
2:58:Holmeses, success in the province of detective work must always
3:1:be, to a very large extent, the result of luck. Sherlock Holmes
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn per_match_multi_line1_only_first_line() {
		log::debug!(">>> Entering per_match_multi_line1_only_first_line()");
        let matcher =
            RegexMatcher::new(r"(?s:.{0})(Doctor Watsons|Sherlock)").unwrap();
        let mut printer = StandardBuilder::new()
            .per_match(true)
            .per_match_one_line(true)
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("per_match_multi_line1_only_first_line(): got → {}", got);
		log::debug!("<<< Exiting per_match_multi_line1_only_first_line()");
        let expected = "\
1:9:For the Doctor Watsons of this world, as opposed to the Sherlock
1:57:For the Doctor Watsons of this world, as opposed to the Sherlock
3:49:be, to a very large extent, the result of luck. Sherlock Holmes
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn per_match_multi_line2_only_first_line() {
		log::debug!(">>> Entering per_match_multi_line2_only_first_line()");
        let matcher =
            RegexMatcher::new(r"(?s)Watson.+?(Holmeses|clearly)").unwrap();
        let mut printer = StandardBuilder::new()
            .per_match(true)
            .per_match_one_line(true)
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("per_match_multi_line2_only_first_line(): got → {}", got);
		log::debug!("<<< Exiting per_match_multi_line2_only_first_line()");
        let expected = "\
1:16:For the Doctor Watsons of this world, as opposed to the Sherlock
5:12:but Doctor Watson has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn per_match_multi_line3_only_first_line() {
		log::debug!(">>> Entering per_match_multi_line3_only_first_line()");
        let matcher =
            RegexMatcher::new(r"(?s)Watson.+?Holmeses|always.+?be").unwrap();
        let mut printer = StandardBuilder::new()
            .per_match(true)
            .per_match_one_line(true)
            .column(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("per_match_multi_line3_only_first_line(): got → {}", got);
		log::debug!("<<< Exiting per_match_multi_line3_only_first_line()");
        let expected = "\
1:16:For the Doctor Watsons of this world, as opposed to the Sherlock
2:58:Holmeses, success in the province of detective work must always
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn replacement_passthru() {
		log::debug!(">>> Entering replacement_passthru()");
		log::debug!("replacement_passthru(): matcher → (unprintable)");
        let matcher = RegexMatcher::new(r"Sherlock|Doctor (\w+)").unwrap();
        let mut printer = StandardBuilder::new()
            .replacement(Some(b"doctah $1 MD".to_vec()))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .passthru(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("replacement_passthru(): got → {}", got);
		log::debug!("<<< Exiting replacement_passthru()");
        let expected = "\
1:For the doctah Watsons MD of this world, as opposed to the doctah  MD
2-Holmeses, success in the province of detective work must always
3:be, to a very large extent, the result of luck. doctah  MD Holmes
4-can extract a clew from a wisp of straw or a flake of cigar ash;
5:but doctah Watson MD has to have it taken out for him and dusted,
6-and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn replacement() {
		log::debug!(">>> Entering replacement()");
		log::debug!("replacement(): matcher → (unprintable)");
        let matcher = RegexMatcher::new(r"Sherlock|Doctor (\w+)").unwrap();
        let mut printer = StandardBuilder::new()
            .replacement(Some(b"doctah $1 MD".to_vec()))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("replacement(): got → {}", got);
		log::debug!("<<< Exiting replacement(replacement={:?})", replacement);
        let expected = "\
1:For the doctah Watsons MD of this world, as opposed to the doctah  MD
3:be, to a very large extent, the result of luck. doctah  MD Holmes
5:but doctah Watson MD has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    // This is a somewhat weird test that checks the behavior of attempting
    // to replace a line terminator with something else.
    //
    // See: https://github.com/BurntSushi/ripgrep/issues/1311
    #[test]
    fn replacement_multi_line() {
		log::debug!(">>> Entering replacement_multi_line()");
		log::debug!("replacement_multi_line(): matcher → (unprintable)");
        let matcher = RegexMatcher::new(r"\n").unwrap();
        let mut printer = StandardBuilder::new()
            .replacement(Some(b"?".to_vec()))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .multi_line(true)
            .build()
            .search_reader(
                &matcher,
                "hello\nworld\n".as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("replacement_multi_line(): got → {}", got);
        let expected = "1:hello?world?\n";
		log::debug!("replacement_multi_line(): expected → {}", expected);
		log::debug!("<<< Exiting replacement_multi_line()");
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn replacement_multi_line_diff_line_term() {
		log::debug!(">>> Entering replacement_multi_line_diff_line_term()");
        let matcher = RegexMatcherBuilder::new()
            .line_terminator(Some(b'\x00'))
            .build(r"\n")
            .unwrap();
        let mut printer = StandardBuilder::new()
            .replacement(Some(b"?".to_vec()))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_terminator(LineTerminator::byte(b'\x00'))
            .line_number(true)
            .multi_line(true)
            .build()
            .search_reader(
                &matcher,
                "hello\nworld\n".as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("replacement_multi_line_diff_line_term(): got → {}", got);
        let expected = "1:hello?world?\x00";
		log::debug!("replacement_multi_line_diff_line_term(): expected → {}", expected);
		log::debug!("<<< Exiting replacement_multi_line_diff_line_term()");
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn replacement_multi_line_combine_lines() {
		log::debug!(">>> Entering replacement_multi_line_combine_lines()");
		log::debug!("replacement_multi_line_combine_lines(): matcher → (unprintable)");
        let matcher = RegexMatcher::new(r"\n(.)?").unwrap();
        let mut printer = StandardBuilder::new()
            .replacement(Some(b"?$1".to_vec()))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .multi_line(true)
            .build()
            .search_reader(
                &matcher,
                "hello\nworld\n".as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("replacement_multi_line_combine_lines(): got → {}", got);
        let expected = "1:hello?world?\n";
		log::debug!("replacement_multi_line_combine_lines(): expected → {}", expected);
		log::debug!("<<< Exiting replacement_multi_line_combine_lines()");
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn replacement_max_columns() {
		log::debug!(">>> Entering replacement_max_columns()");
		log::debug!("replacement_max_columns(): matcher → (unprintable)");
        let matcher = RegexMatcher::new(r"Sherlock|Doctor (\w+)").unwrap();
        let mut printer = StandardBuilder::new()
            .max_columns(Some(67))
            .replacement(Some(b"doctah $1 MD".to_vec()))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("replacement_max_columns(): got → {}", got);
		log::debug!("<<< Exiting replacement_max_columns()");
        let expected = "\
1:[Omitted long line with 2 matches]
3:be, to a very large extent, the result of luck. doctah  MD Holmes
5:but doctah Watson MD has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn replacement_max_columns_preview1() {
		log::debug!(">>> Entering replacement_max_columns_preview1()");
		log::debug!("replacement_max_columns_preview1(): matcher → (unprintable)");
        let matcher = RegexMatcher::new(r"Sherlock|Doctor (\w+)").unwrap();
        let mut printer = StandardBuilder::new()
            .max_columns(Some(67))
            .max_columns_preview(true)
            .replacement(Some(b"doctah $1 MD".to_vec()))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("replacement_max_columns_preview1(): got → {}", got);
		log::debug!("<<< Exiting replacement_max_columns_preview1()");
        let expected = "\
1:For the doctah Watsons MD of this world, as opposed to the doctah   [... 0 more matches]
3:be, to a very large extent, the result of luck. doctah  MD Holmes
5:but doctah Watson MD has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn replacement_max_columns_preview2() {
		log::debug!(">>> Entering replacement_max_columns_preview2()");
        let matcher =
            RegexMatcher::new("exhibited|dusted|has to have it").unwrap();
        let mut printer = StandardBuilder::new()
            .max_columns(Some(43))
            .max_columns_preview(true)
            .replacement(Some(b"xxx".to_vec()))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(false)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("replacement_max_columns_preview2(): got → {}", got);
		log::debug!("<<< Exiting replacement_max_columns_preview2()");
        let expected = "\
but Doctor Watson xxx taken out for him and [... 1 more match]
and xxx clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn replacement_only_matching() {
		log::debug!(">>> Entering replacement_only_matching()");
		log::debug!("replacement_only_matching(): matcher → (unprintable)");
        let matcher = RegexMatcher::new(r"Sherlock|Doctor (\w+)").unwrap();
        let mut printer = StandardBuilder::new()
            .only_matching(true)
            .replacement(Some(b"doctah $1 MD".to_vec()))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("replacement_only_matching(): got → {}", got);
		log::debug!("<<< Exiting replacement_only_matching()");
        let expected = "\
1:doctah Watsons MD
1:doctah  MD
3:doctah  MD
5:doctah Watson MD
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn replacement_per_match() {
		log::debug!(">>> Entering replacement_per_match()");
		log::debug!("replacement_per_match(): matcher → (unprintable)");
        let matcher = RegexMatcher::new(r"Sherlock|Doctor (\w+)").unwrap();
        let mut printer = StandardBuilder::new()
            .per_match(true)
            .replacement(Some(b"doctah $1 MD".to_vec()))
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("replacement_per_match(): got → {}", got);
		log::debug!("<<< Exiting replacement_per_match()");
        let expected = "\
1:For the doctah Watsons MD of this world, as opposed to the doctah  MD
1:For the doctah Watsons MD of this world, as opposed to the doctah  MD
3:be, to a very large extent, the result of luck. doctah  MD Holmes
5:but doctah Watson MD has to have it taken out for him and dusted,
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn invert() {
		log::debug!(">>> Entering invert()");
        let matcher = RegexMatcher::new(r"Sherlock").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
		log::debug!("invert(): printer → (unprintable)");
        log::debug!("invert(): matcher → (unprintable)");
        SearcherBuilder::new()
            .line_number(true)
            .invert_match(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("invert(): got → {}", got);
		log::debug!("<<< Exiting invert()");
        let expected = "\
2:Holmeses, success in the province of detective work must always
4:can extract a clew from a wisp of straw or a flake of cigar ash;
5:but Doctor Watson has to have it taken out for him and dusted,
6:and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn invert_multi_line() {
		log::debug!(">>> Entering invert_multi_line()");
        let matcher = RegexMatcher::new(r"(?s:.{0})Sherlock").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
		log::debug!("invert_multi_line(): printer → (unprintable)");
        log::debug!("invert_multi_line(): matcher → (unprintable)");
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .invert_match(true)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("invert_multi_line(): got → {}", got);
		log::debug!("<<< Exiting invert_multi_line()");
        let expected = "\
2:Holmeses, success in the province of detective work must always
4:can extract a clew from a wisp of straw or a flake of cigar ash;
5:but Doctor Watson has to have it taken out for him and dusted,
6:and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn invert_context() {
		log::debug!(">>> Entering invert_context()");
        let matcher = RegexMatcher::new(r"Sherlock").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
		log::debug!("invert_context(): printer → (unprintable)");
        log::debug!("invert_context(): matcher → (unprintable)");
        SearcherBuilder::new()
            .line_number(true)
            .invert_match(true)
            .before_context(1)
            .after_context(1)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("invert_context(): got → {}", got);
		log::debug!("<<< Exiting invert_context()");
        let expected = "\
1-For the Doctor Watsons of this world, as opposed to the Sherlock
2:Holmeses, success in the province of detective work must always
3-be, to a very large extent, the result of luck. Sherlock Holmes
4:can extract a clew from a wisp of straw or a flake of cigar ash;
5:but Doctor Watson has to have it taken out for him and dusted,
6:and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn invert_context_multi_line() {
		log::debug!(">>> Entering invert_context_multi_line()");
        let matcher = RegexMatcher::new(r"(?s:.{0})Sherlock").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
		log::debug!("invert_context_multi_line(): printer → (unprintable)");
        log::debug!("invert_context_multi_line(): matcher → (unprintable)");
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .invert_match(true)
            .before_context(1)
            .after_context(1)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("invert_context_multi_line(): got → {}", got);
		log::debug!("<<< Exiting invert_context_multi_line()");
        let expected = "\
1-For the Doctor Watsons of this world, as opposed to the Sherlock
2:Holmeses, success in the province of detective work must always
3-be, to a very large extent, the result of luck. Sherlock Holmes
4:can extract a clew from a wisp of straw or a flake of cigar ash;
5:but Doctor Watson has to have it taken out for him and dusted,
6:and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn invert_context_only_matching() {
		log::debug!(">>> Entering invert_context_only_matching()");
		log::debug!("invert_context_only_matching(): matcher → (unprintable)");
        let matcher = RegexMatcher::new(r"Sherlock").unwrap();
        let mut printer = StandardBuilder::new()
            .only_matching(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .line_number(true)
            .invert_match(true)
            .before_context(1)
            .after_context(1)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("invert_context_only_matching(): got → {}", got);
		log::debug!("<<< Exiting invert_context_only_matching()");
        let expected = "\
1-Sherlock
2:Holmeses, success in the province of detective work must always
3-Sherlock
4:can extract a clew from a wisp of straw or a flake of cigar ash;
5:but Doctor Watson has to have it taken out for him and dusted,
6:and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn invert_context_only_matching_multi_line() {
		log::debug!(">>> Entering invert_context_only_matching_multi_line()");
		log::debug!("invert_context_only_matching_multi_line(): matcher → (unprintable)");
        let matcher = RegexMatcher::new(r"(?s:.{0})Sherlock").unwrap();
        let mut printer = StandardBuilder::new()
            .only_matching(true)
            .build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .multi_line(true)
            .line_number(true)
            .invert_match(true)
            .before_context(1)
            .after_context(1)
            .build()
            .search_reader(
                &matcher,
                SHERLOCK.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("invert_context_only_matching_multi_line(): got → {}", got);
		log::debug!("<<< Exiting invert_context_only_matching_multi_line()");
        let expected = "\
1-Sherlock
2:Holmeses, success in the province of detective work must always
3-Sherlock
4:can extract a clew from a wisp of straw or a flake of cigar ash;
5:but Doctor Watson has to have it taken out for him and dusted,
6:and exhibited clearly, with a label attached.
";
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn regression_search_empty_with_crlf() {
		log::debug!(">>> Entering regression_search_empty_with_crlf()");
        let matcher =
            RegexMatcherBuilder::new().crlf(true).build(r"x?").unwrap();
        let mut printer = StandardBuilder::new()
            .color_specs(ColorSpecs::default_with_color())
            .build(Ansi::new(vec![]));
        SearcherBuilder::new()
            .line_terminator(LineTerminator::crlf())
            .build()
            .search_reader(&matcher, &b"\n"[..], printer.sink(&matcher))
            .unwrap();

        let got = printer_contents_ansi(&mut printer);
		log::debug!("regression_search_empty_with_crlf(): got → {}", got);
		log::debug!("<<< Exiting regression_search_empty_with_crlf()");
        assert!(!got.is_empty());
    }

    #[test]
    fn regression_after_context_with_match() {
		log::debug!(">>> Entering regression_after_context_with_match()");
		log::debug!("<<< Exiting regression_after_context_with_match()");
        let haystack = "\
a
b
c
d
e
d
e
d
e
d
e
";

        let matcher = RegexMatcherBuilder::new().build(r"d").unwrap();
		log::debug!("regression_after_context_with_match(): matcher → (unprintable)");
		log::debug!("regression_after_context_with_match(): printer → (unprintable)");
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
        SearcherBuilder::new()
            .max_matches(Some(1))
            .line_number(true)
            .after_context(2)
            .build()
            .search_reader(
                &matcher,
                haystack.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();

        let got = printer_contents(&mut printer);
		log::debug!("regression_after_context_with_match(): got → {}", got);
        let expected = "4:d\n5-e\n6:d\n";
		log::debug!("regression_after_context_with_match(): expected → {}", expected);
        assert_eq_printed!(expected, got);
    }

    #[test]
    fn regression_crlf_preserve() {
		log::debug!(">>> Entering regression_crlf_preserve()");
        let haystack = "hello\nworld\r\n";
		log::debug!("regression_crlf_preserve(): haystack → {}", haystack);
        let matcher =
            RegexMatcherBuilder::new().crlf(true).build(r".").unwrap();
        let mut printer = StandardBuilder::new().build(NoColor::new(vec![]));
		log::debug!("regression_crlf_preserve(): printer → (unprintable)");
        let mut searcher = SearcherBuilder::new()
            .line_number(false)
            .line_terminator(LineTerminator::crlf())
            .build();

        searcher
            .search_reader(
                &matcher,
                haystack.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();
        let got = printer_contents(&mut printer);
		log::debug!("regression_crlf_preserve(): got → {}", got);
        let expected = "hello\nworld\r\n";
		log::debug!("regression_crlf_preserve(): expected → {}", expected);
        assert_eq_printed!(expected, got);

        let mut printer = StandardBuilder::new()
            .replacement(Some(b"$0".to_vec()))
            .build(NoColor::new(vec![]));
        searcher
            .search_reader(
                &matcher,
                haystack.as_bytes(),
                printer.sink(&matcher),
            )
            .unwrap();
        let got = printer_contents(&mut printer);
        let expected = "hello\nworld\r\n";
		log::debug!("<<< Exiting regression_crlf_preserve()");
        assert_eq_printed!(expected, got);
    }
}
