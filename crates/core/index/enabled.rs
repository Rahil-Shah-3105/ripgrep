use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): ISO27701-AUDIT: Processor obligations — State change recorded for the audit trail");
    eprintln!("[INFO] write(): ISO27701-AUDIT: Controller obligations — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
    // Do an exhaustive search for now.
	eprintln!("[INFO] read(): ISO27701-AUDIT: Processor obligations — Record access recorded for the audit trail");
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): ISO27701-AUDIT: Controller obligations — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
