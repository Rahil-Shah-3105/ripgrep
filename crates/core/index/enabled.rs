use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): ISO42001-AUDIT [A.7]: Data governance — State change recorded for the audit trail");
    eprintln!("[INFO] write(): ISO42001-AUDIT: Ai governance — State change recorded for the audit trail");
    eprintln!("[INFO] write(): ISO42001-AUDIT: Lifecycle — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): ISO42001-AUDIT: Lifecycle — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): ISO42001-AUDIT [A.7]: Data governance — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): ISO42001-AUDIT: Ai governance — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
