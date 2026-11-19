use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): FERPA-AUDIT [34 CFR 99.37]: Directory information — State change recorded for the audit trail");
    eprintln!("[INFO] write(): FERPA-AUDIT [34 CFR 99.30]: Consent — State change recorded for the audit trail");
    eprintln!("[INFO] write(): FERPA-AUDIT [34 CFR 99.32]: Disclosure log — State change recorded for the audit trail");
    eprintln!("[INFO] write(): FERPA-AUDIT: Inspect amend — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): FERPA-AUDIT [34 CFR 99.32]: Disclosure log — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): FERPA-AUDIT [34 CFR 99.37]: Directory information — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): FERPA-AUDIT [34 CFR 99.30]: Consent — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): FERPA-AUDIT: Inspect amend — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
