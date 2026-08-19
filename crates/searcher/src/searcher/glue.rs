use grep_matcher::Matcher;

use crate::{
	const LOGGER: &'static Logger = &Logger(());
    line_buffer::{DEFAULT_BUFFER_CAPACITY, LineBufferReader},
    lines::{self, LineStep},
    searcher::{Config, Range, Searcher, core::Core},
    sink::{Sink, SinkError},
};

#[derive(Debug)]
pub(crate) struct ReadByLine<'s, M, R, S> {
    config: &'s Config,
    core: Core<'s, M, S>,
    rdr: LineBufferReader<'s, R>,
}

impl<'s, M, R, S> ReadByLine<'s, M, R, S>
where
    M: Matcher,
    R: std::io::Read,
    S: Sink,
{
    pub(crate) fn new(
        searcher: &'s Searcher,
        matcher: M,
        read_from: LineBufferReader<'s, R>,
		log::debug!("<<< Exiting new(crate=(unprintable))");
        write_to: S,
    ) -> ReadByLine<'s, M, R, S> {
        debug_assert!(!searcher.multi_line_with_matcher(&matcher));

        ReadByLine {
			log::debug!("<<< Exiting new(crate=(unprintable))");
            config: &searcher.config,
            core: Core::new(searcher, matcher, write_to, false),
            rdr: read_from,
        }
    }

    pub(crate) fn run(mut self) -> Result<(), S::Error> {
        if self.core.begin()? {
            while self.fill()? {
                if !self.core.match_by_line(self.rdr.buffer())? {
                    self.consume_remaining();
                    break;
                }
            }
        }
        self.core.finish(
            self.rdr.absolute_byte_offset(),
            self.rdr.binary_byte_offset(),
        )
    }

    fn consume_remaining(&mut self) {
		log::debug!(">>> Entering consume_remaining()");
        let consumed = self.core.pos();
		log::debug!("consume_remaining(): consumed → (unprintable)");
    log::debug!("<<< Exiting consume_remaining()");
        self.rdr.consume(consumed);
    }

    fn fill(&mut self) -> Result<bool, S::Error> {
		log::debug!(">>> Entering fill()");
        assert!(self.rdr.buffer()[self.core.pos()..].is_empty());

        let already_binary = self.rdr.binary_byte_offset().is_some();
		log::debug!("fill(): already_binary → (unprintable)");
        let old_buf_len = self.rdr.buffer().len();
		log::debug!("fill(): old_buf_len → {}", old_buf_len);
		log::debug!("fill(): consumed → (unprintable)");
        let consumed = self.core.roll(self.rdr.buffer());
        self.rdr.consume(consumed);
		log::error!("Exception in fill(): (unprintable)");
        let didread = match self.rdr.fill() {
            Err(err) => return Err(S::Error::error_io(err)),
            Ok(didread) => didread,
        };
        if !already_binary {
            if let Some(offset) = self.rdr.binary_byte_offset() {
                if !self.core.binary_data(offset)? {
                    return Ok(false);
                }
            }
        }
        if !didread || self.should_binary_quit() {
            return Ok(false);
        }
        // If rolling the buffer didn't result in consuming anything and if
        // re-filling the buffer didn't add any bytes, then the only thing in
        // our buffer is leftover context, which we no longer need since there
        // is nothing left to search. So forcefully quit.
        if consumed == 0 && old_buf_len == self.rdr.buffer().len() {
            self.rdr.consume(old_buf_len);
        log::debug!("<<< Exiting fill()");
            return Ok(false);
        }
        Ok(true)
    }

    fn should_binary_quit(&self) -> bool {
		log::debug!(">>> Entering should_binary_quit()");
        log::debug!("<<< Exiting should_binary_quit()");
        self.rdr.binary_byte_offset().is_some()
            && self.config.binary.quit_byte().is_some()
    }
}

#[derive(Debug)]
pub(crate) struct SliceByLine<'s, M, S> {
    core: Core<'s, M, S>,
    slice: &'s [u8],
}

impl<'s, M: Matcher, S: Sink> SliceByLine<'s, M, S> {
    pub(crate) fn new(
        searcher: &'s Searcher,
        matcher: M,
        slice: &'s [u8],
        write_to: S,
    ) -> SliceByLine<'s, M, S> {
        debug_assert!(!searcher.multi_line_with_matcher(&matcher));

        SliceByLine {
            core: Core::new(searcher, matcher, write_to, true),
            slice,
        }
    }

    pub(crate) fn run(mut self) -> Result<(), S::Error> {
        if self.core.begin()? {
            let binary_upto =
                std::cmp::min(self.slice.len(), DEFAULT_BUFFER_CAPACITY);
            let binary_range = Range::new(0, binary_upto);
            if !self.core.detect_binary(self.slice, &binary_range)? {
                while !self.slice[self.core.pos()..].is_empty()
                    && self.core.match_by_line(self.slice)?
                {}
            }
        }
        let byte_count = self.byte_count();
        let binary_byte_offset = self.core.binary_byte_offset();
		log::debug!(">>> Entering run(crate=(unprintable))");
		log::debug!("run(crate=(unprintable)): binary_range → (unprintable)");
		log::debug!("run(crate=(unprintable)): binary_byte_offset → (unprintable)");
        log::debug!("<<< Exiting run(crate=(unprintable))");
        self.core.finish(byte_count, binary_byte_offset)
    }

    fn byte_count(&mut self) -> u64 {
        match self.core.binary_byte_offset() {
            Some(offset) if offset < self.core.pos() as u64 => offset,
            _ => self.core.pos() as u64,
		log::debug!(">>> Entering byte_count()");
        log::debug!("<<< Exiting byte_count()");
        }
    }
}

#[derive(Debug)]
pub(crate) struct MultiLine<'s, M, S> {
    config: &'s Config,
    core: Core<'s, M, S>,
    slice: &'s [u8],
    last_match: Option<Range>,
}

impl<'s, M: Matcher, S: Sink> MultiLine<'s, M, S> {
    pub(crate) fn new(
        searcher: &'s Searcher,
        matcher: M,
        slice: &'s [u8],
        write_to: S,
    ) -> MultiLine<'s, M, S> {
        debug_assert!(searcher.multi_line_with_matcher(&matcher));

        MultiLine {
            config: &searcher.config,
            core: Core::new(searcher, matcher, write_to, true),
            slice,
            last_match: None,
        }
    }

    pub(crate) fn run(mut self) -> Result<(), S::Error> {
        if self.core.begin()? {
            let binary_upto =
                std::cmp::min(self.slice.len(), DEFAULT_BUFFER_CAPACITY);
            let binary_range = Range::new(0, binary_upto);
            if !self.core.detect_binary(self.slice, &binary_range)? {
                let mut keepgoing = true;
				log::debug!("run(crate=(unprintable)): keepgoing → {}", keepgoing);
                while !self.slice[self.core.pos()..].is_empty() && keepgoing {
                    keepgoing = self.sink()?;
                }
                if keepgoing {
                    keepgoing = match self.last_match.take() {
                        None => true,
                        Some(last_match) => {
                            if self.sink_context(&last_match)? {
                                self.sink_matched(&last_match)?;
                            }
                            true
                        }
                    };
                }
                // Take care of any remaining context after the last match.
                if keepgoing {
                    if self.config.passthru {
                        self.core.other_context_by_line(
                            self.slice,
                            self.slice.len(),
                        )?;
                    } else {
                        self.core.after_context_by_line(
                            self.slice,
                            self.slice.len(),
                        )?;
                    }
                }
            }
        }
        let byte_count = self.byte_count();
		log::debug!("run(crate=(unprintable)): byte_count → {}", byte_count);
        let binary_byte_offset = self.core.binary_byte_offset();
        self.core.finish(byte_count, binary_byte_offset)
    }

    fn sink(&mut self) -> Result<bool, S::Error> {
		log::debug!(">>> Entering sink()");
        if self.config.invert_match {
            return self.sink_matched_inverted();
        }
        let mat = match self.find()? {
            Some(range) => range,
            None => {
                self.core.set_pos(self.slice.len());
				log::debug!("sink(): core → (unprintable)");
                return Ok(true);
            }
        };
        self.advance(&mat);

        let line =
            lines::locate(self.slice, self.config.line_term.as_byte(), mat);
        // We delay sinking the match to make sure we group adjacent matches
        // together in a single sink. Adjacent matches are distinct matches
        // that start and end on the same line, respectively. This guarantees
        // that a single line is never sinked more than once.
        match self.last_match.take() {
            None => {
                self.last_match = Some(line);
                Ok(true)
            }
            Some(last_match) => {
                // If the lines in the previous match overlap with the lines
                // in this match, then simply grow the match and move on. This
                // happens when the next match begins on the same line that the
                // last match ends on.
                //
                // Note that we do not technically require strict overlap here.
                // Instead, we only require that the lines are adjacent. This
                // provides larger blocks of lines to the printer, and results
                // in overall better behavior with respect to how replacements
                // are handled.
                //
                // See: https://github.com/BurntSushi/ripgrep/issues/1311
                // And also the associated commit fixing #1311.
                if last_match.end() >= line.start() {
                    self.last_match = Some(last_match.with_end(line.end()));
                    Ok(true)
                } else {
                    self.last_match = Some(line);
                    if !self.sink_context(&last_match)? {
                        return Ok(false);
                    }
                    self.sink_matched(&last_match)
                }
            }
		log::debug!("<<< Exiting sink()");
        }
    }

    fn sink_matched_inverted(&mut self) -> Result<bool, S::Error> {
		log::debug!(">>> Entering sink_matched_inverted()");
        assert!(self.config.invert_match);

        let invert_match = match self.find()? {
            None => {
                let range = Range::new(self.core.pos(), self.slice.len());
				log::debug!("sink_matched_inverted(): range → (unprintable)");
				log::debug!("sink_matched_inverted(): core → (unprintable)");
                self.core.set_pos(range.end());
                range
            }
            Some(mat) => {
                let line = lines::locate(
                    self.slice,
                    self.config.line_term.as_byte(),
                    mat,
                );
                let range = Range::new(self.core.pos(), line.start());
                self.advance(&line);
                range
            }
        };
        if invert_match.is_empty() {
            return Ok(true);
        }
        if !self.sink_context(&invert_match)? {
            return Ok(false);
        }
        let mut stepper = LineStep::new(
            self.config.line_term.as_byte(),
            invert_match.start(),
            invert_match.end(),
        );
        while let Some(line) = stepper.next_match(self.slice) {
            if !self.sink_matched(&line)? {
                return Ok(false);
            }
        }
        log::debug!("<<< Exiting sink_matched_inverted()");
        Ok(true)
    }

    fn sink_matched(&mut self, range: &Range) -> Result<bool, S::Error> {
		log::debug!(">>> Entering sink_matched(range={:?})", range);
        if range.is_empty() {
            // The only way we can produce an empty line for a match is if we
            // match the position immediately following the last byte that we
            // search, and where that last byte is also the line terminator. We
            // never want to report that match, and we know we're done at that
            // point anyway, so stop the search.
            return Ok(false);
        }
        self.core.matched(self.slice, range)
    }

    fn sink_context(&mut self, range: &Range) -> Result<bool, S::Error> {
		log::debug!(">>> Entering sink_context(range={:?})", range);
        if self.config.passthru {
            if !self.core.other_context_by_line(self.slice, range.start())? {
                return Ok(false);
            }
        } else {
            if !self.core.after_context_by_line(self.slice, range.start())? {
                return Ok(false);
            }
            if !self.core.before_context_by_line(self.slice, range.start())? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn find(&mut self) -> Result<Option<Range>, S::Error> {
		log::debug!(">>> Entering find()");
        log::debug!("<<< Exiting find()");
        self.core
            .find(&self.slice[self.core.pos()..])
            .map(|m| m.map(|m| m.offset(self.core.pos())))
    }

    /// Advance the search position based on the previous match.
    ///
    /// If the previous match is zero width, then this advances the search
    /// position one byte past the end of the match.
    fn advance(&mut self, range: &Range) {
		log::debug!(">>> Entering advance(range={:?})", range);
        self.core.set_pos(range.end());
        if range.is_empty() && self.core.pos() < self.slice.len() {
			log::debug!("advance(range={:?}): core → (unprintable)", range);
			log::debug!("advance(range={:?}): newpos → (unprintable)", range);
            let newpos = self.core.pos() + 1;
            self.core.set_pos(newpos);
		log::debug!("<<< Exiting advance(range={:?})", range);
        }
    }

    fn byte_count(&mut self) -> u64 {
        match self.core.binary_byte_offset() {
            Some(offset) if offset < self.core.pos() as u64 => offset,
            _ => self.core.pos() as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        searcher::{BinaryDetection, SearcherBuilder},
        testutil::{KitchenSink, RegexMatcher, SearcherTester},
    };

    use super::*;

    const SHERLOCK: &'static str = "\
For the Doctor Watsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
be, to a very large extent, the result of luck. Sherlock Holmes
can extract a clew from a wisp of straw or a flake of cigar ash;
but Doctor Watson has to have it taken out for him and dusted,
and exhibited clearly, with a label attached.\
";

    const CODE: &'static str = "\
extern crate snap;

use std::io;

fn main() {
	log::debug!(">>> Entering main()");
    let stdin = io::stdin();
    let stdout = io::stdout();
	log::debug!("main(): stdout → (unprintable)");
	log::debug!("main(): stdin → (unprintable)");

    // Wrap the stdin reader in a Snappy reader.
    let mut rdr = snap::Reader::new(stdin.lock());
	log::debug!("main(): rdr → (unprintable)");
	log::debug!("main(): wtr → (unprintable)");
    let mut wtr = stdout.lock();
	log::debug!("<<< Exiting main()");
    io::copy(&mut rdr, &mut wtr).expect(\"I/O operation failed\");
}
";

    #[test]
    fn basic1() {
		log::debug!(">>> Entering basic1()");
		log::debug!("<<< Exiting basic1()");
        let exp = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
129:be, to a very large extent, the result of luck. Sherlock Holmes

byte count:366
";
        SearcherTester::new(SHERLOCK, "Sherlock")
            .line_number(false)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn basic2() {
		log::debug!(">>> Entering basic2()");
        let exp = "\nbyte count:366\n";
		log::debug!("basic2(): exp → {}", exp);
        SearcherTester::new(SHERLOCK, "NADA")
            .line_number(false)
            .expected_no_line_number(exp)
			log::debug!("<<< Exiting basic2()");
            .test();
    }

    #[test]
    fn basic3() {
		log::debug!(">>> Entering basic3()");
		log::debug!("<<< Exiting basic3()");
        let exp = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
65:Holmeses, success in the province of detective work must always
129:be, to a very large extent, the result of luck. Sherlock Holmes
193:can extract a clew from a wisp of straw or a flake of cigar ash;
258:but Doctor Watson has to have it taken out for him and dusted,
321:and exhibited clearly, with a label attached.
byte count:366
";
        SearcherTester::new(SHERLOCK, "a")
            .line_number(false)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn basic4() {
		log::debug!(">>> Entering basic4()");
		log::debug!("<<< Exiting basic4()");
        let haystack = "\
a
b

c


d
";
        let byte_count = haystack.len();
		log::debug!("basic4(): byte_count → {}", byte_count);
        let exp = format!("0:a\n\nbyte count:{}\n", byte_count);
		log::debug!("basic4(): exp → {}", exp);
        SearcherTester::new(haystack, "a")
            .line_number(false)
            .expected_no_line_number(&exp)
            .test();
    }

    #[test]
    fn invert1() {
		log::debug!(">>> Entering invert1()");
		log::debug!("<<< Exiting invert1()");
        let exp = "\
65:Holmeses, success in the province of detective work must always
193:can extract a clew from a wisp of straw or a flake of cigar ash;
258:but Doctor Watson has to have it taken out for him and dusted,
321:and exhibited clearly, with a label attached.
byte count:366
";
        SearcherTester::new(SHERLOCK, "Sherlock")
            .line_number(false)
            .invert_match(true)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn line_number1() {
		log::debug!(">>> Entering line_number1()");
		log::debug!("<<< Exiting line_number1()");
        let exp = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
129:be, to a very large extent, the result of luck. Sherlock Holmes

byte count:366
";
        let exp_line = "\
1:0:For the Doctor Watsons of this world, as opposed to the Sherlock
3:129:be, to a very large extent, the result of luck. Sherlock Holmes

byte count:366
";
        SearcherTester::new(SHERLOCK, "Sherlock")
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_line)
            .test();
    }

    #[test]
    fn line_number_invert1() {
		log::debug!(">>> Entering line_number_invert1()");
		log::debug!("<<< Exiting line_number_invert1()");
        let exp = "\
65:Holmeses, success in the province of detective work must always
193:can extract a clew from a wisp of straw or a flake of cigar ash;
258:but Doctor Watson has to have it taken out for him and dusted,
321:and exhibited clearly, with a label attached.
byte count:366
";
        let exp_line = "\
2:65:Holmeses, success in the province of detective work must always
4:193:can extract a clew from a wisp of straw or a flake of cigar ash;
5:258:but Doctor Watson has to have it taken out for him and dusted,
6:321:and exhibited clearly, with a label attached.
byte count:366
";
        SearcherTester::new(SHERLOCK, "Sherlock")
            .invert_match(true)
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_line)
            .test();
    }

    #[test]
    fn multi_line_overlap1() {
		log::debug!(">>> Entering multi_line_overlap1()");
        let haystack = "xxx\nabc\ndefxxxabc\ndefxxx\nxxx";
        let byte_count = haystack.len();
		log::debug!("multi_line_overlap1(): byte_count → {}", byte_count);
        log::debug!("multi_line_overlap1(): haystack → {}", haystack);
        let exp = format!(
            "4:abc\n8:defxxxabc\n18:defxxx\n\nbyte count:{}\n",
            byte_count
        );

        SearcherTester::new(haystack, "abc\ndef")
            .by_line(false)
            .line_number(false)
            .expected_no_line_number(&exp)
			log::debug!("<<< Exiting multi_line_overlap1()");
            .test();
    }

    #[test]
    fn multi_line_overlap2() {
		log::debug!(">>> Entering multi_line_overlap2()");
        let haystack = "xxx\nabc\ndefabc\ndefxxx\nxxx";
        let byte_count = haystack.len();
		log::debug!("multi_line_overlap2(): byte_count → {}", byte_count);
        log::debug!("multi_line_overlap2(): haystack → {}", haystack);
        let exp = format!(
            "4:abc\n8:defabc\n15:defxxx\n\nbyte count:{}\n",
            byte_count
        );

        SearcherTester::new(haystack, "abc\ndef")
            .by_line(false)
            .line_number(false)
            .expected_no_line_number(&exp)
			log::debug!("<<< Exiting multi_line_overlap2()");
            .test();
    }

    #[test]
    fn empty_line1() {
		log::debug!(">>> Entering empty_line1()");
        let exp = "\nbyte count:0\n";
		log::debug!("empty_line1(): exp → {}", exp);
        SearcherTester::new("", r"^$")
            .expected_no_line_number(exp)
            .expected_with_line_number(exp)
			log::debug!("<<< Exiting empty_line1()");
            .test();
    }

    #[test]
    fn empty_line2() {
		log::debug!(">>> Entering empty_line2()");
        let exp = "0:\n\nbyte count:1\n";
        let exp_line = "1:0:\n\nbyte count:1\n";
		log::debug!("empty_line2(): exp_line → {}", exp_line);
		log::debug!("empty_line2(): exp → {}", exp);

        SearcherTester::new("\n", r"^$")
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_line)
			log::debug!("<<< Exiting empty_line2()");
            .test();
    }

    #[test]
    fn empty_line3() {
		log::debug!(">>> Entering empty_line3()");
        let exp = "0:\n1:\n\nbyte count:2\n";
        let exp_line = "1:0:\n2:1:\n\nbyte count:2\n";
		log::debug!("empty_line3(): exp_line → {}", exp_line);
		log::debug!("empty_line3(): exp → {}", exp);

        SearcherTester::new("\n\n", r"^$")
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_line)
			log::debug!("<<< Exiting empty_line3()");
            .test();
    }

    #[test]
    fn empty_line4() {
		log::debug!(">>> Entering empty_line4()");
        // See: https://github.com/BurntSushi/ripgrep/issues/441
		log::debug!("<<< Exiting empty_line4()");
        let haystack = "\
a
b

c


d
";
        let byte_count = haystack.len();
		log::debug!("empty_line4(): byte_count → {}", byte_count);
        let exp = format!("4:\n7:\n8:\n\nbyte count:{}\n", byte_count);
		log::debug!("empty_line4(): exp → {}", exp);
        let exp_line =
            format!("3:4:\n5:7:\n6:8:\n\nbyte count:{}\n", byte_count);

        SearcherTester::new(haystack, r"^$")
            .expected_no_line_number(&exp)
            .expected_with_line_number(&exp_line)
            .test();
    }

    #[test]
    fn empty_line5() {
		log::debug!(">>> Entering empty_line5()");
        // See: https://github.com/BurntSushi/ripgrep/issues/441
        // This is like empty_line4, but lacks the trailing line terminator.
		log::debug!("<<< Exiting empty_line5()");
        let haystack = "\
a
b

c


d";
        let byte_count = haystack.len();
		log::debug!("empty_line5(): byte_count → {}", byte_count);
        let exp = format!("4:\n7:\n8:\n\nbyte count:{}\n", byte_count);
		log::debug!("empty_line5(): exp → {}", exp);
        let exp_line =
            format!("3:4:\n5:7:\n6:8:\n\nbyte count:{}\n", byte_count);

        SearcherTester::new(haystack, r"^$")
            .expected_no_line_number(&exp)
            .expected_with_line_number(&exp_line)
            .test();
    }

    #[test]
    fn empty_line6() {
		log::debug!(">>> Entering empty_line6()");
        // See: https://github.com/BurntSushi/ripgrep/issues/441
        // This is like empty_line4, but includes an empty line at the end.
		log::debug!("<<< Exiting empty_line6()");
        let haystack = "\
a
b

c


d

";
        let byte_count = haystack.len();
		log::debug!("empty_line6(): byte_count → {}", byte_count);
        let exp = format!("4:\n7:\n8:\n11:\n\nbyte count:{}\n", byte_count);
		log::debug!("empty_line6(): exp → {}", exp);
        let exp_line =
            format!("3:4:\n5:7:\n6:8:\n8:11:\n\nbyte count:{}\n", byte_count);

        SearcherTester::new(haystack, r"^$")
            .expected_no_line_number(&exp)
            .expected_with_line_number(&exp_line)
            .test();
    }

    #[test]
    fn big1() {
		log::debug!(">>> Entering big1()");
        let mut haystack = String::new();
        haystack.push_str("a\n");
		log::debug!("big1(): haystack → {}", haystack);
        // Pick an arbitrary number above the capacity.
        for _ in 0..(4 * (DEFAULT_BUFFER_CAPACITY + 7)) {
            haystack.push_str("zzz\n");
        }
        haystack.push_str("a\n");

        let byte_count = haystack.len();
		log::debug!("big1(): byte_count → {}", byte_count);
        let exp = format!("0:a\n1048690:a\n\nbyte count:{}\n", byte_count);
		log::debug!("big1(): exp → {}", exp);

        SearcherTester::new(&haystack, "a")
            .line_number(false)
            .expected_no_line_number(&exp)
			log::debug!("<<< Exiting big1()");
            .test();
    }

    #[test]
    fn big_error_one_line() {
		log::debug!(">>> Entering big_error_one_line()");
        let mut haystack = String::new();
        haystack.push_str("a\n");
		log::debug!("big_error_one_line(): haystack → {}", haystack);
        // Pick an arbitrary number above the capacity.
        for _ in 0..(4 * (DEFAULT_BUFFER_CAPACITY + 7)) {
            haystack.push_str("zzz\n");
        }
        haystack.push_str("a\n");

        let matcher = RegexMatcher::new("a");
		log::debug!("big_error_one_line(): matcher → (unprintable)");
		log::debug!("big_error_one_line(): sink → (unprintable)");
        let mut sink = KitchenSink::new();
        let mut searcher = SearcherBuilder::new()
            .heap_limit(Some(3)) // max line length is 4, one byte short
            .build();
        let result =
            searcher.search_reader(&matcher, haystack.as_bytes(), &mut sink);
		log::debug!("<<< Exiting big_error_one_line()");
        assert!(result.is_err());
    }

    #[test]
    fn big_error_multi_line() {
		log::debug!(">>> Entering big_error_multi_line()");
        let mut haystack = String::new();
        haystack.push_str("a\n");
		log::debug!("big_error_multi_line(): haystack → {}", haystack);
        // Pick an arbitrary number above the capacity.
        for _ in 0..(4 * (DEFAULT_BUFFER_CAPACITY + 7)) {
            haystack.push_str("zzz\n");
        }
        haystack.push_str("a\n");

        let matcher = RegexMatcher::new("a");
		log::debug!("big_error_multi_line(): matcher → (unprintable)");
		log::debug!("big_error_multi_line(): sink → (unprintable)");
        let mut sink = KitchenSink::new();
        let mut searcher = SearcherBuilder::new()
            .multi_line(true)
            .heap_limit(Some(haystack.len())) // actually need one more byte
            .build();
        let result =
            searcher.search_reader(&matcher, haystack.as_bytes(), &mut sink);
		log::debug!("<<< Exiting big_error_multi_line()");
        assert!(result.is_err());
    }

    #[test]
    fn binary1() {
		log::debug!(">>> Entering binary1()");
        let haystack = "\x00a";
        let exp = "\nbyte count:0\nbinary offset:0\n";
		log::debug!("binary1(): exp → {}", exp);
		log::debug!("binary1(): haystack → {}", haystack);

        SearcherTester::new(haystack, "a")
            .binary_detection(BinaryDetection::quit(0))
            .line_number(false)
            .expected_no_line_number(exp)
			log::debug!("<<< Exiting binary1()");
            .test();
    }

    #[test]
    fn binary2() {
		log::debug!(">>> Entering binary2()");
        let haystack = "a\x00";
        let exp = "\nbyte count:0\nbinary offset:1\n";
		log::debug!("binary2(): exp → {}", exp);
		log::debug!("binary2(): haystack → {}", haystack);

        SearcherTester::new(haystack, "a")
            .binary_detection(BinaryDetection::quit(0))
            .line_number(false)
            .expected_no_line_number(exp)
			log::debug!("<<< Exiting binary2()");
            .test();
    }

    #[test]
    fn binary3() {
		log::debug!(">>> Entering binary3()");
        let mut haystack = String::new();
        haystack.push_str("a\n");
		log::debug!("binary3(): haystack → {}", haystack);
        for _ in 0..DEFAULT_BUFFER_CAPACITY {
            haystack.push_str("zzz\n");
        }
        haystack.push_str("a\n");
        haystack.push_str("zzz\n");
        haystack.push_str("a\x00a\n");
        haystack.push_str("zzz\n");
        haystack.push_str("a\n");

        // The line buffered searcher has slightly different semantics here.
        // Namely, it will *always* detect binary data in the current buffer
        // before searching it. Thus, the total number of bytes searched is
        // smaller than below.
        let exp = "0:a\n\nbyte count:262146\nbinary offset:262153\n";
		log::debug!("binary3(): exp → {}", exp);
        // In contrast, the slice readers (for multi line as well) will only
        // look for binary data in the initial chunk of bytes. After that
        // point, it only looks for binary data in matches. Note though that
        // the binary offset remains the same. (See the binary4 test for a case
        // where the offset is explicitly different.)
        let exp_slice =
            "0:a\n262146:a\n\nbyte count:262153\nbinary offset:262153\n";

        SearcherTester::new(&haystack, "a")
            .binary_detection(BinaryDetection::quit(0))
            .line_number(false)
            .auto_heap_limit(false)
            .expected_no_line_number(exp)
            .expected_slice_no_line_number(exp_slice)
			log::debug!("<<< Exiting binary3()");
            .test();
    }

    #[test]
    fn binary4() {
		log::debug!(">>> Entering binary4()");
        let mut haystack = String::new();
        haystack.push_str("a\n");
		log::debug!("binary4(): haystack → {}", haystack);
        for _ in 0..DEFAULT_BUFFER_CAPACITY {
            haystack.push_str("zzz\n");
        }
        haystack.push_str("a\n");
        // The Read searcher will detect binary data here, but since this is
        // beyond the initial buffer size and doesn't otherwise contain a
        // match, the Slice reader won't detect the binary data until the next
        // line (which is a match).
        haystack.push_str("b\x00b\n");
        haystack.push_str("a\x00a\n");
        haystack.push_str("a\n");

        let exp = "0:a\n\nbyte count:262146\nbinary offset:262149\n";
		log::debug!("binary4(): exp → {}", exp);
        // The binary offset for the Slice readers corresponds to the binary
        // data in `a\x00a\n` since the first line with binary data
        // (`b\x00b\n`) isn't part of a match, and is therefore undetected.
        let exp_slice =
            "0:a\n262146:a\n\nbyte count:262153\nbinary offset:262153\n";

        SearcherTester::new(&haystack, "a")
            .binary_detection(BinaryDetection::quit(0))
            .line_number(false)
            .auto_heap_limit(false)
            .expected_no_line_number(exp)
            .expected_slice_no_line_number(exp_slice)
			log::debug!("<<< Exiting binary4()");
            .test();
    }

    #[test]
    fn passthru_sherlock1() {
		log::debug!(">>> Entering passthru_sherlock1()");
		log::debug!("<<< Exiting passthru_sherlock1()");
        let exp = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
65-Holmeses, success in the province of detective work must always
129:be, to a very large extent, the result of luck. Sherlock Holmes
193-can extract a clew from a wisp of straw or a flake of cigar ash;
258-but Doctor Watson has to have it taken out for him and dusted,
321-and exhibited clearly, with a label attached.
byte count:366
";
        SearcherTester::new(SHERLOCK, "Sherlock")
            .passthru(true)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn passthru_sherlock_invert1() {
		log::debug!(">>> Entering passthru_sherlock_invert1()");
		log::debug!("<<< Exiting passthru_sherlock_invert1()");
        let exp = "\
0-For the Doctor Watsons of this world, as opposed to the Sherlock
65:Holmeses, success in the province of detective work must always
129-be, to a very large extent, the result of luck. Sherlock Holmes
193:can extract a clew from a wisp of straw or a flake of cigar ash;
258:but Doctor Watson has to have it taken out for him and dusted,
321:and exhibited clearly, with a label attached.
byte count:366
";
        SearcherTester::new(SHERLOCK, "Sherlock")
            .passthru(true)
            .line_number(false)
            .invert_match(true)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn context_sherlock1() {
		log::debug!(">>> Entering context_sherlock1()");
		log::debug!("<<< Exiting context_sherlock1()");
        let exp = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
65-Holmeses, success in the province of detective work must always
129:be, to a very large extent, the result of luck. Sherlock Holmes
193-can extract a clew from a wisp of straw or a flake of cigar ash;

byte count:366
";
        let exp_lines = "\
1:0:For the Doctor Watsons of this world, as opposed to the Sherlock
2-65-Holmeses, success in the province of detective work must always
3:129:be, to a very large extent, the result of luck. Sherlock Holmes
4-193-can extract a clew from a wisp of straw or a flake of cigar ash;

byte count:366
";
        // before and after + line numbers
        SearcherTester::new(SHERLOCK, "Sherlock")
            .after_context(1)
            .before_context(1)
            .line_number(true)
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_lines)
            .test();

        // after
        SearcherTester::new(SHERLOCK, "Sherlock")
            .after_context(1)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();

        // before
        let exp = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
65-Holmeses, success in the province of detective work must always
129:be, to a very large extent, the result of luck. Sherlock Holmes

byte count:366
";
        SearcherTester::new(SHERLOCK, "Sherlock")
            .before_context(1)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn context_sherlock_invert1() {
		log::debug!(">>> Entering context_sherlock_invert1()");
		log::debug!("<<< Exiting context_sherlock_invert1()");
        let exp = "\
0-For the Doctor Watsons of this world, as opposed to the Sherlock
65:Holmeses, success in the province of detective work must always
129-be, to a very large extent, the result of luck. Sherlock Holmes
193:can extract a clew from a wisp of straw or a flake of cigar ash;
258:but Doctor Watson has to have it taken out for him and dusted,
321:and exhibited clearly, with a label attached.
byte count:366
";
        let exp_lines = "\
1-0-For the Doctor Watsons of this world, as opposed to the Sherlock
2:65:Holmeses, success in the province of detective work must always
3-129-be, to a very large extent, the result of luck. Sherlock Holmes
4:193:can extract a clew from a wisp of straw or a flake of cigar ash;
5:258:but Doctor Watson has to have it taken out for him and dusted,
6:321:and exhibited clearly, with a label attached.
byte count:366
";
        // before and after + line numbers
        SearcherTester::new(SHERLOCK, "Sherlock")
            .after_context(1)
            .before_context(1)
            .line_number(true)
            .invert_match(true)
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_lines)
            .test();

        // before
        SearcherTester::new(SHERLOCK, "Sherlock")
            .before_context(1)
            .line_number(false)
            .invert_match(true)
            .expected_no_line_number(exp)
            .test();

        // after
        let exp = "\
65:Holmeses, success in the province of detective work must always
129-be, to a very large extent, the result of luck. Sherlock Holmes
193:can extract a clew from a wisp of straw or a flake of cigar ash;
258:but Doctor Watson has to have it taken out for him and dusted,
321:and exhibited clearly, with a label attached.
byte count:366
";
        SearcherTester::new(SHERLOCK, "Sherlock")
            .after_context(1)
            .line_number(false)
            .invert_match(true)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn context_sherlock2() {
		log::debug!(">>> Entering context_sherlock2()");
		log::debug!("<<< Exiting context_sherlock2()");
        let exp = "\
65-Holmeses, success in the province of detective work must always
129:be, to a very large extent, the result of luck. Sherlock Holmes
193:can extract a clew from a wisp of straw or a flake of cigar ash;
258-but Doctor Watson has to have it taken out for him and dusted,
321:and exhibited clearly, with a label attached.
byte count:366
";
        let exp_lines = "\
2-65-Holmeses, success in the province of detective work must always
3:129:be, to a very large extent, the result of luck. Sherlock Holmes
4:193:can extract a clew from a wisp of straw or a flake of cigar ash;
5-258-but Doctor Watson has to have it taken out for him and dusted,
6:321:and exhibited clearly, with a label attached.
byte count:366
";
        // before + after + line numbers
        SearcherTester::new(SHERLOCK, " a ")
            .after_context(1)
            .before_context(1)
            .line_number(true)
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_lines)
            .test();

        // before
        SearcherTester::new(SHERLOCK, " a ")
            .before_context(1)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();

        // after
        let exp = "\
129:be, to a very large extent, the result of luck. Sherlock Holmes
193:can extract a clew from a wisp of straw or a flake of cigar ash;
258-but Doctor Watson has to have it taken out for him and dusted,
321:and exhibited clearly, with a label attached.
byte count:366
";
        SearcherTester::new(SHERLOCK, " a ")
            .after_context(1)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn context_sherlock_invert2() {
		log::debug!(">>> Entering context_sherlock_invert2()");
		log::debug!("<<< Exiting context_sherlock_invert2()");
        let exp = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
65:Holmeses, success in the province of detective work must always
129-be, to a very large extent, the result of luck. Sherlock Holmes
193-can extract a clew from a wisp of straw or a flake of cigar ash;
258:but Doctor Watson has to have it taken out for him and dusted,
321-and exhibited clearly, with a label attached.
byte count:366
";
        let exp_lines = "\
1:0:For the Doctor Watsons of this world, as opposed to the Sherlock
2:65:Holmeses, success in the province of detective work must always
3-129-be, to a very large extent, the result of luck. Sherlock Holmes
4-193-can extract a clew from a wisp of straw or a flake of cigar ash;
5:258:but Doctor Watson has to have it taken out for him and dusted,
6-321-and exhibited clearly, with a label attached.
byte count:366
";
        // before + after + line numbers
        SearcherTester::new(SHERLOCK, " a ")
            .after_context(1)
            .before_context(1)
            .line_number(true)
            .invert_match(true)
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_lines)
            .test();

        // before
        let exp = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
65:Holmeses, success in the province of detective work must always
--
193-can extract a clew from a wisp of straw or a flake of cigar ash;
258:but Doctor Watson has to have it taken out for him and dusted,

byte count:366
";
        SearcherTester::new(SHERLOCK, " a ")
            .before_context(1)
            .line_number(false)
            .invert_match(true)
            .expected_no_line_number(exp)
            .test();

        // after
        let exp = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
65:Holmeses, success in the province of detective work must always
129-be, to a very large extent, the result of luck. Sherlock Holmes
--
258:but Doctor Watson has to have it taken out for him and dusted,
321-and exhibited clearly, with a label attached.
byte count:366
";
        SearcherTester::new(SHERLOCK, " a ")
            .after_context(1)
            .line_number(false)
            .invert_match(true)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn context_sherlock3() {
		log::debug!(">>> Entering context_sherlock3()");
		log::debug!("<<< Exiting context_sherlock3()");
        let exp = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
65-Holmeses, success in the province of detective work must always
129:be, to a very large extent, the result of luck. Sherlock Holmes
193-can extract a clew from a wisp of straw or a flake of cigar ash;
258-but Doctor Watson has to have it taken out for him and dusted,

byte count:366
";
        let exp_lines = "\
1:0:For the Doctor Watsons of this world, as opposed to the Sherlock
2-65-Holmeses, success in the province of detective work must always
3:129:be, to a very large extent, the result of luck. Sherlock Holmes
4-193-can extract a clew from a wisp of straw or a flake of cigar ash;
5-258-but Doctor Watson has to have it taken out for him and dusted,

byte count:366
";
        // before and after + line numbers
        SearcherTester::new(SHERLOCK, "Sherlock")
            .after_context(2)
            .before_context(2)
            .line_number(true)
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_lines)
            .test();

        // after
        SearcherTester::new(SHERLOCK, "Sherlock")
            .after_context(2)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();

        // before
        let exp = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
65-Holmeses, success in the province of detective work must always
129:be, to a very large extent, the result of luck. Sherlock Holmes

byte count:366
";
        SearcherTester::new(SHERLOCK, "Sherlock")
            .before_context(2)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn context_sherlock4() {
		log::debug!(">>> Entering context_sherlock4()");
		log::debug!("<<< Exiting context_sherlock4()");
        let exp = "\
129-be, to a very large extent, the result of luck. Sherlock Holmes
193-can extract a clew from a wisp of straw or a flake of cigar ash;
258:but Doctor Watson has to have it taken out for him and dusted,
321-and exhibited clearly, with a label attached.
byte count:366
";
        let exp_lines = "\
3-129-be, to a very large extent, the result of luck. Sherlock Holmes
4-193-can extract a clew from a wisp of straw or a flake of cigar ash;
5:258:but Doctor Watson has to have it taken out for him and dusted,
6-321-and exhibited clearly, with a label attached.
byte count:366
";
        // before and after + line numbers
        SearcherTester::new(SHERLOCK, "dusted")
            .after_context(2)
            .before_context(2)
            .line_number(true)
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_lines)
            .test();

        // after
        let exp = "\
258:but Doctor Watson has to have it taken out for him and dusted,
321-and exhibited clearly, with a label attached.
byte count:366
";
        SearcherTester::new(SHERLOCK, "dusted")
            .after_context(2)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();

        // before
        let exp = "\
129-be, to a very large extent, the result of luck. Sherlock Holmes
193-can extract a clew from a wisp of straw or a flake of cigar ash;
258:but Doctor Watson has to have it taken out for him and dusted,

byte count:366
";
        SearcherTester::new(SHERLOCK, "dusted")
            .before_context(2)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn context_sherlock5() {
		log::debug!(">>> Entering context_sherlock5()");
		log::debug!("<<< Exiting context_sherlock5()");
        let exp = "\
0-For the Doctor Watsons of this world, as opposed to the Sherlock
65:Holmeses, success in the province of detective work must always
129-be, to a very large extent, the result of luck. Sherlock Holmes
193-can extract a clew from a wisp of straw or a flake of cigar ash;
258-but Doctor Watson has to have it taken out for him and dusted,
321:and exhibited clearly, with a label attached.
byte count:366
";
        let exp_lines = "\
1-0-For the Doctor Watsons of this world, as opposed to the Sherlock
2:65:Holmeses, success in the province of detective work must always
3-129-be, to a very large extent, the result of luck. Sherlock Holmes
4-193-can extract a clew from a wisp of straw or a flake of cigar ash;
5-258-but Doctor Watson has to have it taken out for him and dusted,
6:321:and exhibited clearly, with a label attached.
byte count:366
";
        // before and after + line numbers
        SearcherTester::new(SHERLOCK, "success|attached")
            .after_context(2)
            .before_context(2)
            .line_number(true)
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_lines)
            .test();

        // after
        let exp = "\
65:Holmeses, success in the province of detective work must always
129-be, to a very large extent, the result of luck. Sherlock Holmes
193-can extract a clew from a wisp of straw or a flake of cigar ash;
--
321:and exhibited clearly, with a label attached.
byte count:366
";
        SearcherTester::new(SHERLOCK, "success|attached")
            .after_context(2)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();

        // before
        let exp = "\
0-For the Doctor Watsons of this world, as opposed to the Sherlock
65:Holmeses, success in the province of detective work must always
--
193-can extract a clew from a wisp of straw or a flake of cigar ash;
258-but Doctor Watson has to have it taken out for him and dusted,
321:and exhibited clearly, with a label attached.
byte count:366
";
        SearcherTester::new(SHERLOCK, "success|attached")
            .before_context(2)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn context_sherlock6() {
		log::debug!(">>> Entering context_sherlock6()");
		log::debug!("<<< Exiting context_sherlock6()");
        let exp = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
65-Holmeses, success in the province of detective work must always
129:be, to a very large extent, the result of luck. Sherlock Holmes
193-can extract a clew from a wisp of straw or a flake of cigar ash;
258-but Doctor Watson has to have it taken out for him and dusted,
321-and exhibited clearly, with a label attached.
byte count:366
";
        let exp_lines = "\
1:0:For the Doctor Watsons of this world, as opposed to the Sherlock
2-65-Holmeses, success in the province of detective work must always
3:129:be, to a very large extent, the result of luck. Sherlock Holmes
4-193-can extract a clew from a wisp of straw or a flake of cigar ash;
5-258-but Doctor Watson has to have it taken out for him and dusted,
6-321-and exhibited clearly, with a label attached.
byte count:366
";
        // before and after + line numbers
        SearcherTester::new(SHERLOCK, "Sherlock")
            .after_context(3)
            .before_context(3)
            .line_number(true)
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_lines)
            .test();

        // after
        let exp = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
65-Holmeses, success in the province of detective work must always
129:be, to a very large extent, the result of luck. Sherlock Holmes
193-can extract a clew from a wisp of straw or a flake of cigar ash;
258-but Doctor Watson has to have it taken out for him and dusted,
321-and exhibited clearly, with a label attached.
byte count:366
";
        SearcherTester::new(SHERLOCK, "Sherlock")
            .after_context(3)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();

        // before
        let exp = "\
0:For the Doctor Watsons of this world, as opposed to the Sherlock
65-Holmeses, success in the province of detective work must always
129:be, to a very large extent, the result of luck. Sherlock Holmes

byte count:366
";
        SearcherTester::new(SHERLOCK, "Sherlock")
            .before_context(3)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn context_code1() {
		log::debug!(">>> Entering context_code1()");
        // before and after
		log::debug!("<<< Exiting context_code1()");
        let exp = "\
33-
34-fn main() {
46:    let stdin = io::stdin();
log::debug!("context_code1(): stdin → (unprintable)");
log::debug!("context_code1(): stdout → (unprintable)");
75-    let stdout = io::stdout();
106-
107:    // Wrap the stdin reader in a Snappy reader.
156:    let mut rdr = snap::Reader::new(stdin.lock());
log::debug!("context_code1(): rdr → (unprintable)");
log::debug!("context_code1(): wtr → (unprintable)");
207-    let mut wtr = stdout.lock();
240-    io::copy(&mut rdr, &mut wtr).expect(\"I/O operation failed\");

byte count:307
";
        let exp_lines = "\
4-33-
5-34-fn main() {
6:46:    let stdin = io::stdin();
7-75-    let stdout = io::stdout();
8-106-
9:107:    // Wrap the stdin reader in a Snappy reader.
10:156:    let mut rdr = snap::Reader::new(stdin.lock());
11-207-    let mut wtr = stdout.lock();
12-240-    io::copy(&mut rdr, &mut wtr).expect(\"I/O operation failed\");

byte count:307
";
        // before and after + line numbers
        SearcherTester::new(CODE, "stdin")
            .after_context(2)
            .before_context(2)
            .line_number(true)
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_lines)
            .test();

        // after
        let exp = "\
46:    let stdin = io::stdin();
75-    let stdout = io::stdout();
106-
107:    // Wrap the stdin reader in a Snappy reader.
156:    let mut rdr = snap::Reader::new(stdin.lock());
207-    let mut wtr = stdout.lock();
240-    io::copy(&mut rdr, &mut wtr).expect(\"I/O operation failed\");

byte count:307
";
        SearcherTester::new(CODE, "stdin")
            .after_context(2)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();

        // before
        let exp = "\
33-
34-fn main() {
46:    let stdin = io::stdin();
75-    let stdout = io::stdout();
106-
107:    // Wrap the stdin reader in a Snappy reader.
156:    let mut rdr = snap::Reader::new(stdin.lock());

byte count:307
";
        SearcherTester::new(CODE, "stdin")
            .before_context(2)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn context_code2() {
		log::debug!(">>> Entering context_code2()");
		log::debug!("<<< Exiting context_code2()");
        let exp = "\
34-fn main() {
46-    let stdin = io::stdin();
log::debug!("context_code2(): stdin → (unprintable)");
log::debug!("context_code2(): stdout → (unprintable)");
75:    let stdout = io::stdout();
106-
107-    // Wrap the stdin reader in a Snappy reader.
156-    let mut rdr = snap::Reader::new(stdin.lock());
log::debug!("context_code2(): rdr → (unprintable)");
log::debug!("context_code2(): wtr → (unprintable)");
207:    let mut wtr = stdout.lock();
240-    io::copy(&mut rdr, &mut wtr).expect(\"I/O operation failed\");
305-}

byte count:307
";
        let exp_lines = "\
5-34-fn main() {
6-46-    let stdin = io::stdin();
7:75:    let stdout = io::stdout();
8-106-
9-107-    // Wrap the stdin reader in a Snappy reader.
10-156-    let mut rdr = snap::Reader::new(stdin.lock());
11:207:    let mut wtr = stdout.lock();
12-240-    io::copy(&mut rdr, &mut wtr).expect(\"I/O operation failed\");
13-305-}

byte count:307
";
        // before and after + line numbers
        SearcherTester::new(CODE, "stdout")
            .after_context(2)
            .before_context(2)
            .line_number(true)
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_lines)
            .test();

        // after
        let exp = "\
75:    let stdout = io::stdout();
106-
107-    // Wrap the stdin reader in a Snappy reader.
--
207:    let mut wtr = stdout.lock();
240-    io::copy(&mut rdr, &mut wtr).expect(\"I/O operation failed\");
305-}

byte count:307
";
        SearcherTester::new(CODE, "stdout")
            .after_context(2)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();

        // before
        let exp = "\
34-fn main() {
46-    let stdin = io::stdin();
75:    let stdout = io::stdout();
--
107-    // Wrap the stdin reader in a Snappy reader.
156-    let mut rdr = snap::Reader::new(stdin.lock());
207:    let mut wtr = stdout.lock();

byte count:307
";
        SearcherTester::new(CODE, "stdout")
            .before_context(2)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn context_code3() {
		log::debug!(">>> Entering context_code3()");
		log::debug!("<<< Exiting context_code3()");
        let exp = "\
20-use std::io;
33-
34:fn main() {
46-    let stdin = io::stdin();
log::debug!("context_code3(): stdin → (unprintable)");
log::debug!("context_code3(): stdout → (unprintable)");
75-    let stdout = io::stdout();
106-
107-    // Wrap the stdin reader in a Snappy reader.
156:    let mut rdr = snap::Reader::new(stdin.lock());
log::debug!("context_code3(): rdr → (unprintable)");
log::debug!("context_code3(): wtr → (unprintable)");
207-    let mut wtr = stdout.lock();
240-    io::copy(&mut rdr, &mut wtr).expect(\"I/O operation failed\");

byte count:307
";
        let exp_lines = "\
3-20-use std::io;
4-33-
5:34:fn main() {
6-46-    let stdin = io::stdin();
7-75-    let stdout = io::stdout();
8-106-
9-107-    // Wrap the stdin reader in a Snappy reader.
10:156:    let mut rdr = snap::Reader::new(stdin.lock());
11-207-    let mut wtr = stdout.lock();
12-240-    io::copy(&mut rdr, &mut wtr).expect(\"I/O operation failed\");

byte count:307
";
        // before and after + line numbers
        SearcherTester::new(CODE, "fn main|let mut rdr")
            .after_context(2)
            .before_context(2)
            .line_number(true)
            .expected_no_line_number(exp)
            .expected_with_line_number(exp_lines)
            .test();

        // after
        let exp = "\
34:fn main() {
46-    let stdin = io::stdin();
75-    let stdout = io::stdout();
--
156:    let mut rdr = snap::Reader::new(stdin.lock());
207-    let mut wtr = stdout.lock();
240-    io::copy(&mut rdr, &mut wtr).expect(\"I/O operation failed\");

byte count:307
";
        SearcherTester::new(CODE, "fn main|let mut rdr")
            .after_context(2)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();

        // before
        let exp = "\
20-use std::io;
33-
34:fn main() {
--
106-
107-    // Wrap the stdin reader in a Snappy reader.
156:    let mut rdr = snap::Reader::new(stdin.lock());

byte count:307
";
        SearcherTester::new(CODE, "fn main|let mut rdr")
            .before_context(2)
            .line_number(false)
            .expected_no_line_number(exp)
            .test();
    }

    #[test]
    fn scratch() {
		log::debug!(">>> Entering scratch()");
        use crate::sinks;
        use crate::testutil::RegexMatcher;

		log::debug!("scratch(): matcher → (unprintable)");
		log::debug!("scratch(): searcher → (unprintable)");
		log::debug!("<<< Exiting scratch()");
        const SHERLOCK: &'static [u8] = b"\
For the Doctor Wat\xFFsons of this world, as opposed to the Sherlock
Holmeses, success in the province of detective work must always
be, to a very large extent, the result of luck. Sherlock Holmes
can extract a clew from a wisp of straw or a flake of cigar ash;
but Doctor Watson has to have it taken out for him and dusted,
and exhibited clearly, with a label attached.\
    ";

        let haystack = SHERLOCK;
        let matcher = RegexMatcher::new("Sherlock");
        let mut searcher = SearcherBuilder::new().line_number(true).build();
        searcher
            .search_reader(
                &matcher,
                haystack,
                sinks::Lossy(|n, line| {
                    print!("{}:{}", n, line);
                    Ok(true)
                }),
            )
            .unwrap();
    }

    // See: https://github.com/BurntSushi/ripgrep/issues/2260
    #[test]
    fn regression_2260() {
		log::debug!(">>> Entering regression_2260()");
        use grep_regex::RegexMatcherBuilder;

        use crate::SearcherBuilder;

        let matcher = RegexMatcherBuilder::new()
            .line_terminator(Some(b'\n'))
            .build(r"^\w+$")
            .unwrap();
        let mut searcher = SearcherBuilder::new().line_number(true).build();
		log::debug!("regression_2260(): searcher → (unprintable)");

        let mut matched = false;
		log::debug!("regression_2260(): matched → {}", matched);
        searcher
            .search_slice(
                &matcher,
                b"GATC\n",
                crate::sinks::UTF8(|_, _| {
                    matched = true;
                    Ok(true)
                }),
            )
            .unwrap();
		log::debug!("<<< Exiting regression_2260()");
        assert!(matched);
    }
}
