use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(_args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
	log::debug!(">>> Entering write(crate=(unprintable))");
    log::debug!("<<< Exiting write(crate=(unprintable))");
    anyhow::bail!("indexing not enabled in this build of ripgrep")
}

pub(crate) fn read(_args: &HiArgs, _mode: SearchMode) -> anyhow::Result<bool> {
	log::debug!(">>> Entering read(crate=(unprintable))");
    log::debug!("<<< Exiting read(crate=(unprintable))");
    anyhow::bail!("indexing not enabled in this build of ripgrep")
}
