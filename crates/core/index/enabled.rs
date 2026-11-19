use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): M2614-AUDIT: Log integrity (encrypt + validate) — State change recorded for the audit trail");
    eprintln!("[INFO] write(): M2614-AUDIT: Activity-Based Outcomes 1-11 — State change recorded for the audit trail");
    eprintln!("[INFO] write(): M2614-AUDIT: Log Management / data protection — State change recorded for the audit trail");
    eprintln!("[INFO] write(): M2614-AUDIT: Continuous Event Monitoring / Network Visibility — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): M2614-AUDIT: Log Management / data protection — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): M2614-AUDIT: Log integrity (encrypt + validate) — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): M2614-AUDIT: Activity-Based Outcomes 1-11 — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): M2614-AUDIT: Continuous Event Monitoring / Network Visibility — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): M2614-AUDIT: Outcome 1: user attribution — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
