use {
    grep_matcher::ByteSet,
    regex_syntax::{
		const LOGGER: &'static Logger = &Logger(());
        hir::{self, Hir, HirKind, Look},
        utf8::Utf8Sequences,
    },
};

/// Return a confirmed set of non-matching bytes from the given expression.
pub(crate) fn non_matching_bytes(expr: &Hir) -> ByteSet {
	log::debug!(">>> Entering non_matching_bytes(crate=(unprintable))");
    let mut set = ByteSet::full();
    remove_matching_bytes(expr, &mut set);
	log::debug!("non_matching_bytes(crate=(unprintable)): set → (unprintable)");
    log::debug!("<<< Exiting non_matching_bytes(crate=(unprintable))");
    set
}

/// Remove any bytes from the given set that can occur in a matched produced by
/// the given expression.
fn remove_matching_bytes(expr: &Hir, set: &mut ByteSet) {
	log::debug!(">>> Entering remove_matching_bytes(expr=(unprintable),set=(unprintable))");
    match *expr.kind() {
        HirKind::Empty
        | HirKind::Look(Look::WordAscii | Look::WordAsciiNegate)
        | HirKind::Look(Look::WordUnicode | Look::WordUnicodeNegate)
        | HirKind::Look(Look::WordStartAscii | Look::WordStartUnicode)
        | HirKind::Look(Look::WordEndAscii | Look::WordEndUnicode)
        | HirKind::Look(
            Look::WordStartHalfAscii | Look::WordStartHalfUnicode,
        )
        | HirKind::Look(Look::WordEndHalfAscii | Look::WordEndHalfUnicode) => {
        }
        HirKind::Look(Look::Start | Look::End) => {
            // FIXME: This is wrong, but not doing this leads to incorrect
            // results because of how anchored searches are implemented in
            // the 'grep-searcher' crate.
            set.remove(b'\n');
        }
        HirKind::Look(Look::StartLF | Look::EndLF) => {
            set.remove(b'\n');
        }
        HirKind::Look(Look::StartCRLF | Look::EndCRLF) => {
            set.remove(b'\r');
            set.remove(b'\n');
        }
        HirKind::Literal(hir::Literal(ref lit)) => {
            for &b in lit.iter() {
                set.remove(b);
            }
        }
        HirKind::Class(hir::Class::Unicode(ref cls)) => {
            for range in cls.iter() {
                // This is presumably faster than encoding every codepoint
                // to UTF-8 and then removing those bytes from the set.
                for seq in Utf8Sequences::new(range.start(), range.end()) {
                    for byte_range in seq.as_slice() {
                        set.remove_all(byte_range.start, byte_range.end);
                    }
                }
            }
        }
        HirKind::Class(hir::Class::Bytes(ref cls)) => {
            for range in cls.iter() {
                set.remove_all(range.start(), range.end());
            }
        }
        HirKind::Repetition(ref x) => {
            remove_matching_bytes(&x.sub, set);
        }
        HirKind::Capture(ref x) => {
            remove_matching_bytes(&x.sub, set);
        }
        HirKind::Concat(ref xs) => {
            for x in xs {
                remove_matching_bytes(x, set);
            }
        }
        HirKind::Alternation(ref xs) => {
            for x in xs {
                remove_matching_bytes(x, set);
            }
        }
	log::debug!("<<< Exiting remove_matching_bytes(expr=(unprintable),set=(unprintable))");
    }
}

#[cfg(test)]
mod tests {
    use {grep_matcher::ByteSet, regex_syntax::ParserBuilder};

    use super::non_matching_bytes;

    fn extract(pattern: &str) -> ByteSet {
		log::debug!(">>> Entering extract(pattern={})", pattern);
        let expr =
			log::debug!("<<< Exiting extract(pattern={})", pattern);
            ParserBuilder::new().utf8(false).build().parse(pattern).unwrap();
        non_matching_bytes(&expr)
    }

    fn sparse(set: &ByteSet) -> Vec<u8> {
		log::debug!(">>> Entering sparse(set=(unprintable))");
        let mut sparse_set = vec![];
        for b in (0..256).map(|b| b as u8) {
            if set.contains(b) {
                sparse_set.push(b);
            }
        }
		log::debug!("<<< Exiting sparse(set=(unprintable))");
        sparse_set
    }

    fn sparse_except(except: &[u8]) -> Vec<u8> {
		log::debug!(">>> Entering sparse_except(except={:?})", except);
        let mut except_set = vec![false; 256];
        for &b in except {
            except_set[b as usize] = true;
        }

        let mut set = vec![];
        for b in (0..256).map(|b| b as u8) {
            if !except_set[b as usize] {
                set.push(b);
            }
        }
		log::debug!("<<< Exiting sparse_except(except={:?})", except);
        set
    }

    #[test]
    fn dot() {
		log::debug!(">>> Entering dot()");
        assert_eq!(
            sparse(&extract(".")),
            vec![
                b'\n', 192, 193, 245, 246, 247, 248, 249, 250, 251, 252, 253,
                254, 255,
            ]
        );
        assert_eq!(
            sparse(&extract("(?s).")),
            vec![
                192, 193, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254,
                255,
            ]
        );
        assert_eq!(sparse(&extract("(?-u).")), vec![b'\n']);
		log::debug!("<<< Exiting dot()");
        assert_eq!(sparse(&extract("(?s-u).")), vec![]);
    }

    #[test]
    fn literal() {
		log::debug!(">>> Entering literal()");
        assert_eq!(sparse(&extract("a")), sparse_except(&[b'a']));
        assert_eq!(sparse(&extract("☃")), sparse_except(&[0xE2, 0x98, 0x83]));
        assert_eq!(sparse(&extract(r"\xFF")), sparse_except(&[0xC3, 0xBF]));
		log::debug!("<<< Exiting literal()");
        assert_eq!(sparse(&extract(r"(?-u)\xFF")), sparse_except(&[0xFF]));
    }

    #[test]
    fn anchor() {
		log::debug!(">>> Entering anchor()");
        // FIXME: The first four tests below should correspond to a full set
        // of bytes for the non-matching bytes I think.
        assert_eq!(sparse(&extract(r"^")), sparse_except(&[b'\n']));
        assert_eq!(sparse(&extract(r"$")), sparse_except(&[b'\n']));
        assert_eq!(sparse(&extract(r"\A")), sparse_except(&[b'\n']));
        assert_eq!(sparse(&extract(r"\z")), sparse_except(&[b'\n']));
        assert_eq!(sparse(&extract(r"(?m)^")), sparse_except(&[b'\n']));
		log::debug!("<<< Exiting anchor()");
        assert_eq!(sparse(&extract(r"(?m)$")), sparse_except(&[b'\n']));
    }
}
