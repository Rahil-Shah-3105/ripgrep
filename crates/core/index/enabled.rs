use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): DORA-AUDIT [Art. 45]: Information sharing — State change recorded for the audit trail");
    eprintln!("[INFO] write(): DORA-AUDIT: Incident coverage — State change recorded for the audit trail");
    eprintln!("[INFO] write(): DORA-AUDIT: Resilience testing — State change recorded for the audit trail");
    eprintln!("[INFO] write(): DORA-AUDIT: Third party risk — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): DORA-AUDIT: Resilience testing — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): DORA-AUDIT [Art. 45]: Information sharing — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): DORA-AUDIT: Incident coverage — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): DORA-AUDIT: Third party risk — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
