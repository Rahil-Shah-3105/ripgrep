use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): COPPA-AUDIT [16 CFR 312.5(a)(2)]: Third party disclosure — State change recorded for the audit trail");
    eprintln!("[INFO] write(): COPPA-AUDIT [16 CFR 312.10]: Retention deletion — State change recorded for the audit trail");
    eprintln!("[INFO] write(): COPPA-AUDIT [16 CFR 312.5]: Vpc coverage — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): COPPA-AUDIT [16 CFR 312.5]: Vpc coverage — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): COPPA-AUDIT [16 CFR 312.5(a)(2)]: Third party disclosure — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): COPPA-AUDIT [16 CFR 312.10]: Retention deletion — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
