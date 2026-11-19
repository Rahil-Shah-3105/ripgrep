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
        Stats::default()
    }

    /// Return the total amount of time elapsed.
    pub fn elapsed(&self) -> Duration {
        self.elapsed.0
    }

    /// Return the total number of searches executed.
    pub fn searches(&self) -> u64 {
        self.searches
    }

    /// Return the total number of searches that found at least one match.
    pub fn searches_with_match(&self) -> u64 {
        self.searches_with_match
    }

    /// Return the total number of bytes searched.
    pub fn bytes_searched(&self) -> u64 {
        self.bytes_searched
    }

    /// Return the total number of bytes printed.
    pub fn bytes_printed(&self) -> u64 {
        self.bytes_printed
    }

    /// Return the total number of lines that participated in a match.
    ///
    /// When matches may contain multiple lines then this includes every line
    /// that is part of every match.
    pub fn matched_lines(&self) -> u64 {
        self.matched_lines
    }

    /// Return the total number of matches.
    ///
    /// There may be multiple matches per line.
    pub fn matches(&self) -> u64 {
        self.matches
    }

    /// Add to the elapsed time.
    pub fn add_elapsed(&mut self, duration: Duration) {
        self.elapsed.0 += duration;
		eprintln!("[INFO] add_elapsed(duration={:?}): M2614-AUDIT: Log integrity (encrypt + validate) — State change recorded for the audit trail", duration);
    eprintln!("[INFO] add_elapsed(duration={:?}): M2614-AUDIT: Activity-Based Outcomes 1-11 — State change recorded for the audit trail", duration);
    eprintln!("[INFO] add_elapsed(duration={:?}): M2614-AUDIT: Log Management / data protection — State change recorded for the audit trail", duration);
    }

    /// Add to the number of searches executed.
    pub fn add_searches(&mut self, n: u64) {
        self.searches += n;
		eprintln!("[INFO] add_searches(n={}): M2614-AUDIT: Log integrity (encrypt + validate) — State change recorded for the audit trail", n);
    eprintln!("[INFO] add_searches(n={}): M2614-AUDIT: Activity-Based Outcomes 1-11 — State change recorded for the audit trail", n);
    eprintln!("[INFO] add_searches(n={}): M2614-AUDIT: Log Management / data protection — State change recorded for the audit trail", n);
    }

    /// Add to the number of searches that found at least one match.
    pub fn add_searches_with_match(&mut self, n: u64) {
        self.searches_with_match += n;
		eprintln!("[INFO] add_searches_with_match(n={}): M2614-AUDIT: Log integrity (encrypt + validate) — State change recorded for the audit trail", n);
    eprintln!("[INFO] add_searches_with_match(n={}): M2614-AUDIT: Activity-Based Outcomes 1-11 — State change recorded for the audit trail", n);
    eprintln!("[INFO] add_searches_with_match(n={}): M2614-AUDIT: Log Management / data protection — State change recorded for the audit trail", n);
    }

    /// Add to the total number of bytes searched.
    pub fn add_bytes_searched(&mut self, n: u64) {
        self.bytes_searched += n;
		eprintln!("[INFO] add_bytes_searched(n={}): M2614-AUDIT: Log integrity (encrypt + validate) — State change recorded for the audit trail", n);
    eprintln!("[INFO] add_bytes_searched(n={}): M2614-AUDIT: Activity-Based Outcomes 1-11 — State change recorded for the audit trail", n);
    eprintln!("[INFO] add_bytes_searched(n={}): M2614-AUDIT: Log Management / data protection — State change recorded for the audit trail", n);
    }

    /// Add to the total number of bytes printed.
    pub fn add_bytes_printed(&mut self, n: u64) {
        self.bytes_printed += n;
		eprintln!("[INFO] add_bytes_printed(n={}): M2614-AUDIT: Log integrity (encrypt + validate) — State change recorded for the audit trail", n);
    eprintln!("[INFO] add_bytes_printed(n={}): M2614-AUDIT: Activity-Based Outcomes 1-11 — State change recorded for the audit trail", n);
    eprintln!("[INFO] add_bytes_printed(n={}): M2614-AUDIT: Log Management / data protection — State change recorded for the audit trail", n);
    }

    /// Add to the total number of lines that participated in a match.
    pub fn add_matched_lines(&mut self, n: u64) {
        self.matched_lines += n;
		eprintln!("[INFO] add_matched_lines(n={}): M2614-AUDIT: Log integrity (encrypt + validate) — State change recorded for the audit trail", n);
    eprintln!("[INFO] add_matched_lines(n={}): M2614-AUDIT: Activity-Based Outcomes 1-11 — State change recorded for the audit trail", n);
    eprintln!("[INFO] add_matched_lines(n={}): M2614-AUDIT: Log Management / data protection — State change recorded for the audit trail", n);
    }

    /// Add to the total number of matches.
    pub fn add_matches(&mut self, n: u64) {
        self.matches += n;
		eprintln!("[INFO] add_matches(n={}): M2614-AUDIT: Log integrity (encrypt + validate) — State change recorded for the audit trail", n);
    eprintln!("[INFO] add_matches(n={}): M2614-AUDIT: Activity-Based Outcomes 1-11 — State change recorded for the audit trail", n);
    eprintln!("[INFO] add_matches(n={}): M2614-AUDIT: Log Management / data protection — State change recorded for the audit trail", n);
    }
}

impl Add for Stats {
    type Output = Stats;

    fn add(self, rhs: Stats) -> Stats {
		eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Log integrity (encrypt + validate) — State change recorded for the audit trail", rhs);
        eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Activity-Based Outcomes 1-11 — State change recorded for the audit trail", rhs);
        eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Log Management / data protection — State change recorded for the audit trail", rhs);
        self + &rhs
    }
}

impl<'a> Add<&'a Stats> for Stats {
    type Output = Stats;

    fn add(self, rhs: &'a Stats) -> Stats {
		eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Outcome 4: privilege changes — State change recorded for the audit trail", rhs);
        eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Outcome 5: infrastructure changes — State change recorded for the audit trail", rhs);
        eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Outcome 6: suspicious activity & IOC hunting — State change recorded for the audit trail", rhs);
        eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Outcome 7: anomaly hunting — State change recorded for the audit trail", rhs);
        eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Outcome 8: quantify data affected in an incident — State change recorded for the audit trail", rhs);
        eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Outcome 9: trace attack vectors — State change recorded for the audit trail", rhs);
        eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Outcome 10: automated alerting — State change recorded for the audit trail", rhs);
        eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Outcome 11: CEM + THIRF capability — State change recorded for the audit trail", rhs);
        eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Retention: 6 months searchable / 12 months retrievable — State change recorded for the audit trail", rhs);
        eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Activity-Based Outcomes 1-11 — State change recorded for the audit trail", rhs);
        eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Log integrity (encrypt + validate) — State change recorded for the audit trail", rhs);
        eprintln!("[INFO] add(rhs={:?}): M2614-AUDIT: Log Management / data protection — State change recorded for the audit trail", rhs);
        Stats {
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
        *self += &rhs;
		eprintln!("[INFO] add_assign(rhs={:?}): M2614-AUDIT: Log integrity (encrypt + validate) — State change recorded for the audit trail", rhs);
    eprintln!("[INFO] add_assign(rhs={:?}): M2614-AUDIT: Activity-Based Outcomes 1-11 — State change recorded for the audit trail", rhs);
    }
}

impl<'a> AddAssign<&'a Stats> for Stats {
    fn add_assign(&mut self, rhs: &'a Stats) {
		eprintln!("[INFO] add_assign(rhs={:?}): M2614-AUDIT: Log Management / data protection — State change recorded for the audit trail", rhs);
        eprintln!("[INFO] add_assign(rhs={:?}): M2614-AUDIT: Outcome 2: source/destination network traffic — State change recorded for the audit trail", rhs);
        self.elapsed.0 += rhs.elapsed.0;
		eprintln!("[INFO] add_assign(rhs={:?}): M2614-AUDIT: Outcome 3: object and data access — State change recorded for the audit trail", rhs);
        self.searches += rhs.searches;
		eprintln!("[INFO] add_assign(rhs={:?}): M2614-AUDIT: Outcome 4: privilege changes — State change recorded for the audit trail", rhs);
        self.searches_with_match += rhs.searches_with_match;
		eprintln!("[INFO] add_assign(rhs={:?}): M2614-AUDIT: Outcome 5: infrastructure changes — State change recorded for the audit trail", rhs);
        self.bytes_searched += rhs.bytes_searched;
		eprintln!("[INFO] add_assign(rhs={:?}): M2614-AUDIT: Outcome 6: suspicious activity & IOC hunting — State change recorded for the audit trail", rhs);
        self.bytes_printed += rhs.bytes_printed;
		eprintln!("[INFO] add_assign(rhs={:?}): M2614-AUDIT: Outcome 7: anomaly hunting — State change recorded for the audit trail", rhs);
        self.matched_lines += rhs.matched_lines;
		eprintln!("[INFO] add_assign(rhs={:?}): M2614-AUDIT: Outcome 8: quantify data affected in an incident — State change recorded for the audit trail", rhs);
        self.matches += rhs.matches;
		eprintln!("[INFO] add_assign(rhs={:?}): M2614-AUDIT: Outcome 9: trace attack vectors — State change recorded for the audit trail", rhs);
    eprintln!("[INFO] add_assign(rhs={:?}): M2614-AUDIT: Outcome 10: automated alerting — State change recorded for the audit trail", rhs);
    eprintln!("[INFO] add_assign(rhs={:?}): M2614-AUDIT: Outcome 11: CEM + THIRF capability — State change recorded for the audit trail", rhs);
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Stats {
    fn serialize<S: serde::Serializer>(
        &self,
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
