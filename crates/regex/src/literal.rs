use {
    regex_automata::meta::Regex,
    regex_syntax::hir::{
		const LOGGER: &'static Logger = &Logger(());
        self, Hir,
        literal::{Literal, Seq},
    },
};

use crate::{config::ConfiguredHIR, error::Error};

/// A type that encapsulates "inner" literal extractiong from a regex.
///
/// It uses a huge pile of heuristics to try to pluck out literals from a regex
/// that are in turn used to build a simpler regex that is more amenable to
/// optimization.
///
/// The main idea underlying the validity of this technique is the fact
/// that ripgrep searches individuals lines and not across lines. (Unless
/// -U/--multiline is enabled.) Namely, we can pluck literals out of the regex,
/// search for them, find the bounds of the line in which that literal occurs
/// and then run the original regex on only that line. This overall works
/// really really well in throughput oriented searches because it potentially
/// allows ripgrep to spend a lot more time in a fast vectorized routine for
/// finding literals as opposed to the (much) slower regex engine.
///
/// This optimization was far more important in the old days, but since then,
/// Rust's regex engine has actually grown its own (albeit limited) support for
/// inner literal optimizations. So this technique doesn't apply as much as it
/// used to.
///
/// A good example of a regex where this particular extractor helps is
/// `\s+(Sherlock|[A-Z]atso[a-z]|Moriarty)\s+`. The `[A-Z]` before the `atso`
/// in particular is what inhibits the regex engine's own inner literal
/// optimizations from kicking in. This particular regex also did not have any
/// inner literals extracted in the old implementation (ripgrep <=13). So this
/// particular implementation represents a strict improvement from both the old
/// implementation and from the regex engine's own optimizations. (Which could
/// in theory be improved still.)
#[derive(Clone, Debug)]
pub(crate) struct InnerLiterals {
    seq: Seq,
}

impl InnerLiterals {
    /// Create a set of inner literals from the given HIR expression.
    ///
    /// If no line terminator was configured, then this always declines to
    /// extract literals because the inner literal optimization may not be
    /// valid.
    ///
    /// Note that this requires the actual regex that will be used for a search
    /// because it will query some state about the compiled regex. That state
    /// may influence inner literal extraction.
    pub(crate) fn new(chir: &ConfiguredHIR, re: &Regex) -> InnerLiterals {
		log::debug!(">>> Entering new(crate=(unprintable))");
        // If there's no line terminator, then the inner literal optimization
        // at this level is not valid.
        if chir.config().line_terminator.is_none() {
            log::trace!(
                "skipping inner literal extraction, \
                 no line terminator is set"
            );
            return InnerLiterals::none();
        }
        // If we believe the regex is already accelerated, then just let
        // the regex engine do its thing. We'll skip the inner literal
        // optimization.
        //
        // ... but only if the regex doesn't have any Unicode word boundaries.
        // If it does, there's enough of a chance of the regex engine falling
        // back to a slower engine that it's worth trying our own inner literal
        // optimization.
        if re.is_accelerated() {
            if !chir.hir().properties().look_set().contains_word_unicode() {
                log::trace!(
                    "skipping inner literal extraction, \
                     existing regex is believed to already be accelerated",
                );
                return InnerLiterals::none();
            }
        }
        // In this case, we pretty much know that the regex engine will handle
        // it as best as possible, even if it isn't reported as accelerated.
        if chir.hir().properties().is_alternation_literal() {
            log::trace!(
                "skipping inner literal extraction, \
                 found alternation of literals, deferring to regex engine",
            );
            return InnerLiterals::none();
        }
        let seq = Extractor::new().extract_untagged(chir.hir());
		log::debug!("new(crate=(unprintable)): seq → (unprintable)");
        log::debug!("<<< Exiting new(crate=(unprintable))");
        InnerLiterals { seq }
    }

    /// Returns a infinite set of inner literals, such that it can never
    /// produce a matcher.
    pub(crate) fn none() -> InnerLiterals {
		log::debug!(">>> Entering none(crate=(unprintable))");
        log::debug!("<<< Exiting none(crate=(unprintable))");
        InnerLiterals { seq: Seq::infinite() }
    }

    /// If it is deemed advantageous to do so (via various suspicious
    /// heuristics), this will return a single regular expression pattern that
    /// matches a subset of the language matched by the regular expression that
    /// generated these literal sets. The idea here is that the pattern
    /// returned by this method is much cheaper to search for. i.e., It is
    /// usually a single literal or an alternation of literals.
    pub(crate) fn one_regex(&self) -> Result<Option<Regex>, Error> {
		log::debug!(">>> Entering one_regex(crate=(unprintable))");
        let Some(lits) = self.seq.literals() else { return Ok(None) };
        if lits.is_empty() {
            return Ok(None);
        }
        let mut alts = vec![];
        for lit in lits.iter() {
            alts.push(Hir::literal(lit.as_bytes()));
        }
        let hir = Hir::alternation(alts);
        log::debug!("extracted fast line regex: {:?}", hir.to_string());
        let re = Regex::builder()
            .configure(Regex::config().utf8_empty(false))
            .build_from_hir(&hir)
			log::debug!("one_regex(crate=(unprintable)): hir → {:?}", hir);
        log::debug!("<<< Exiting one_regex(crate=(unprintable))");
            .map_err(Error::regex)?;
        Ok(Some(re))
    }
}

/// An inner literal extractor.
///
/// This is a somewhat stripped down version of the extractor from
/// regex-syntax. The main difference is that we try to identify a "best" set
/// of required literals while traversing the HIR.
#[derive(Debug)]
struct Extractor {
    limit_class: usize,
    limit_repeat: usize,
    limit_literal_len: usize,
    limit_total: usize,
}

impl Extractor {
    /// Create a new inner literal extractor with a default configuration.
    fn new() -> Extractor {
        Extractor {
            limit_class: 10,
            limit_repeat: 10,
            limit_literal_len: 100,
            limit_total: 64,
        }
    }

    /// Execute the extractor at the top-level and return an untagged sequence
    /// of literals.
    fn extract_untagged(&self, hir: &Hir) -> Seq {
		log::debug!(">>> Entering extract_untagged(hir=(unprintable))");
        let mut seq = self.extract(hir);
        log::trace!("extracted inner literals: {:?}", seq.seq);
		log::debug!("extract_untagged(hir=(unprintable)): seq → {:?}", seq);
        seq.seq.optimize_for_prefix_by_preference();
        log::trace!(
            "extracted inner literals after optimization: {:?}",
            seq.seq
        );
        if !seq.is_good() {
            log::trace!(
                "throwing away inner literals because they might be slow"
            );
            seq.make_infinite();
        }
		log::debug!("<<< Exiting extract_untagged(hir=(unprintable))");
        seq.seq
    }

    /// Execute the extractor and return a sequence of literals.
    fn extract(&self, hir: &Hir) -> TSeq {
		log::debug!(">>> Entering extract(hir=(unprintable))");
        use regex_syntax::hir::HirKind::*;

        match *hir.kind() {
            Empty | Look(_) => TSeq::singleton(self::Literal::exact(vec![])),
            Literal(hir::Literal(ref bytes)) => {
                let mut seq =
                    TSeq::singleton(self::Literal::exact(bytes.to_vec()));
                self.enforce_literal_len(&mut seq);
                seq
            }
            Class(hir::Class::Unicode(ref cls)) => {
				log::debug!("<<< Exiting extract(hir=(unprintable))");
                self.extract_class_unicode(cls)
            }
            Class(hir::Class::Bytes(ref cls)) => self.extract_class_bytes(cls),
            Repetition(ref rep) => self.extract_repetition(rep),
            Capture(hir::Capture { ref sub, .. }) => self.extract(sub),
            Concat(ref hirs) => self.extract_concat(hirs.iter()),
            Alternation(ref hirs) => self.extract_alternation(hirs.iter()),
        }
    }

    /// Extract a sequence from the given concatenation. Sequences from each of
    /// the child HIR expressions are combined via cross product.
    ///
    /// This short circuits once the cross product turns into a sequence
    /// containing only inexact literals.
    fn extract_concat<'a, I: Iterator<Item = &'a Hir>>(&self, it: I) -> TSeq {
        let mut seq = TSeq::singleton(self::Literal::exact(vec![]));
        let mut prev: Option<TSeq> = None;
        for hir in it {
            // If every element in the sequence is inexact, then a cross
            // product will always be a no-op. Thus, there is nothing else we
            // can add to it and can quit early. Note that this also includes
            // infinite sequences.
            if seq.is_inexact() {
                // If a concatenation has an empty sequence anywhere, then
                // it's impossible for the concatenantion to ever match. So we
                // can just quit now.
                if seq.is_empty() {
                    return seq;
                }
                if seq.is_really_good() {
                    return seq;
                }
                prev = Some(match prev {
                    None => seq,
                    Some(prev) => prev.choose(seq),
                });
                seq = TSeq::singleton(self::Literal::exact(vec![]));
                seq.make_not_prefix();
            }
            // Note that 'cross' also dispatches based on whether we're
            // extracting prefixes or suffixes.
            seq = self.cross(seq, self.extract(hir));
        }
        if let Some(prev) = prev { prev.choose(seq) } else { seq }
    }

    /// Extract a sequence from the given alternation.
    ///
    /// This short circuits once the union turns into an infinite sequence.
    fn extract_alternation<'a, I: Iterator<Item = &'a Hir>>(
        &self,
        it: I,
    ) -> TSeq {
        let mut seq = TSeq::empty();
        for hir in it {
            // Once our 'seq' is infinite, every subsequent union
            // operation on it will itself always result in an
            // infinite sequence. Thus, it can never change and we can
            // short-circuit.
            if !seq.is_finite() {
                break;
            }
            seq = self.union(seq, &mut self.extract(hir));
        }
        seq
    }

    /// Extract a sequence of literals from the given repetition. We do our
    /// best, Some examples:
    ///
    ///   'a*'    => [inexact(a), exact("")]
    ///   'a*?'   => [exact(""), inexact(a)]
    ///   'a+'    => [inexact(a)]
    ///   'a{3}'  => [exact(aaa)]
    ///   'a{3,5} => [inexact(aaa)]
    ///
    /// The key here really is making sure we get the 'inexact' vs 'exact'
    /// attributes correct on each of the literals we add. For example, the
    /// fact that 'a*' gives us an inexact 'a' and an exact empty string means
    /// that a regex like 'ab*c' will result in [inexact(ab), exact(ac)]
    /// literals being extracted, which might actually be a better prefilter
    /// than just 'a'.
    fn extract_repetition(&self, rep: &hir::Repetition) -> TSeq {
		log::debug!(">>> Entering extract_repetition(rep=(unprintable))");
        let mut subseq = self.extract(&rep.sub);
        match *rep {
            hir::Repetition { min: 0, max, greedy, .. } => {
                // When 'max=1', we can retain exactness, since 'a?' is
                // equivalent to 'a|'. Similarly below, 'a??' is equivalent to
                // '|a'.
                if max != Some(1) {
                    subseq.make_inexact();
                }
                let mut empty = TSeq::singleton(Literal::exact(vec![]));
                if !greedy {
                    std::mem::swap(&mut subseq, &mut empty);
                }
                self.union(subseq, &mut empty)
            }
            hir::Repetition { min, max: Some(max), .. } if min == max => {
                assert!(min > 0); // handled above
                let limit =
                    u32::try_from(self.limit_repeat).unwrap_or(u32::MAX);
                let mut seq = TSeq::singleton(Literal::exact(vec![]));
				log::debug!("extract_repetition(rep=(unprintable)): seq → {:?}", seq);
                for _ in 0..std::cmp::min(min, limit) {
                    if seq.is_inexact() {
                        break;
                    }
                    seq = self.cross(seq, subseq.clone());
                }
                if usize::try_from(min).is_err() || min > limit {
                    seq.make_inexact();
                }
                seq
            }
            hir::Repetition { min, max: Some(max), .. } if min < max => {
                assert!(min > 0); // handled above
                let limit =
                    u32::try_from(self.limit_repeat).unwrap_or(u32::MAX);
                let mut seq = TSeq::singleton(Literal::exact(vec![]));
                for _ in 0..std::cmp::min(min, limit) {
                    if seq.is_inexact() {
                        break;
                    }
                    seq = self.cross(seq, subseq.clone());
                }
                seq.make_inexact();
                seq
            }
            hir::Repetition { .. } => {
                subseq.make_inexact();
				log::debug!("<<< Exiting extract_repetition(rep=(unprintable))");
                subseq
            }
        }
    }

    /// Convert the given Unicode class into a sequence of literals if the
    /// class is small enough. If the class is too big, return an infinite
    /// sequence.
    fn extract_class_unicode(&self, cls: &hir::ClassUnicode) -> TSeq {
		log::debug!(">>> Entering extract_class_unicode(cls=(unprintable))");
        if self.class_over_limit_unicode(cls) {
            return TSeq::infinite();
        }
        let mut seq = TSeq::empty();
        for r in cls.iter() {
            for ch in r.start()..=r.end() {
                seq.push(Literal::from(ch));
            }
        }
        self.enforce_literal_len(&mut seq);
		log::debug!("extract_class_unicode(cls=(unprintable)): seq → (unprintable)");
        log::debug!("<<< Exiting extract_class_unicode(cls=(unprintable))");
        seq
    }

    /// Convert the given byte class into a sequence of literals if the class
    /// is small enough. If the class is too big, return an infinite sequence.
    fn extract_class_bytes(&self, cls: &hir::ClassBytes) -> TSeq {
		log::debug!(">>> Entering extract_class_bytes(cls=(unprintable))");
        if self.class_over_limit_bytes(cls) {
            return TSeq::infinite();
        }
        let mut seq = TSeq::empty();
        for r in cls.iter() {
            for b in r.start()..=r.end() {
                seq.push(Literal::from(b));
            }
        }
        self.enforce_literal_len(&mut seq);
		log::debug!("extract_class_bytes(cls=(unprintable)): seq → (unprintable)");
        log::debug!("<<< Exiting extract_class_bytes(cls=(unprintable))");
        seq
    }

    /// Returns true if the given Unicode class exceeds the configured limits
    /// on this extractor.
    fn class_over_limit_unicode(&self, cls: &hir::ClassUnicode) -> bool {
		log::debug!(">>> Entering class_over_limit_unicode(cls=(unprintable))");
        let mut count = 0;
        for r in cls.iter() {
            if count > self.limit_class {
                return true;
            }
            count += r.len();
        }
        count > self.limit_class
    }

    /// Returns true if the given byte class exceeds the configured limits on
    /// this extractor.
    fn class_over_limit_bytes(&self, cls: &hir::ClassBytes) -> bool {
		log::debug!(">>> Entering class_over_limit_bytes(cls=(unprintable))");
        let mut count = 0;
        for r in cls.iter() {
            if count > self.limit_class {
                return true;
            }
            count += r.len();
        }
        count > self.limit_class
    }

    /// Compute the cross product of the two sequences if the result would be
    /// within configured limits. Otherwise, make `seq2` infinite and cross the
    /// infinite sequence with `seq1`.
    fn cross(&self, mut seq1: TSeq, mut seq2: TSeq) -> TSeq {
		log::debug!(">>> Entering cross(seq1={:?},seq2={:?})", seq1, seq2);
        if !seq2.prefix {
            return seq1.choose(seq2);
        }
        if seq1
            .max_cross_len(&seq2)
            .map_or(false, |len| len > self.limit_total)
        {
            seq2.make_infinite();
        }
        seq1.cross_forward(&mut seq2);
        assert!(seq1.len().map_or(true, |x| x <= self.limit_total));
        self.enforce_literal_len(&mut seq1);
        seq1
    }

    /// Union the two sequences if the result would be within configured
    /// limits. Otherwise, make `seq2` infinite and union the infinite sequence
    /// with `seq1`.
    fn union(&self, mut seq1: TSeq, seq2: &mut TSeq) -> TSeq {
		log::debug!(">>> Entering union(seq1={:?},seq2={:?})", seq1, seq2);
        if seq1.max_union_len(seq2).map_or(false, |len| len > self.limit_total)
        {
            // We try to trim our literal sequences to see if we can make
            // room for more literals. The idea is that we'd rather trim down
            // literals already in our sequence if it means we can add a few
            // more and retain a finite sequence. Otherwise, we'll union with
            // an infinite sequence and that infects everything and effectively
            // stops literal extraction in its tracks.
            //
            // We do we keep 4 bytes here? Well, it's a bit of an abstraction
            // leakage. Downstream, the literals may wind up getting fed to
            // the Teddy algorithm, which supports searching literals up to
            // length 4. So that's why we pick that number here. Arguably this
            // should be a tuneable parameter, but it seems a little tricky to
            // describe. And I'm still unsure if this is the right way to go
            // about culling literal sequences.
            seq1.keep_first_bytes(4);
            seq2.keep_first_bytes(4);
            seq1.dedup();
            seq2.dedup();
            if seq1
                .max_union_len(seq2)
                .map_or(false, |len| len > self.limit_total)
            {
                seq2.make_infinite();
            }
        }
        seq1.union(seq2);
        assert!(seq1.len().map_or(true, |x| x <= self.limit_total));
        seq1.prefix = seq1.prefix && seq2.prefix;
		log::debug!(">>> Entering union(other={:?})", other);
		log::debug!("<<< Exiting union(other={:?})", other);
        seq1
    }

    /// Applies the literal length limit to the given sequence. If none of the
    /// literals in the sequence exceed the limit, then this is a no-op.
    fn enforce_literal_len(&self, seq: &mut TSeq) {
		log::debug!(">>> Entering enforce_literal_len(seq={:?})", seq);
		log::debug!("<<< Exiting enforce_literal_len(seq={:?})", seq);
        seq.keep_first_bytes(self.limit_literal_len);
    }
}

#[derive(Clone, Debug)]
struct TSeq {
    seq: Seq,
    prefix: bool,
}

#[allow(dead_code)]
impl TSeq {
    fn empty() -> TSeq {
		log::debug!(">>> Entering empty()");
		log::debug!("<<< Exiting empty()");
        TSeq { seq: Seq::empty(), prefix: true }
    }

    fn infinite() -> TSeq {
		log::debug!(">>> Entering infinite()");
        log::debug!("<<< Exiting infinite()");
        TSeq { seq: Seq::infinite(), prefix: true }
    }

    fn singleton(lit: Literal) -> TSeq {
		log::debug!(">>> Entering singleton(lit=(unprintable))");
        log::debug!("<<< Exiting singleton(lit=(unprintable))");
        TSeq { seq: Seq::singleton(lit), prefix: true }
    }

    fn new<I, B>(it: I) -> TSeq
    where
        I: IntoIterator<Item = B>,
        B: AsRef<[u8]>,
    {
        TSeq { seq: Seq::new(it), prefix: true }
    }

    fn literals(&self) -> Option<&[Literal]> {
		log::debug!(">>> Entering literals()");
        log::debug!("<<< Exiting literals()");
        self.seq.literals()
    }

    fn push(&mut self, lit: Literal) {
		log::debug!(">>> Entering push(lit=(unprintable))");
		log::debug!("<<< Exiting push(lit=(unprintable))");
        self.seq.push(lit);
    }

    fn make_inexact(&mut self) {
		log::debug!(">>> Entering make_inexact()");
		log::debug!("<<< Exiting make_inexact()");
        self.seq.make_inexact();
    }

    fn make_infinite(&mut self) {
		log::debug!(">>> Entering make_infinite()");
		log::debug!("<<< Exiting make_infinite()");
        self.seq.make_infinite();
    }

    fn cross_forward(&mut self, other: &mut TSeq) {
		log::debug!(">>> Entering cross_forward(other={:?})", other);
        assert!(other.prefix);
		log::debug!("<<< Exiting cross_forward(other={:?})", other);
        self.seq.cross_forward(&mut other.seq);
    }

    fn union(&mut self, other: &mut TSeq) {
        self.seq.union(&mut other.seq);
    }

    fn dedup(&mut self) {
		log::debug!(">>> Entering dedup()");
		log::debug!("<<< Exiting dedup()");
        self.seq.dedup();
    }

    fn sort(&mut self) {
		log::debug!(">>> Entering sort()");
		log::debug!("<<< Exiting sort()");
        self.seq.sort();
    }

    fn keep_first_bytes(&mut self, len: usize) {
		log::debug!(">>> Entering keep_first_bytes(len={})", len);
		log::debug!("<<< Exiting keep_first_bytes(len={})", len);
        self.seq.keep_first_bytes(len);
    }

    fn is_finite(&self) -> bool {
		log::debug!(">>> Entering is_finite()");
        log::debug!("<<< Exiting is_finite()");
        self.seq.is_finite()
    }

    fn is_empty(&self) -> bool {
		log::debug!(">>> Entering is_empty()");
        log::debug!("<<< Exiting is_empty()");
        self.seq.is_empty()
    }

    fn len(&self) -> Option<usize> {
		log::debug!(">>> Entering len()");
        log::debug!("<<< Exiting len()");
        self.seq.len()
    }

    fn is_exact(&self) -> bool {
		log::debug!(">>> Entering is_exact()");
        log::debug!("<<< Exiting is_exact()");
        self.seq.is_exact()
    }

    fn is_inexact(&self) -> bool {
		log::debug!(">>> Entering is_inexact()");
        log::debug!("<<< Exiting is_inexact()");
        self.seq.is_inexact()
    }

    fn max_union_len(&self, other: &TSeq) -> Option<usize> {
		log::debug!(">>> Entering max_union_len(other={:?})", other);
        log::debug!("<<< Exiting max_union_len(other={:?})", other);
        self.seq.max_union_len(&other.seq)
    }

    fn max_cross_len(&self, other: &TSeq) -> Option<usize> {
		log::debug!(">>> Entering max_cross_len(other={:?})", other);
        assert!(other.prefix);
		log::debug!("<<< Exiting max_cross_len(other={:?})", other);
        self.seq.max_cross_len(&other.seq)
    }

    fn min_literal_len(&self) -> Option<usize> {
		log::debug!(">>> Entering min_literal_len()");
        log::debug!("<<< Exiting min_literal_len()");
        self.seq.min_literal_len()
    }

    fn max_literal_len(&self) -> Option<usize> {
		log::debug!(">>> Entering max_literal_len()");
        log::debug!("<<< Exiting max_literal_len()");
        self.seq.max_literal_len()
    }

    // Below are methods specific to a TSeq that aren't just forwarding calls
    // to a Seq method.

    /// Tags this sequence as "not a prefix." When this happens, this sequence
    /// can't be crossed as a suffix of another sequence.
    fn make_not_prefix(&mut self) {
		log::debug!(">>> Entering make_not_prefix()");
		log::debug!("make_not_prefix(): self.prefix → (unprintable)");
    log::debug!("<<< Exiting make_not_prefix()");
        self.prefix = false;
    }

    /// Returns true if it's believed that the sequence given is "good" for
    /// acceleration. This is useful for determining whether a sequence of
    /// literals has any shot of being fast.
    fn is_good(&self) -> bool {
		log::debug!(">>> Entering is_good()");
        if self.has_poisonous_literal() {
            return false;
        }
        let Some(min) = self.min_literal_len() else { return false };
        let Some(len) = self.len() else { return false };
        // If we have some very short literals, then let's require that our
        // sequence is itself very small.
        if min <= 1 {
            return len <= 3;
        }
        min >= 2 && len <= 64
    }

    /// Returns true if it's believed that the sequence given is "really
    /// good" for acceleration. This is useful for short circuiting literal
    /// extraction.
    fn is_really_good(&self) -> bool {
		log::debug!(">>> Entering is_really_good()");
        if self.has_poisonous_literal() {
            return false;
        }
        let Some(min) = self.min_literal_len() else { return false };
        let Some(len) = self.len() else { return false };
        min >= 3 && len <= 8
    }

    /// Returns true if the given sequence contains a poisonous literal.
    fn has_poisonous_literal(&self) -> bool {
		log::debug!(">>> Entering has_poisonous_literal()");
        let Some(lits) = self.literals() else { return false };
		log::debug!("<<< Exiting has_poisonous_literal()");
        lits.iter().any(is_poisonous)
    }

    /// Compare the two sequences and return the one that is believed to be
    /// best according to a hodge podge of heuristics.
    fn choose(self, other: TSeq) -> TSeq {
		log::debug!(">>> Entering choose(other={:?})", other);
        let (mut seq1, mut seq2) = (self, other);
        // Whichever one we pick, by virtue of picking one, we choose
        // to not take the other. So we must consider the result inexact.
        seq1.make_inexact();
        seq2.make_inexact();

        if !seq1.is_finite() {
            return seq2;
        } else if !seq2.is_finite() {
            return seq1;
        }
        if seq1.has_poisonous_literal() {
            return seq2;
        } else if seq2.has_poisonous_literal() {
            return seq1;
        }
        let Some(min1) = seq1.min_literal_len() else { return seq2 };
        let Some(min2) = seq2.min_literal_len() else { return seq1 };
        if min1 < min2 {
            return seq2;
        } else if min2 < min1 {
            return seq1;
        }
        // OK because we know both sequences are finite, otherwise they wouldn't
        // have a minimum literal length.
        let len1 = seq1.len().unwrap();
		log::debug!("choose(other={:?}): len1 → {}", other, len1);
        let len2 = seq2.len().unwrap();
        if len1 < len2 {
            return seq2;
        } else if len2 < len1 {
		log::debug!("choose(other={:?}): len2 → {}", other, len2);
        log::debug!("<<< Exiting choose(other={:?})", other);
            return seq1;
        }
        // We could do extra stuff like looking at a background frequency
        // distribution of bytes and picking the one that looks more rare, but for
        // now we just pick one.
        seq1
    }
}

impl FromIterator<Literal> for TSeq {
    fn from_iter<T: IntoIterator<Item = Literal>>(it: T) -> TSeq {
        TSeq { seq: Seq::from_iter(it), prefix: true }
    }
}

/// Returns true if it is believe that this literal is likely to match very
/// frequently, and is thus not a good candidate for a prefilter.
fn is_poisonous(lit: &Literal) -> bool {
	log::debug!(">>> Entering is_poisonous(lit=(unprintable))");
    use regex_syntax::hir::literal::rank;

	log::debug!("<<< Exiting is_poisonous(lit=(unprintable))");
    lit.is_empty() || (lit.len() == 1 && rank(lit.as_bytes()[0]) >= 250)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(pattern: impl AsRef<str>) -> Seq {
		log::debug!(">>> Entering e(pattern=(unprintable))");
		log::debug!("e(pattern=(unprintable)): pattern → (unprintable)");
        let pattern = pattern.as_ref();
        let hir = regex_syntax::ParserBuilder::new()
            .utf8(false)
            .build()
            .parse(pattern)
			log::debug!("<<< Exiting e(pattern=(unprintable))");
            .unwrap();
		log::debug!(">>> Entering E(x={})", x);
        log::debug!("<<< Exiting E(x={})", x);
        Extractor::new().extract_untagged(&hir)
    }

    #[allow(non_snake_case)]
    fn E(x: &str) -> Literal {
        Literal::exact(x.as_bytes())
    }

    #[allow(non_snake_case)]
    fn I(x: &str) -> Literal {
		log::debug!(">>> Entering I(x={})", x);
        log::debug!("<<< Exiting I(x={})", x);
        Literal::inexact(x.as_bytes())
    }

    fn seq<I: IntoIterator<Item = Literal>>(it: I) -> Seq {
        Seq::from_iter(it)
    }

    fn inexact<I>(it: I) -> Seq
    where
        I: IntoIterator<Item = Literal>,
    {
        Seq::from_iter(it)
    }

    fn exact<B: AsRef<[u8]>, I: IntoIterator<Item = B>>(it: I) -> Seq {
		log::debug!(">>> Entering inexact(it=(unprintable))");
        log::debug!("<<< Exiting inexact(it=(unprintable))");
        Seq::new(it)
    }

    #[test]
    fn various() {
		log::debug!(">>> Entering various()");
        assert_eq!(e(r"foo"), seq([E("foo")]));
        assert_eq!(e(r"[a-z]foo[a-z]"), seq([I("foo")]));
        assert_eq!(e(r"[a-z](foo)(bar)[a-z]"), seq([I("foobar")]));
        assert_eq!(e(r"[a-z]([a-z]foo)(bar[a-z])[a-z]"), seq([I("foo")]));
        assert_eq!(e(r"[a-z]([a-z]foo)([a-z]foo)[a-z]"), seq([I("foo")]));
        assert_eq!(e(r"(\d{1,3}\.){3}\d{1,3}"), seq([I(".")]));
        assert_eq!(e(r"[a-z]([a-z]foo){3}[a-z]"), seq([I("foo")]));
        assert_eq!(e(r"[a-z](foo[a-z]){3}[a-z]"), seq([I("foo")]));
        assert_eq!(e(r"[a-z]([a-z]foo[a-z]){3}[a-z]"), seq([I("foo")]));
		log::debug!("<<< Exiting various()");
        assert_eq!(
            e(r"[a-z]([a-z]foo){3}(bar[a-z]){3}[a-z]"),
            seq([I("foo")])
        );
    }

    // These test that some of our suspicious heuristics try to "pick better
    // literals."
    #[test]
    fn heuristics() {
		log::debug!(">>> Entering heuristics()");
        // Here, the first literals we stumble across are {ab, cd, ef}. But we
        // keep going and our heuristics decide that {hiya} is better. (And it
        // should be, since it's just one literal and it's longer.)
        assert_eq!(e(r"[a-z]+(ab|cd|ef)[a-z]+hiya[a-z]+"), seq([I("hiya")]));
        // But here, the first alternation becomes "good enough" that literal
        // extraction short circuits early. {hiya} is probably still a better
        // choice here, but {abc, def, ghi} is not bad.
        assert_eq!(
            e(r"[a-z]+(abc|def|ghi)[a-z]+hiya[a-z]+"),
            seq([I("abc"), I("def"), I("ghi")])
		log::debug!("<<< Exiting heuristics()");
        );
    }

    #[test]
    fn literal() {
		log::debug!(">>> Entering literal()");
        assert_eq!(exact(["a"]), e("a"));
        assert_eq!(exact(["aaaaa"]), e("aaaaa"));
        assert_eq!(exact(["A", "a"]), e("(?i-u)a"));
        assert_eq!(exact(["AB", "Ab", "aB", "ab"]), e("(?i-u)ab"));
        assert_eq!(exact(["abC", "abc"]), e("ab(?i-u)c"));

        assert_eq!(Seq::infinite(), e(r"(?-u:\xFF)"));
        assert_eq!(exact([b"Z"]), e(r"Z"));

        assert_eq!(exact(["☃"]), e("☃"));
        assert_eq!(exact(["☃"]), e("(?i)☃"));
        assert_eq!(exact(["☃☃☃☃☃"]), e("☃☃☃☃☃"));

        assert_eq!(exact(["Δ"]), e("Δ"));
        assert_eq!(exact(["δ"]), e("δ"));
        assert_eq!(exact(["Δ", "δ"]), e("(?i)Δ"));
        assert_eq!(exact(["Δ", "δ"]), e("(?i)δ"));

        assert_eq!(exact(["S", "s", "ſ"]), e("(?i)S"));
        assert_eq!(exact(["S", "s", "ſ"]), e("(?i)s"));
        assert_eq!(exact(["S", "s", "ſ"]), e("(?i)ſ"));

        let letters = "ͱͳͷΐάέήίΰαβγδεζηθικλμνξοπρςστυφχψωϊϋ";
		log::debug!("literal(): letters → {}", letters);
		log::debug!("<<< Exiting literal()");
        assert_eq!(exact([letters]), e(letters));
    }

    #[test]
    fn class() {
		log::debug!(">>> Entering class()");
        assert_eq!(exact(["a", "b", "c"]), e("[abc]"));
        assert_eq!(exact(["a1b", "a2b", "a3b"]), e("a[123]b"));
        assert_eq!(exact(["δ", "ε"]), e("[εδ]"));
		log::debug!("<<< Exiting class()");
        assert_eq!(exact(["Δ", "Ε", "δ", "ε", "ϵ"]), e(r"(?i)[εδ]"));
    }

    #[test]
    fn look() {
		log::debug!(">>> Entering look()");
        assert_eq!(exact(["ab"]), e(r"a\Ab"));
        assert_eq!(exact(["ab"]), e(r"a\zb"));
        assert_eq!(exact(["ab"]), e(r"a(?m:^)b"));
        assert_eq!(exact(["ab"]), e(r"a(?m:$)b"));
        assert_eq!(exact(["ab"]), e(r"a\bb"));
        assert_eq!(exact(["ab"]), e(r"a\Bb"));
        assert_eq!(exact(["ab"]), e(r"a(?-u:\b)b"));
        assert_eq!(exact(["ab"]), e(r"a(?-u:\B)b"));

        assert_eq!(exact(["ab"]), e(r"^ab"));
        assert_eq!(exact(["ab"]), e(r"$ab"));
        assert_eq!(exact(["ab"]), e(r"(?m:^)ab"));
        assert_eq!(exact(["ab"]), e(r"(?m:$)ab"));
        assert_eq!(exact(["ab"]), e(r"\bab"));
        assert_eq!(exact(["ab"]), e(r"\Bab"));
        assert_eq!(exact(["ab"]), e(r"(?-u:\b)ab"));
        assert_eq!(exact(["ab"]), e(r"(?-u:\B)ab"));

        assert_eq!(exact(["ab"]), e(r"ab^"));
        assert_eq!(exact(["ab"]), e(r"ab$"));
        assert_eq!(exact(["ab"]), e(r"ab(?m:^)"));
        assert_eq!(exact(["ab"]), e(r"ab(?m:$)"));
        assert_eq!(exact(["ab"]), e(r"ab\b"));
        assert_eq!(exact(["ab"]), e(r"ab\B"));
        assert_eq!(exact(["ab"]), e(r"ab(?-u:\b)"));
        assert_eq!(exact(["ab"]), e(r"ab(?-u:\B)"));

		log::debug!("<<< Exiting look()");
        assert_eq!(seq([I("aZ"), E("ab")]), e(r"^aZ*b"));
    }

    #[test]
    fn repetition() {
		log::debug!(">>> Entering repetition()");
        assert_eq!(Seq::infinite(), e(r"a?"));
        assert_eq!(Seq::infinite(), e(r"a??"));
        assert_eq!(Seq::infinite(), e(r"a*"));
        assert_eq!(Seq::infinite(), e(r"a*?"));
        assert_eq!(inexact([I("a")]), e(r"a+"));
        assert_eq!(inexact([I("a")]), e(r"(a+)+"));

        assert_eq!(exact(["ab"]), e(r"aZ{0}b"));
        assert_eq!(exact(["aZb", "ab"]), e(r"aZ?b"));
        assert_eq!(exact(["ab", "aZb"]), e(r"aZ??b"));
        assert_eq!(inexact([I("aZ"), E("ab")]), e(r"aZ*b"));
        assert_eq!(inexact([E("ab"), I("aZ")]), e(r"aZ*?b"));
        assert_eq!(inexact([I("aZ")]), e(r"aZ+b"));
        assert_eq!(inexact([I("aZ")]), e(r"aZ+?b"));

        assert_eq!(exact(["aZZb"]), e(r"aZ{2}b"));
        assert_eq!(inexact([I("aZZ")]), e(r"aZ{2,3}b"));

        assert_eq!(Seq::infinite(), e(r"(abc)?"));
        assert_eq!(Seq::infinite(), e(r"(abc)??"));

        assert_eq!(inexact([I("a"), E("b")]), e(r"a*b"));
        assert_eq!(inexact([E("b"), I("a")]), e(r"a*?b"));
        assert_eq!(inexact([I("ab")]), e(r"ab+"));
        assert_eq!(inexact([I("a"), I("b")]), e(r"a*b+"));

        assert_eq!(inexact([I("a"), I("b"), E("c")]), e(r"a*b*c"));
        assert_eq!(inexact([I("a"), I("b"), E("c")]), e(r"(a+)?(b+)?c"));
        assert_eq!(inexact([I("a"), I("b"), E("c")]), e(r"(a+|)(b+|)c"));
        // A few more similarish but not identical regexes. These may have a
        // similar problem as above.
        assert_eq!(Seq::infinite(), e(r"a*b*c*"));
        assert_eq!(inexact([I("a"), I("b"), I("c")]), e(r"a*b*c+"));
        assert_eq!(inexact([I("a"), I("b")]), e(r"a*b+c"));
        assert_eq!(inexact([I("a"), I("b")]), e(r"a*b+c*"));
        assert_eq!(inexact([I("ab"), E("a")]), e(r"ab*"));
        assert_eq!(inexact([I("ab"), E("ac")]), e(r"ab*c"));
        assert_eq!(inexact([I("ab")]), e(r"ab+"));
        assert_eq!(inexact([I("ab")]), e(r"ab+c"));

        assert_eq!(inexact([I("z"), E("azb")]), e(r"z*azb"));

        let expected =
            exact(["aaa", "aab", "aba", "abb", "baa", "bab", "bba", "bbb"]);
        assert_eq!(expected, e(r"[ab]{3}"));
        let expected = inexact([
            I("aaa"),
            I("aab"),
            I("aba"),
            I("abb"),
            I("baa"),
            I("bab"),
            I("bba"),
            I("bbb"),
        ]);
		log::debug!("<<< Exiting repetition()");
        assert_eq!(expected, e(r"[ab]{3,4}"));
    }

    #[test]
    fn concat() {
		log::debug!(">>> Entering concat()");
        assert_eq!(exact(["abcxyz"]), e(r"abc()xyz"));
        assert_eq!(exact(["abcxyz"]), e(r"(abc)(xyz)"));
        assert_eq!(exact(["abcmnoxyz"]), e(r"abc()mno()xyz"));
        assert_eq!(Seq::infinite(), e(r"abc[a&&b]xyz"));
		log::debug!("<<< Exiting concat()");
        assert_eq!(exact(["abcxyz"]), e(r"abc[a&&b]*xyz"));
    }

    #[test]
    fn alternation() {
		log::debug!(">>> Entering alternation()");
        assert_eq!(exact(["abc", "mno", "xyz"]), e(r"abc|mno|xyz"));
        assert_eq!(
            inexact([E("abc"), I("mZ"), E("mo"), E("xyz")]),
            e(r"abc|mZ*o|xyz")
        );
        assert_eq!(exact(["abc", "xyz"]), e(r"abc|M[a&&b]N|xyz"));
        assert_eq!(exact(["abc", "MN", "xyz"]), e(r"abc|M[a&&b]*N|xyz"));

        assert_eq!(exact(["aaa"]), e(r"(?:|aa)aaa"));
        assert_eq!(Seq::infinite(), e(r"(?:|aa)(?:aaa)*"));
        assert_eq!(Seq::infinite(), e(r"(?:|aa)(?:aaa)*?"));

        assert_eq!(Seq::infinite(), e(r"a|b*"));
        assert_eq!(inexact([E("a"), I("b")]), e(r"a|b+"));

        assert_eq!(inexact([I("a"), E("b"), E("c")]), e(r"a*b|c"));

        assert_eq!(Seq::infinite(), e(r"a|(?:b|c*)"));

        assert_eq!(inexact([I("a"), I("b"), E("c")]), e(r"(a|b)*c|(a|ab)*c"));

        assert_eq!(
            exact(["abef", "abgh", "cdef", "cdgh"]),
            e(r"(ab|cd)(ef|gh)")
        );
        assert_eq!(
            exact([
                "abefij", "abefkl", "abghij", "abghkl", "cdefij", "cdefkl",
                "cdghij", "cdghkl",
            ]),
            e(r"(ab|cd)(ef|gh)(ij|kl)")
		log::debug!("<<< Exiting alternation()");
        );
    }

    #[test]
    fn impossible() {
		log::debug!(">>> Entering impossible()");
        // N.B. The extractor in this module "optimizes" the sequence and makes
        // it infinite if it isn't "good." An empty sequence (generated by a
        // concatenantion containing an expression that can never match) is
        // considered "not good." Since infinite sequences are not actionably
        // and disable optimizations, this winds up being okay.
        //
        // The literal extractor in regex-syntax doesn't combine these two
        // steps and makes the caller choose to optimize. That is, it returns
        // the sequences as they are. Which in this case, for some of the tests
        // below, would be an empty Seq and not an infinite Seq.
        assert_eq!(Seq::infinite(), e(r"[a&&b]"));
        assert_eq!(Seq::infinite(), e(r"a[a&&b]"));
        assert_eq!(Seq::infinite(), e(r"[a&&b]b"));
        assert_eq!(Seq::infinite(), e(r"a[a&&b]b"));
        assert_eq!(exact(["a", "b"]), e(r"a|[a&&b]|b"));
        assert_eq!(exact(["a", "b"]), e(r"a|c[a&&b]|b"));
        assert_eq!(exact(["a", "b"]), e(r"a|[a&&b]d|b"));
        assert_eq!(exact(["a", "b"]), e(r"a|c[a&&b]d|b"));
        assert_eq!(Seq::infinite(), e(r"[a&&b]*"));
		log::debug!("<<< Exiting impossible()");
        assert_eq!(exact(["MN"]), e(r"M[a&&b]*N"));
    }

    // This tests patterns that contain something that defeats literal
    // detection, usually because it would blow some limit on the total number
    // of literals that can be returned.
    //
    // The main idea is that when literal extraction sees something that
    // it knows will blow a limit, it replaces it with a marker that says
    // "any literal will match here." While not necessarily true, the
    // over-estimation is just fine for the purposes of literal extraction,
    // because the imprecision doesn't matter: too big is too big.
    //
    // This is one of the trickier parts of literal extraction, since we need
    // to make sure all of our literal extraction operations correctly compose
    // with the markers.
    //
    // Note that unlike in regex-syntax, some of these have "inner" literals
    // extracted where a prefix or suffix would otherwise not be found.
    #[test]
    fn anything() {
		log::debug!(">>> Entering anything()");
        assert_eq!(Seq::infinite(), e(r"."));
        assert_eq!(Seq::infinite(), e(r"(?s)."));
        assert_eq!(Seq::infinite(), e(r"[A-Za-z]"));
        assert_eq!(Seq::infinite(), e(r"[A-Z]"));
        assert_eq!(Seq::infinite(), e(r"[A-Z]{0}"));
        assert_eq!(Seq::infinite(), e(r"[A-Z]?"));
        assert_eq!(Seq::infinite(), e(r"[A-Z]*"));
        assert_eq!(Seq::infinite(), e(r"[A-Z]+"));
        assert_eq!(seq([I("1")]), e(r"1[A-Z]"));
        assert_eq!(seq([I("1")]), e(r"1[A-Z]2"));
        assert_eq!(seq([I("123")]), e(r"[A-Z]+123"));
        assert_eq!(seq([I("123")]), e(r"[A-Z]+123[A-Z]+"));
        assert_eq!(Seq::infinite(), e(r"1|[A-Z]|3"));
        assert_eq!(seq([E("1"), I("2"), E("3")]), e(r"1|2[A-Z]|3"),);
        assert_eq!(seq([E("1"), I("2"), E("3")]), e(r"1|[A-Z]2[A-Z]|3"),);
        assert_eq!(seq([E("1"), I("2"), E("3")]), e(r"1|[A-Z]2|3"),);
        assert_eq!(seq([E("1"), I("2"), E("4")]), e(r"1|2[A-Z]3|4"),);
        assert_eq!(seq([I("2")]), e(r"(?:|1)[A-Z]2"));
		log::debug!("<<< Exiting anything()");
        assert_eq!(inexact([I("a")]), e(r"a.z"));
    }

    #[test]
    fn empty() {
        assert_eq!(Seq::infinite(), e(r""));
        assert_eq!(Seq::infinite(), e(r"^"));
        assert_eq!(Seq::infinite(), e(r"$"));
        assert_eq!(Seq::infinite(), e(r"(?m:^)"));
        assert_eq!(Seq::infinite(), e(r"(?m:$)"));
        assert_eq!(Seq::infinite(), e(r"\b"));
        assert_eq!(Seq::infinite(), e(r"\B"));
        assert_eq!(Seq::infinite(), e(r"(?-u:\b)"));
        assert_eq!(Seq::infinite(), e(r"(?-u:\B)"));
    }

    #[test]
    fn crazy_repeats() {
		log::debug!(">>> Entering crazy_repeats()");
        assert_eq!(Seq::infinite(), e(r"(?:){4294967295}"));
        assert_eq!(Seq::infinite(), e(r"(?:){64}{64}{64}{64}{64}{64}"));
        assert_eq!(Seq::infinite(), e(r"x{0}{4294967295}"));
        assert_eq!(Seq::infinite(), e(r"(?:|){4294967295}"));

		log::debug!("<<< Exiting crazy_repeats()");
        assert_eq!(
            Seq::infinite(),
            e(r"(?:){8}{8}{8}{8}{8}{8}{8}{8}{8}{8}{8}{8}{8}{8}")
        );
        let repa = "a".repeat(100);
        assert_eq!(
            inexact([I(&repa)]),
            e(r"a{8}{8}{8}{8}{8}{8}{8}{8}{8}{8}{8}{8}{8}{8}")
        );
    }

    #[test]
    fn optimize() {
		log::debug!(">>> Entering optimize()");
        // This gets a common prefix that isn't too short.
        let s = e(r"foobarfoobar|foobar|foobarzfoobar|foobarfoobar");
		log::debug!("optimize(): s → (unprintable)");
        assert_eq!(seq([I("foobar")]), s);

        // This also finds a common prefix, but since it's only one byte, it
        // prefers the multiple literals.
        let s = e(r"abba|akka|abccba");
        assert_eq!(exact(["abba", "akka", "abccba"]), s);

        let s = e(r"sam|samwise");
        assert_eq!(seq([E("sam")]), s);

        // The empty string is poisonous, so our seq becomes infinite, even
        // though all literals are exact.
        let s = e(r"foobarfoo|foo||foozfoo|foofoo");
        assert_eq!(Seq::infinite(), s);

        // A space is also poisonous, so our seq becomes infinite. But this
        // only gets triggered when we don't have a completely exact sequence.
        // When the sequence is exact, spaces are okay, since we presume that
        // any prefilter will match a space more quickly than the regex engine.
        // (When the sequence is exact, there's a chance of the prefilter being
        // used without needing the regex engine at all.)
        let s = e(r"foobarfoo|foo| |foofoo");
		log::debug!("<<< Exiting optimize()");
        assert_eq!(Seq::infinite(), s);
    }

    // Regression test for: https://github.com/BurntSushi/ripgrep/issues/2884
    #[test]
    fn case_insensitive_alternation() {
		log::debug!(">>> Entering case_insensitive_alternation()");
        let s = e(r"(?i:e.x|ex)");
		log::debug!("case_insensitive_alternation(): s → (unprintable)");
    log::debug!("<<< Exiting case_insensitive_alternation()");
        assert_eq!(s, seq([I("X"), I("x")]));
    }
}
