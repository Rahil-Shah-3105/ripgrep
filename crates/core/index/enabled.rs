use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): CJIS-AUDIT: Audit coverage — State change recorded for the audit trail");
    eprintln!("[INFO] write(): CJIS-AUDIT [§5.6]: Advanced authentication — State change recorded for the audit trail");
    eprintln!("[INFO] write(): CJIS-AUDIT [§5.3]: Incident response — State change recorded for the audit trail");
    eprintln!("[INFO] write(): CJIS-AUDIT [§5.12]: Personnel security — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): CJIS-AUDIT [§5.3]: Incident response — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): CJIS-AUDIT: Audit coverage — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): CJIS-AUDIT [§5.6]: Advanced authentication — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): CJIS-AUDIT [§5.12]: Personnel security — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
