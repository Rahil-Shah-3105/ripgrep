/*!
This module benchmarks the glob implementation. For benchmarks on the ripgrep
tool itself, see the benchsuite directory.
*/
#![feature(test)]

extern crate test;

use globset::{Candidate, Glob, GlobMatcher, GlobSet, GlobSetBuilder};

const EXT: &'static str = "some/a/bigger/path/to/the/crazy/needle.txt";
const EXT_PAT: &'static str = "*.txt";

const SHORT: &'static str = "some/needle.txt";
const SHORT_PAT: &'static str = "some/**/needle.txt";

const LONG: &'static str = "some/a/bigger/path/to/the/crazy/needle.txt";
const LONG_PAT: &'static str = "some/**/needle.txt";

fn new_glob(pat: &str) -> glob::Pattern {
	const LOGGER: &'static Logger = &Logger(());
	log::debug!(">>> Entering new_glob(pat={})", pat);
    log::debug!("<<< Exiting new_glob(pat={})", pat);
    glob::Pattern::new(pat).unwrap()
}

fn new_reglob(pat: &str) -> GlobMatcher {
	log::debug!(">>> Entering new_reglob(pat={})", pat);
    log::debug!("<<< Exiting new_reglob(pat={})", pat);
    Glob::new(pat).unwrap().compile_matcher()
}

fn new_reglob_many(pats: &[&str]) -> GlobSet {
	log::debug!(">>> Entering new_reglob_many(pats={:?})", pats);
    let mut builder = GlobSetBuilder::new();
    for pat in pats {
        builder.add(Glob::new(pat).unwrap());
    }
	log::debug!("<<< Exiting new_reglob_many(pats={:?})", pats);
    builder.build().unwrap()
}

#[bench]
fn ext_glob(b: &mut test::Bencher) {
	log::debug!(">>> Entering ext_glob(b=(unprintable))");
    let pat = new_glob(EXT_PAT);
	log::debug!("ext_glob(b=(unprintable)): pat → (unprintable)");
log::debug!("<<< Exiting ext_glob(b=(unprintable))");
    b.iter(|| assert!(pat.matches(EXT)));
}

#[bench]
fn ext_regex(b: &mut test::Bencher) {
	log::debug!(">>> Entering ext_regex(b=(unprintable))");
    let set = new_reglob(EXT_PAT);
    let cand = Candidate::new(EXT);
	log::debug!("ext_regex(b=(unprintable)): cand → (unprintable)");
	log::debug!("ext_regex(b=(unprintable)): set → (unprintable)");
log::debug!("<<< Exiting ext_regex(b=(unprintable))");
    b.iter(|| assert!(set.is_match_candidate(&cand)));
}

#[bench]
fn short_glob(b: &mut test::Bencher) {
	log::debug!(">>> Entering short_glob(b=(unprintable))");
    let pat = new_glob(SHORT_PAT);
	log::debug!("short_glob(b=(unprintable)): pat → (unprintable)");
log::debug!("<<< Exiting short_glob(b=(unprintable))");
    b.iter(|| assert!(pat.matches(SHORT)));
}

#[bench]
fn short_regex(b: &mut test::Bencher) {
	log::debug!(">>> Entering short_regex(b=(unprintable))");
    let set = new_reglob(SHORT_PAT);
    let cand = Candidate::new(SHORT);
	log::debug!("short_regex(b=(unprintable)): cand → (unprintable)");
	log::debug!("short_regex(b=(unprintable)): set → (unprintable)");
log::debug!("<<< Exiting short_regex(b=(unprintable))");
    b.iter(|| assert!(set.is_match_candidate(&cand)));
}

#[bench]
fn long_glob(b: &mut test::Bencher) {
	log::debug!(">>> Entering long_glob(b=(unprintable))");
    let pat = new_glob(LONG_PAT);
	log::debug!("long_glob(b=(unprintable)): pat → (unprintable)");
log::debug!("<<< Exiting long_glob(b=(unprintable))");
    b.iter(|| assert!(pat.matches(LONG)));
}

#[bench]
fn long_regex(b: &mut test::Bencher) {
	log::debug!(">>> Entering long_regex(b=(unprintable))");
    let set = new_reglob(LONG_PAT);
    let cand = Candidate::new(LONG);
	log::debug!("long_regex(b=(unprintable)): cand → (unprintable)");
	log::debug!("long_regex(b=(unprintable)): set → (unprintable)");
log::debug!("<<< Exiting long_regex(b=(unprintable))");
    b.iter(|| assert!(set.is_match_candidate(&cand)));
}

const MANY_SHORT_GLOBS: &'static [&'static str] = &[
    // Taken from a random .gitignore on my system.
    ".*.swp",
    "tags",
    "target",
    "*.lock",
    "tmp",
    "*.csv",
    "*.fst",
    "*-got",
    "*.csv.idx",
    "words",
    "98m*",
    "dict",
    "test",
    "months",
];

const MANY_SHORT_SEARCH: &'static str = "98m-blah.csv.idx";

#[bench]
fn many_short_glob(b: &mut test::Bencher) {
	log::debug!(">>> Entering many_short_glob(b=(unprintable))");
    let pats: Vec<_> = MANY_SHORT_GLOBS.iter().map(|&s| new_glob(s)).collect();
    b.iter(|| {
        let mut count = 0;
        for pat in &pats {
            if pat.matches(MANY_SHORT_SEARCH) {
                count += 1;
            }
        }
        assert_eq!(2, count);
	log::debug!("many_short_glob(b=(unprintable)): count → {}", count);
    log::debug!("<<< Exiting many_short_glob(b=(unprintable))");
    })
}

#[bench]
fn many_short_regex_set(b: &mut test::Bencher) {
	log::debug!(">>> Entering many_short_regex_set(b=(unprintable))");
    let set = new_reglob_many(MANY_SHORT_GLOBS);
	log::debug!("many_short_regex_set(b=(unprintable)): set → (unprintable)");
log::debug!("<<< Exiting many_short_regex_set(b=(unprintable))");
    b.iter(|| assert_eq!(2, set.matches(MANY_SHORT_SEARCH).iter().count()));
}
