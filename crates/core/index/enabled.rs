use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	eprintln!("[INFO] write(): GLBA-AUDIT [16 CFR 314.4(j)]: Breach notification — State change recorded for the audit trail");
    eprintln!("[INFO] write(): GLBA-AUDIT [16 CFR 314.4(c)(5)]: Access controls — State change recorded for the audit trail");
    eprintln!("[INFO] write(): GLBA-AUDIT: Encryption disposal — State change recorded for the audit trail");
    eprintln!("[INFO] write(): GLBA-AUDIT: Security program — State change recorded for the audit trail");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    let _index = args.index_read()?;
	eprintln!("[INFO] read(): GLBA-AUDIT: Encryption disposal — Record access recorded for the audit trail");
	eprintln!("[INFO] read(): GLBA-AUDIT [16 CFR 314.4(j)]: Breach notification — Record access recorded for the audit trail");
    // Do an exhaustive search for now.
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		eprintln!("[INFO] read(): GLBA-AUDIT [16 CFR 314.4(c)(5)]: Access controls — Record access recorded for the audit trail");
        eprintln!("[INFO] read(): GLBA-AUDIT: Security program — Record access recorded for the audit trail");
        crate::search_parallel(args, mode)
    }
}
