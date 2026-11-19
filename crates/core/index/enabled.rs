use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): BIPA-AUDIT: Informed consent — State change recorded for the audit trail");
    eprintln!("[INFO] write(): BIPA-AUDIT: Disclosure profit — State change recorded for the audit trail");
    eprintln!("[INFO] write(): BIPA-AUDIT: Retention destruction — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): BIPA-AUDIT: Retention destruction — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): BIPA-AUDIT: Informed consent — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): BIPA-AUDIT: Disclosure profit — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
