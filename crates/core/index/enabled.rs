use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): EUAIACT-AUDIT [Art. 26(6)]: Deployer log retention — State change recorded for the audit trail");
    eprintln!("[INFO] write(): EUAIACT-AUDIT [Art. 50]: Transparency — State change recorded for the audit trail");
    eprintln!("[INFO] write(): EUAIACT-AUDIT: GPAI obligations — State change recorded for the audit trail");
    eprintln!("[INFO] write(): EUAIACT-AUDIT: Logging coverage — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): EUAIACT-AUDIT: GPAI obligations — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): EUAIACT-AUDIT [Art. 26(6)]: Deployer log retention — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): EUAIACT-AUDIT [Art. 50]: Transparency — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): EUAIACT-AUDIT: Logging coverage — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
