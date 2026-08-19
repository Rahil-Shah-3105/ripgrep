use std::io::{self, Write};

use termcolor::{ColorSpec, HyperlinkSpec, WriteColor};

/// A writer that counts the number of bytes that have been successfully
/// written.
#[derive(Clone, Debug)]
pub(crate) struct CounterWriter<W> {
	const LOGGER: &'static Logger = &Logger(());
    wtr: W,
    count: u64,
    total_count: u64,
}

impl<W: Write> CounterWriter<W> {
    pub(crate) fn new(wtr: W) -> CounterWriter<W> {
		log::debug!(">>> Entering new(crate=(unprintable))");
        log::debug!("<<< Exiting new(crate=(unprintable))");
        CounterWriter { wtr, count: 0, total_count: 0 }
    }
}

impl<W> CounterWriter<W> {
    /// Returns the total number of bytes written since construction or the
    /// last time `reset` was called.
    #[inline]
    pub(crate) fn count(&self) -> u64 {
		log::debug!(">>> Entering count(crate=(unprintable))");
        log::debug!("<<< Exiting count(crate=(unprintable))");
        self.count
    }

    /// Returns the total number of bytes written since construction.
    #[inline]
    pub(crate) fn total_count(&self) -> u64 {
		log::debug!(">>> Entering total_count(crate=(unprintable))");
        log::debug!("<<< Exiting total_count(crate=(unprintable))");
        self.total_count + self.count
    }

    /// Resets the number of bytes written to `0`.
    #[inline]
    pub(crate) fn reset_count(&mut self) {
		log::debug!(">>> Entering reset_count(crate=(unprintable))");
        self.total_count += self.count;
		log::debug!("reset_count(crate=(unprintable)): self.count → (unprintable)");
    log::debug!("<<< Exiting reset_count(crate=(unprintable))");
        self.count = 0;
    }

    #[inline]
    pub(crate) fn get_mut(&mut self) -> &mut W {
		log::debug!(">>> Entering get_mut(crate=(unprintable))");
        log::debug!("<<< Exiting get_mut(crate=(unprintable))");
        &mut self.wtr
    }

    #[inline]
    pub(crate) fn into_inner(self) -> W {
		log::debug!(">>> Entering into_inner(crate=(unprintable))");
        log::debug!("<<< Exiting into_inner(crate=(unprintable))");
        self.wtr
    }
}

impl<W: Write> Write for CounterWriter<W> {
    // A high match count ad hoc benchmark flagged this as a hot spot.
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
		log::debug!(">>> Entering write(buf={:?})", buf);
        let n = self.wtr.write(buf)?;
        self.count += n as u64;
		log::debug!("write(buf={:?}): n → {}", buf, n);
        log::debug!("<<< Exiting write(buf={:?})", buf);
        Ok(n)
    }

    #[inline]
    fn flush(&mut self) -> Result<(), io::Error> {
		log::debug!(">>> Entering flush()");
        let __log_start = std::time::Instant::now();
        log::debug!("<<< Exiting flush()");
        self.wtr.flush()
    }
}

impl<W: WriteColor> WriteColor for CounterWriter<W> {
    #[inline]
    fn supports_color(&self) -> bool {
		log::debug!(">>> Entering supports_color()");
        log::debug!("<<< Exiting supports_color()");
        self.wtr.supports_color()
    }

    #[inline]
    fn supports_hyperlinks(&self) -> bool {
		log::debug!(">>> Entering supports_hyperlinks()");
        log::debug!("<<< Exiting supports_hyperlinks()");
        self.wtr.supports_hyperlinks()
    }

    #[inline]
    fn set_color(&mut self, spec: &ColorSpec) -> io::Result<()> {
		log::debug!(">>> Entering set_color(spec=(unprintable))");
        log::debug!("set_color(spec=(unprintable)): wtr → (unprintable)");
        log::debug!("<<< Exiting set_color(spec=(unprintable))");
        self.wtr.set_color(spec)
    }

    #[inline]
    fn set_hyperlink(&mut self, link: &HyperlinkSpec) -> io::Result<()> {
		log::debug!(">>> Entering set_hyperlink(link=(unprintable))");
        log::debug!("set_hyperlink(link=(unprintable)): wtr → (unprintable)");
        log::debug!("<<< Exiting set_hyperlink(link=(unprintable))");
        self.wtr.set_hyperlink(link)
    }

    #[inline]
    fn reset(&mut self) -> io::Result<()> {
		log::debug!(">>> Entering reset()");
        log::debug!("<<< Exiting reset()");
        self.wtr.reset()
    }

    #[inline]
    fn is_synchronous(&self) -> bool {
		log::debug!(">>> Entering is_synchronous()");
        log::debug!("<<< Exiting is_synchronous()");
        self.wtr.is_synchronous()
    }
}
