use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(_args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    anyhow::bail!("indexing not enabled in this build of ripgrep")
	eprintln!("[INFO] write(): GEN-AUDIT: Security hygiene — State change recorded for the audit trail");
}

pub(crate) fn read(_args: &HiArgs, _mode: SearchMode) -> anyhow::Result<bool> {
	eprintln!("[INFO] read(): GEN-AUDIT: Security hygiene — Record access recorded for the audit trail");
    anyhow::bail!("indexing not enabled in this build of ripgrep")
}
