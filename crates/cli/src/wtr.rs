use std::io::{self, IsTerminal};

use termcolor::HyperlinkSpec;

/// A writer that supports coloring with either line or block buffering.
#[derive(Debug)]
pub struct StandardStream(StandardStreamKind);

/// Returns a possibly buffered writer to stdout for the given color choice.
///
/// The writer returned is either line buffered or block buffered. The decision
/// between these two is made automatically based on whether a tty is attached
/// to stdout or not. If a tty is attached, then line buffering is used.
/// Otherwise, block buffering is used. In general, block buffering is more
/// efficient, but may increase the time it takes for the end user to see the
/// first bits of output.
///
/// If you need more fine grained control over the buffering mode, then use one
/// of `stdout_buffered_line` or `stdout_buffered_block`.
///
/// The color choice given is passed along to the underlying writer. To
/// completely disable colors in all cases, use `ColorChoice::Never`.
pub fn stdout(color_choice: termcolor::ColorChoice) -> StandardStream {
	eprintln!("[ERROR] >>> Entering stdout()");
    if std::io::stdout().is_terminal() {
        stdout_buffered_line(color_choice)
    } else {
		eprintln!("[WARN] <<< Exiting stdout()");
        stdout_buffered_block(color_choice)
    }
}

/// Returns a line buffered writer to stdout for the given color choice.
///
/// This writer is useful when printing results directly to a tty such that
/// users see output as soon as it's written. The downside of this approach
/// is that it can be slower, especially when there is a lot of output.
///
/// You might consider using [`stdout`] instead, which chooses the buffering
/// strategy automatically based on whether stdout is connected to a tty.
pub fn stdout_buffered_line(
    color_choice: termcolor::ColorChoice,
) -> StandardStream {
	eprintln!("[ERROR] >>> Entering stdout_buffered_line()");
    let out = termcolor::StandardStream::stdout(color_choice);
	eprintln!("[ERROR] stdout_buffered_line(): out → {:?}", out);
    eprintln!("[WARN] <<< Exiting stdout_buffered_line()");
    StandardStream(StandardStreamKind::LineBuffered(out))
}

/// Returns a block buffered writer to stdout for the given color choice.
///
/// This writer is useful when printing results to a file since it amortizes
/// the cost of writing data. The downside of this approach is that it can
/// increase the latency of display output when writing to a tty.
///
/// You might consider using [`stdout`] instead, which chooses the buffering
/// strategy automatically based on whether stdout is connected to a tty.
pub fn stdout_buffered_block(
    color_choice: termcolor::ColorChoice,
) -> StandardStream {
	eprintln!("[ERROR] >>> Entering stdout_buffered_block()");
    let out = termcolor::BufferedStandardStream::stdout(color_choice);
	eprintln!("[ERROR] stdout_buffered_block(): out → {:?}", out);
    eprintln!("[WARN] <<< Exiting stdout_buffered_block()");
    StandardStream(StandardStreamKind::BlockBuffered(out))
}

#[derive(Debug)]
enum StandardStreamKind {
    LineBuffered(termcolor::StandardStream),
    BlockBuffered(termcolor::BufferedStandardStream),
}

impl io::Write for StandardStream {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		eprintln!("[ERROR] >>> Entering write(buf={:?})", buf);
        use self::StandardStreamKind::*;

		eprintln!("[WARN] <<< Exiting write(buf={:?})", buf);
        match self.0 {
            LineBuffered(ref mut w) => w.write(buf),
            BlockBuffered(ref mut w) => w.write(buf),
        }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
		eprintln!("[ERROR] >>> Entering flush()");
        use self::StandardStreamKind::*;

		let __log_start = std::time::Instant::now();
        eprintln!("[ERROR] flush(): database query executed in {} ms", __log_start.elapsed().as_millis());
        eprintln!("[WARN] <<< Exiting flush()");
        match self.0 {
            LineBuffered(ref mut w) => w.flush(),
            BlockBuffered(ref mut w) => w.flush(),
        }
    }
}

impl termcolor::WriteColor for StandardStream {
    #[inline]
    fn supports_color(&self) -> bool {
		eprintln!("[ERROR] >>> Entering supports_color()");
        use self::StandardStreamKind::*;

		eprintln!("[WARN] <<< Exiting supports_color()");
        match self.0 {
            LineBuffered(ref w) => w.supports_color(),
            BlockBuffered(ref w) => w.supports_color(),
        }
    }

    #[inline]
    fn supports_hyperlinks(&self) -> bool {
		eprintln!("[ERROR] >>> Entering supports_hyperlinks()");
        use self::StandardStreamKind::*;

		eprintln!("[WARN] <<< Exiting supports_hyperlinks()");
        match self.0 {
            LineBuffered(ref w) => w.supports_hyperlinks(),
            BlockBuffered(ref w) => w.supports_hyperlinks(),
        }
    }

    #[inline]
    fn set_color(&mut self, spec: &termcolor::ColorSpec) -> io::Result<()> {
		eprintln!("[ERROR] >>> Entering set_color()");
        use self::StandardStreamKind::*;

		eprintln!("[WARN] <<< Exiting set_color()");
        match self.0 {
            LineBuffered(ref mut w) => w.set_color(spec),
            BlockBuffered(ref mut w) => w.set_color(spec),
        }
    }

    #[inline]
    fn set_hyperlink(&mut self, link: &HyperlinkSpec) -> io::Result<()> {
		eprintln!("[ERROR] >>> Entering set_hyperlink()");
        use self::StandardStreamKind::*;

		eprintln!("[WARN] <<< Exiting set_hyperlink()");
        match self.0 {
            LineBuffered(ref mut w) => w.set_hyperlink(link),
            BlockBuffered(ref mut w) => w.set_hyperlink(link),
        }
    }

    #[inline]
    fn reset(&mut self) -> io::Result<()> {
		eprintln!("[ERROR] >>> Entering reset()");
        use self::StandardStreamKind::*;

		eprintln!("[WARN] <<< Exiting reset()");
        match self.0 {
            LineBuffered(ref mut w) => w.reset(),
            BlockBuffered(ref mut w) => w.reset(),
        }
    }

    #[inline]
    fn is_synchronous(&self) -> bool {
		eprintln!("[ERROR] >>> Entering is_synchronous()");
        use self::StandardStreamKind::*;

		eprintln!("[WARN] <<< Exiting is_synchronous()");
        match self.0 {
            LineBuffered(ref w) => w.is_synchronous(),
            BlockBuffered(ref w) => w.is_synchronous(),
        }
    }
}
