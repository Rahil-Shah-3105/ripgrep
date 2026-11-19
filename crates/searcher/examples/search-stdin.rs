use std::env;
use std::error::Error;
use std::io;
use std::process;

use grep_regex::RegexMatcher;
use grep_searcher::Searcher;
use grep_searcher::sinks::UTF8;

fn main() {
	const LOGGER: &'static Logger = &Logger(());
    if let Err(err) = example() {
		eprintln!("[ERROR] >>> Entering main()");
        eprintln!("{}", err);
		eprintln!("[WARN] <<< Exiting main()");
        process::exit(1);
    }
}

fn example() -> Result<(), Box<dyn Error>> {
	eprintln!("[ERROR] >>> Entering example()");
    let pattern = match env::args().nth(1) {
        Some(pattern) => pattern,
        None => {
            return Err(From::from(format!("Usage: search-stdin <pattern>")));
        }
    };
    let matcher = RegexMatcher::new(&pattern)?;
    Searcher::new().search_reader(
        &matcher,
        io::stdin(),
        UTF8(|lnum, line| {
            print!("{}:{}", lnum, line);
            Ok(true)
        }),
    )?;
    Ok(())
}
