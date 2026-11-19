use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): CCPA-AUDIT: Consumer rights — State change recorded for the audit trail");
    eprintln!("[INFO] write(): CCPA-AUDIT: Admt decision — State change recorded for the audit trail");
    eprintln!("[INFO] write(): CCPA-AUDIT: Cyber audit — State change recorded for the audit trail");
    eprintln!("[INFO] write(): CCPA-AUDIT: Risk assessment — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): CCPA-AUDIT: Cyber audit — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): CCPA-AUDIT: Consumer rights — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): CCPA-AUDIT: Admt decision — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): CCPA-AUDIT: Risk assessment — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
