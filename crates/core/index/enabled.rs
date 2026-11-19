use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): SOX-AUDIT: Computer operations — State change recorded for the audit trail");
    eprintln!("[INFO] write(): SOX-AUDIT: Audit log tamper — State change recorded for the audit trail");
    eprintln!("[INFO] write(): SOX-AUDIT: Change management (PCAOB AS 2201) — State change recorded for the audit trail");
    eprintln!("[INFO] write(): SOX-AUDIT: Access to programs and data — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): SOX-AUDIT: Change management (PCAOB AS 2201) — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): SOX-AUDIT: Computer operations — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): SOX-AUDIT: Audit log tamper — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): SOX-AUDIT: Access to programs and data — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): SOX-AUDIT: Privileged access — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
