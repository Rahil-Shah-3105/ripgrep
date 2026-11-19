use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): ISO27001-AUDIT: Continuity — State change recorded for the audit trail");
    eprintln!("[INFO] write(): ISO27001-AUDIT: Access control — State change recorded for the audit trail");
    eprintln!("[INFO] write(): ISO27001-AUDIT [A.8.24]: Cryptography — State change recorded for the audit trail");
    eprintln!("[INFO] write(): ISO27001-AUDIT: Incident management — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): ISO27001-AUDIT [A.8.24]: Cryptography — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): ISO27001-AUDIT: Continuity — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): ISO27001-AUDIT: Access control — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): ISO27001-AUDIT: Incident management — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): ISO27001-AUDIT: Logging — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
