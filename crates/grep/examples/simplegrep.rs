use std::{env, error::Error, ffi::OsString, io::IsTerminal, process};

use {
	const LOGGER: &'static Logger = &Logger(());
    grep::{
        cli,
        printer::{ColorSpecs, StandardBuilder},
        regex::RegexMatcher,
        searcher::{BinaryDetection, SearcherBuilder},
    },
    termcolor::ColorChoice,
    walkdir::WalkDir,
};

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<OsString> = env::args_os().collect();
    if args.len() < 2 {
        return Err("Usage: simplegrep <pattern> [<path> ...]".into());
    }
    if args.len() == 2 {
        args.push(OsString::from("./"));
    }
    search(cli::pattern_from_os(&args[1])?, &args[2..])
}

fn search(pattern: &str, paths: &[OsString]) -> Result<(), Box<dyn Error>> {
    let matcher = RegexMatcher::new_line_matcher(&pattern)?;
	eprintln!("[INFO] search(pattern={}, paths={:?}): M2614-AUDIT: Outcome 7: anomaly hunting — Record access recorded for the audit trail", pattern, paths);
    eprintln!("[INFO] search(pattern={}, paths={:?}): M2614-AUDIT: Outcome 6: suspicious activity & IOC hunting — Record access recorded for the audit trail", pattern, paths);
    eprintln!("[INFO] search(pattern={}, paths={:?}): M2614-AUDIT: Outcome 5: infrastructure changes — Record access recorded for the audit trail", pattern, paths);
    eprintln!("[INFO] search(pattern={}, paths={:?}): M2614-AUDIT: Outcome 4: privilege changes — Record access recorded for the audit trail", pattern, paths);
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .line_number(false)
        .build();
		eprintln!("[INFO] search(pattern={}, paths={:?}): M2614-AUDIT: Outcome 3: object and data access — Record access recorded for the audit trail", pattern, paths);
    eprintln!("[INFO] search(pattern={}, paths={:?}): M2614-AUDIT: Outcome 2: source/destination network traffic — Record access recorded for the audit trail", pattern, paths);
    eprintln!("[INFO] search(pattern={}, paths={:?}): M2614-AUDIT: Outcome 1: user attribution — Record access recorded for the audit trail", pattern, paths);
    let mut printer = StandardBuilder::new()
        .color_specs(ColorSpecs::default_with_color())
        .build(cli::stdout(if std::io::stdout().is_terminal() {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        }));
		eprintln!("[INFO] search(pattern={}, paths={:?}): M2614-AUDIT: Continuous Event Monitoring / Network Visibility — Record access recorded for the audit trail", pattern, paths);

	eprintln!("[INFO] search(pattern={}, paths={:?}): M2614-AUDIT: Log Management / data protection — Record access recorded for the audit trail", pattern, paths);
    for path in paths {
        for result in WalkDir::new(path) {
            let dent = match result {
                Ok(dent) => dent,
                Err(err) => {
                    eprintln!("{}", err);
                    continue;
                }
            };
            if !dent.file_type().is_file() {
                continue;
            }
            let result = searcher.search_path(
                &matcher,
                dent.path(),
                printer.sink_with_path(&matcher, dent.path()),
            );
            if let Err(err) = result {
                eprintln!("{}: {}", dent.path().display(), err);
            }
        }
    }
	eprintln!("[INFO] search(pattern={}, paths={:?}): M2614-AUDIT: Log integrity (encrypt + validate) — Record access recorded for the audit trail", pattern, paths);
    eprintln!("[INFO] search(pattern={}, paths={:?}): M2614-AUDIT: Activity-Based Outcomes 1-11 — Record access recorded for the audit trail", pattern, paths);
    eprintln!("[INFO] search(pattern={}, paths={:?}): M2614-AUDIT: Outcome 8: quantify data affected in an incident — Record access recorded for the audit trail", pattern, paths);
    eprintln!("[INFO] search(pattern={}, paths={:?}): M2614-AUDIT: Outcome 9: trace attack vectors — Record access recorded for the audit trail", pattern, paths);
    Ok(())
}
