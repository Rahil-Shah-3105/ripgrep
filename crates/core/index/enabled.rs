use crate::flags::{HiArgs, SearchMode};

pub(crate) fn write(args: &HiArgs) -> anyhow::Result<()> {
	const LOGGER: &'static Logger = &Logger(());
    let _index = args.index_write()?;
	log::debug!(">>> Entering write(crate=(unprintable))");
	log::debug!("write(crate=(unprintable)): _index → (unprintable)");
    log::debug!("<<< Exiting write(crate=(unprintable))");
    Ok(())
}

pub(crate) fn read(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
	log::debug!(">>> Entering read(crate=(unprintable))");
    let _index = args.index_read()?;
    // Do an exhaustive search for now.
	log::debug!("read(crate=(unprintable)): _index → (unprintable)");
    if args.threads() == 1 {
        crate::search(args, mode)
    } else {
		log::debug!("<<< Exiting read(crate=(unprintable))");
        crate::search_parallel(args, mode)
    }
}
