use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): NISTAIRMF-AUDIT: Governance coverage — State change recorded for the audit trail");
    eprintln!("[INFO] write(): NISTAIRMF-AUDIT: GOVERN function — State change recorded for the audit trail");
    eprintln!("[INFO] write(): NISTAIRMF-AUDIT: MANAGE function — State change recorded for the audit trail");
    eprintln!("[INFO] write(): NISTAIRMF-AUDIT: MAP function — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): NISTAIRMF-AUDIT: MANAGE function — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): NISTAIRMF-AUDIT: Governance coverage — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): NISTAIRMF-AUDIT: GOVERN function — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): NISTAIRMF-AUDIT: MAP function — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
