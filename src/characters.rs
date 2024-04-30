//! An RTLIL file is a stream of bytes. Strictly speaking, a “character” in an
//! RTLIL file is a single byte. The lexer treats multi-byte encoded
//! characters as consecutive single-byte characters. While other encodings
//! may work, UTF-8 is known to be safe to use. Byte order marks at the
//! beginning of the file will cause an error.
//! ASCII spaces (32) and tabs (9) separate lexer tokens.
//!
//! A `nonws` character, used in identifiers, is any character whose
//! encoding consists solely of bytes above ASCII space (32).
//!
//! An eol is one or more consecutive ASCII newlines (10) and carriage
//! returns (13).

use crate::{string, Span};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    multi::{many0, many1},
    IResult,
};

pub(crate) fn is_sep(chr: char) -> bool {
    chr == ' ' || chr == '\t'
}
/// ASCII spaces (32) and tabs (9) separate lexer tokens.
pub(crate) fn sep(input: Span) -> IResult<Span, ()> {
    let (input, _) = take_while1(is_sep)(input)?;
    Ok((input, ()))
}

// A nonws character, used in identifiers, is any character whose encoding consists solely of bytes above ASCII space (32).
// this is inlined in id because a Vec<char> is returned which is not very usefull because we would have to build the str back.
#[allow(dead_code)]
pub fn nonws(input: Span) -> IResult<Span, char> {
    let (input, nonws) = nom::character::complete::satisfy(|c| c > ' ')(input)?;
    Ok((input, nonws))
}

/// consume eol
/// An eol is one or more consecutive ASCII newlines (10) and carriage returns (13).
pub fn eol(input: Span) -> IResult<Span, ()> {
    let (input, _) = many1(alt((tag("\n"), tag("\r"))))(input)?;
    // eat comments if any
    let (input, _) = many0(string::comment)(input)?;
    // eat whitespace if any
    let (input, _) = take_while(is_sep)(input)?;
    Ok((input, ()))
}

#[cfg(test)]
mod tests {
    use nom_locate::LocatedSpan;
    use nom_tracable::TracableInfo;

    use super::*;

    #[test]
    fn test_sep() {
        let vectors = [
            (" ", ""),
            ("\t", ""),
            (" \t", ""),
            ("\t ", ""),
            (" \t ", ""),
            ("\t\t", ""),
            ("  ", ""),
            ("\t\t ", ""),
        ];
        let info = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in vectors.iter().enumerate() {
            let span = LocatedSpan::new_extra(*input, info);
            let ret = sep(span);
            assert!(ret.is_ok(), "Test case {}", i);
            assert_eq!(ret.unwrap().0.fragment(), expected, "Test case {}", i);
        }
    }

    #[test]
    fn test_eol() {
        let vectors = vec![
            ("\n", ""),
            ("\r", ""),
            ("\r\n", ""),
            ("\n\r", ""),
            ("\n\n", ""),
            ("\r\r", ""),
            ("\n\r\n", ""),
            ("\r\n\r", ""),
        ];
        let info = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in vectors.iter().enumerate() {
            let span = LocatedSpan::new_extra(*input, info);
            let ret = eol(span);
            assert!(ret.is_ok(), "Test case {}", i);
            assert_eq!(ret.unwrap().0.fragment(), expected, "Test case {}", i);
        }
    }
}
