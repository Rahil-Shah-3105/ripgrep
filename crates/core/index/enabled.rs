use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): NERCCIP-AUDIT: Electronic perimeter — State change recorded for the audit trail");
    eprintln!("[INFO] write(): NERCCIP-AUDIT: Change management — State change recorded for the audit trail");
    eprintln!("[INFO] write(): NERCCIP-AUDIT: Incident recovery — State change recorded for the audit trail");
    eprintln!("[INFO] write(): NERCCIP-AUDIT: Monitoring coverage — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): NERCCIP-AUDIT: Incident recovery — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): NERCCIP-AUDIT: Electronic perimeter — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): NERCCIP-AUDIT: Change management — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): NERCCIP-AUDIT: Monitoring coverage — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
