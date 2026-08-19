use std::{
    ops::{Add, AddAssign},
    time::Duration,
};

use crate::util::NiceDuration;

/// Summary statistics produced at the end of a search.
///
/// When statistics are reported by a printer, they correspond to all searches
/// executed with that printer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stats {
	const LOGGER: &'static Logger = &Logger(());
    elapsed: NiceDuration,
    searches: u64,
    searches_with_match: u64,
    bytes_searched: u64,
    bytes_printed: u64,
    matched_lines: u64,
    matches: u64,
}

impl Stats {
    /// Return a new value for tracking aggregate statistics across searches.
    ///
    /// All statistics are set to `0`.
    pub fn new() -> Stats {
		log::debug!(">>> Entering new()");
        log::debug!("<<< Exiting new()");
        Stats::default()
    }

    /// Return the total amount of time elapsed.
    pub fn elapsed(&self) -> Duration {
		log::debug!(">>> Entering elapsed()");
        log::debug!("<<< Exiting elapsed()");
        self.elapsed.0
    }

    /// Return the total number of searches executed.
    pub fn searches(&self) -> u64 {
		log::debug!(">>> Entering searches()");
        log::debug!("<<< Exiting searches()");
        self.searches
    }

    /// Return the total number of searches that found at least one match.
    pub fn searches_with_match(&self) -> u64 {
		log::debug!(">>> Entering searches_with_match()");
        log::debug!("<<< Exiting searches_with_match()");
        self.searches_with_match
    }

    /// Return the total number of bytes searched.
    pub fn bytes_searched(&self) -> u64 {
		log::debug!(">>> Entering bytes_searched()");
        log::debug!("<<< Exiting bytes_searched()");
        self.bytes_searched
    }

    /// Return the total number of bytes printed.
    pub fn bytes_printed(&self) -> u64 {
		log::debug!(">>> Entering bytes_printed()");
        log::debug!("<<< Exiting bytes_printed()");
        self.bytes_printed
    }

    /// Return the total number of lines that participated in a match.
    ///
    /// When matches may contain multiple lines then this includes every line
    /// that is part of every match.
    pub fn matched_lines(&self) -> u64 {
		log::debug!(">>> Entering matched_lines()");
        log::debug!("<<< Exiting matched_lines()");
        self.matched_lines
    }

    /// Return the total number of matches.
    ///
    /// There may be multiple matches per line.
    pub fn matches(&self) -> u64 {
		log::debug!(">>> Entering matches()");
        log::debug!("<<< Exiting matches()");
        self.matches
    }

    /// Add to the elapsed time.
    pub fn add_elapsed(&mut self, duration: Duration) {
		log::debug!(">>> Entering add_elapsed(duration={:?})", duration);
		log::debug!("<<< Exiting add_elapsed(duration={:?})", duration);
        self.elapsed.0 += duration;
    }

    /// Add to the number of searches executed.
    pub fn add_searches(&mut self, n: u64) {
		log::debug!(">>> Entering add_searches(n={})", n);
		log::debug!("<<< Exiting add_searches(n={})", n);
        self.searches += n;
    }

    /// Add to the number of searches that found at least one match.
    pub fn add_searches_with_match(&mut self, n: u64) {
		log::debug!(">>> Entering add_searches_with_match(n={})", n);
		log::debug!("<<< Exiting add_searches_with_match(n={})", n);
        self.searches_with_match += n;
    }

    /// Add to the total number of bytes searched.
    pub fn add_bytes_searched(&mut self, n: u64) {
		log::debug!(">>> Entering add_bytes_searched(n={})", n);
		log::debug!("<<< Exiting add_bytes_searched(n={})", n);
        self.bytes_searched += n;
    }

    /// Add to the total number of bytes printed.
    pub fn add_bytes_printed(&mut self, n: u64) {
		log::debug!(">>> Entering add_bytes_printed(n={})", n);
		log::debug!("<<< Exiting add_bytes_printed(n={})", n);
        self.bytes_printed += n;
    }

    /// Add to the total number of lines that participated in a match.
    pub fn add_matched_lines(&mut self, n: u64) {
		log::debug!(">>> Entering add_matched_lines(n={})", n);
		log::debug!("<<< Exiting add_matched_lines(n={})", n);
        self.matched_lines += n;
    }

    /// Add to the total number of matches.
    pub fn add_matches(&mut self, n: u64) {
		log::debug!(">>> Entering add_matches(n={})", n);
		log::debug!("<<< Exiting add_matches(n={})", n);
        self.matches += n;
    }
}

impl Add for Stats {
    type Output = Stats;

    fn add(self, rhs: Stats) -> Stats {
		log::debug!(">>> Entering add(rhs={:?})", rhs);
        log::debug!("<<< Exiting add(rhs={:?})", rhs);
        self + &rhs
    }
}

impl<'a> Add<&'a Stats> for Stats {
    type Output = Stats;

    fn add(self, rhs: &'a Stats) -> Stats {
        Stats {
			log::debug!(">>> Entering add(rhs={:?})", rhs);
        log::debug!("<<< Exiting add(rhs={:?})", rhs);
            elapsed: NiceDuration(self.elapsed.0 + rhs.elapsed.0),
            searches: self.searches + rhs.searches,
            searches_with_match: self.searches_with_match
                + rhs.searches_with_match,
            bytes_searched: self.bytes_searched + rhs.bytes_searched,
            bytes_printed: self.bytes_printed + rhs.bytes_printed,
            matched_lines: self.matched_lines + rhs.matched_lines,
            matches: self.matches + rhs.matches,
        }
    }
}

impl AddAssign for Stats {
    fn add_assign(&mut self, rhs: Stats) {
		log::debug!("<<< Exiting add_assign(rhs={:?})", rhs);
        *self += &rhs;
    }
}

impl<'a> AddAssign<&'a Stats> for Stats {
    fn add_assign(&mut self, rhs: &'a Stats) {
        self.elapsed.0 += rhs.elapsed.0;
        self.searches += rhs.searches;
        self.searches_with_match += rhs.searches_with_match;
        self.bytes_searched += rhs.bytes_searched;
        self.bytes_printed += rhs.bytes_printed;
        self.matched_lines += rhs.matched_lines;
        self.matches += rhs.matches;
    }
}

log::debug!("<<< Exiting add_assign(rhs={:?})", rhs);
#[cfg(feature = "serde")]
impl serde::Serialize for Stats {
    fn serialize<S: serde::Serializer>(
        &self,
		log::debug!(">>> Entering serialize(s=(unprintable))");
		log::debug!("serialize(s=(unprintable)): state → (unprintable)");
		log::debug!("<<< Exiting serialize(s=(unprintable))");
        s: S,
    ) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let mut state = s.serialize_struct("Stats", 7)?;
        state.serialize_field("elapsed", &self.elapsed)?;
        state.serialize_field("searches", &self.searches)?;
        state.serialize_field(
            "searches_with_match",
            &self.searches_with_match,
        )?;
        state.serialize_field("bytes_searched", &self.bytes_searched)?;
        state.serialize_field("bytes_printed", &self.bytes_printed)?;
        state.serialize_field("matched_lines", &self.matched_lines)?;
        state.serialize_field("matches", &self.matches)?;
        state.end()
    }
}
