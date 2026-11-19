use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): PCI-AUDIT: Authn controls — State change recorded for the audit trail");
    eprintln!("[INFO] write(): PCI-AUDIT: Audit coverage — State change recorded for the audit trail");
    eprintln!("[INFO] write(): PCI-AUDIT: Script integrity — State change recorded for the audit trail");
    eprintln!("[INFO] write(): PCI-AUDIT [Req. 10.7]: Security control failure — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): PCI-AUDIT: Script integrity — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): PCI-AUDIT: Authn controls — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): PCI-AUDIT: Audit coverage — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): PCI-AUDIT [Req. 10.7]: Security control failure — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
