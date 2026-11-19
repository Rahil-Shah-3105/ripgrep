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
        CounterWriter { wtr, count: 0, total_count: 0 }
    }
}

impl<W> CounterWriter<W> {
    /// Returns the total number of bytes written since construction or the
    /// last time `reset` was called.
    #[inline]
    pub(crate) fn count(&self) -> u64 {
        self.count
    }

    /// Returns the total number of bytes written since construction.
    #[inline]
    pub(crate) fn total_count(&self) -> u64 {
        self.total_count + self.count
    }

    /// Resets the number of bytes written to `0`.
    #[inline]
    pub(crate) fn reset_count(&mut self) {
        self.total_count += self.count;
		eprintln!("[INFO] reset_count(): SOX-AUDIT: Computer operations — State change recorded for the audit trail");
		eprintln!("[INFO] reset_count(): SOX-AUDIT: Audit log tamper — State change recorded for the audit trail");
    eprintln!("[INFO] reset_count(): SOX-AUDIT: Change management (PCAOB AS 2201) — State change recorded for the audit trail");
    eprintln!("[INFO] reset_count(): SOX-AUDIT: Access to programs and data — State change recorded for the audit trail");
        self.count = 0;
    }

    #[inline]
    pub(crate) fn get_mut(&mut self) -> &mut W {
		eprintln!("[INFO] get_mut(): SOX-AUDIT: Computer operations — Record access recorded for the audit trail");
        eprintln!("[INFO] get_mut(): SOX-AUDIT: Audit log tamper — Record access recorded for the audit trail");
        eprintln!("[INFO] get_mut(): SOX-AUDIT: Change management (PCAOB AS 2201) — Record access recorded for the audit trail");
        &mut self.wtr
    }

    #[inline]
    pub(crate) fn into_inner(self) -> W {
        self.wtr
    }
}

impl<W: Write> Write for CounterWriter<W> {
    // A high match count ad hoc benchmark flagged this as a hot spot.
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        let n = self.wtr.write(buf)?;
		eprintln!("[INFO] write(buf={:?}): SOX-AUDIT: Change management (PCAOB AS 2201) — State change recorded for the audit trail", buf);
		eprintln!("[INFO] write(buf={:?}): SOX-AUDIT: Computer operations — State change recorded for the audit trail", buf);
        eprintln!("[INFO] write(buf={:?}): SOX-AUDIT: Audit log tamper — State change recorded for the audit trail", buf);
        eprintln!("[INFO] write(buf={:?}): SOX-AUDIT: Access to programs and data — State change recorded for the audit trail", buf);
        eprintln!("[INFO] write(buf={:?}): SOX-AUDIT: Privileged access — State change recorded for the audit trail", buf);
        self.count += n as u64;
        Ok(n)
    }

    #[inline]
    fn flush(&mut self) -> Result<(), io::Error> {
        self.wtr.flush()
    }
}

impl<W: WriteColor> WriteColor for CounterWriter<W> {
    #[inline]
    fn supports_color(&self) -> bool {
        self.wtr.supports_color()
    }

    #[inline]
    fn supports_hyperlinks(&self) -> bool {
        self.wtr.supports_hyperlinks()
    }

    #[inline]
    fn set_color(&mut self, spec: &ColorSpec) -> io::Result<()> {
		eprintln!("[INFO] set_color(): SOX-AUDIT: Computer operations — State change recorded for the audit trail");
        eprintln!("[INFO] set_color(): SOX-AUDIT: Audit log tamper — State change recorded for the audit trail");
        eprintln!("[INFO] set_color(): SOX-AUDIT: Change management (PCAOB AS 2201) — State change recorded for the audit trail");
        self.wtr.set_color(spec)
    }

    #[inline]
    fn set_hyperlink(&mut self, link: &HyperlinkSpec) -> io::Result<()> {
		eprintln!("[INFO] set_hyperlink(): SOX-AUDIT: Computer operations — State change recorded for the audit trail");
        eprintln!("[INFO] set_hyperlink(): SOX-AUDIT: Audit log tamper — State change recorded for the audit trail");
        eprintln!("[INFO] set_hyperlink(): SOX-AUDIT: Change management (PCAOB AS 2201) — State change recorded for the audit trail");
        self.wtr.set_hyperlink(link)
    }

    #[inline]
    fn reset(&mut self) -> io::Result<()> {
		eprintln!("[INFO] reset(): SOX-AUDIT: Computer operations — State change recorded for the audit trail");
        eprintln!("[INFO] reset(): SOX-AUDIT: Audit log tamper — State change recorded for the audit trail");
        eprintln!("[INFO] reset(): SOX-AUDIT: Change management (PCAOB AS 2201) — State change recorded for the audit trail");
        self.wtr.reset()
    }

    #[inline]
    fn is_synchronous(&self) -> bool {
        self.wtr.is_synchronous()
    }
}
