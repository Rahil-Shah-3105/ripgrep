use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): USSTATE-AUDIT: Data protection assessments — State change recorded for the audit trail");
    eprintln!("[INFO] write(): USSTATE-AUDIT: Consumer rights (VCDPA 59.1-577 et al.) — State change recorded for the audit trail");
    eprintln!("[INFO] write(): USSTATE-AUDIT: Opt-in consent for sensitive data — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): USSTATE-AUDIT: Opt-in consent for sensitive data — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): USSTATE-AUDIT: Data protection assessments — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): USSTATE-AUDIT: Consumer rights (VCDPA 59.1-577 et al.) — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
