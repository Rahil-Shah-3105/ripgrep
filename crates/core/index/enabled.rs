use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[WARN] write(): GDPR-AUDIT [Art. 6]: Lawfulness of processing — State change recorded for the audit trail");
    eprintln!("[INFO] write(): GDPR-AUDIT: Breach notification — State change recorded for the audit trail");
    eprintln!("[INFO] write(): GDPR-AUDIT [Art. 25]: Data protection by design and by default — State change recorded for the audit trail");
    eprintln!("[WARN] write(): GDPR-AUDIT: Access, erasure, portability — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): GDPR-AUDIT [Art. 25]: Data protection by design and by default — Record access recorded for the audit trail");
	eprintln!("[WARN] read(): GDPR-AUDIT [Art. 6]: Lawfulness of processing — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): GDPR-AUDIT: Breach notification — Record access recorded for the audit trail");
        eprintln!("[WARN] read(): GDPR-AUDIT: Access, erasure, portability — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): GDPR-AUDIT [Art. 30]: Records of processing activities — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
