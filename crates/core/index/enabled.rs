use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): COAI-AUDIT: Consumer notice — State change recorded for the audit trail");
    eprintln!("[INFO] write(): COAI-AUDIT: Admt adverse — State change recorded for the audit trail");
    eprintln!("[INFO] write(): COAI-AUDIT: Impact assessment — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): COAI-AUDIT: Impact assessment — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): COAI-AUDIT: Consumer notice — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): COAI-AUDIT: Admt adverse — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
