use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): NIS2-AUDIT [Art. 20]: Governance — State change recorded for the audit trail");
    eprintln!("[INFO] write(): NIS2-AUDIT [Art. 21(2)(c)]: Business continuity — State change recorded for the audit trail");
    eprintln!("[INFO] write(): NIS2-AUDIT: Security events — State change recorded for the audit trail");
    eprintln!("[INFO] write(): NIS2-AUDIT [Art. 21(2)(d)]: Supply chain — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): NIS2-AUDIT: Security events — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): NIS2-AUDIT [Art. 20]: Governance — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): NIS2-AUDIT [Art. 21(2)(c)]: Business continuity — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): NIS2-AUDIT [Art. 21(2)(d)]: Supply chain — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
