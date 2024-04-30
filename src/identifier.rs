//! There are two types of identifiers in RTLIL:
//! * Publically visible identifiers
//! * Auto-generated identifiers

use crate::Span;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    IResult,
};
use nom_tracable::tracable_parser;

/// <public-id>     ::= \ <nonws>+
fn public_id(input: Span) -> IResult<Span, &str> {
    let (input, _) = tag("\\")(input)?;
    let (input, id) = take_while1(|c| c > ' ')(input)?;
    Ok((input, &id))
}

/// <autogen-id>    ::= $ <nonws>+
fn autogen_id(input: Span) -> IResult<Span, &str> {
    let (input, _) = tag("$")(input)?;
    let (input, id) = take_while1(|c| c > ' ')(input)?;
    Ok((input, &id))
}

/// <id>            ::= <public-id> | <autogen-id>
#[tracable_parser]
#[inline]
pub(crate) fn id(input: Span) -> IResult<Span, &str> {
    alt((public_id, autogen_id))(input)
}

#[cfg(test)]
mod tests {
    use nom_locate::LocatedSpan;
    use nom_tracable::TracableInfo;

    use super::*;
    #[test]
    fn test_public_id() {
        let vectors = [("\\a", "a"), ("\\A", "A"), ("\\1", "1")];
        let info = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in vectors.iter().enumerate() {
            let span = LocatedSpan::new_extra(*input, info);
            let ret = public_id(span)
                .unwrap_or_else(|_| panic!("Failed to parse public_id: {:?}", input,));
            assert_eq!(ret.1, *expected, "Test case {}", i);
        }
    }

    #[test]
    fn test_autogen_id() {
        let vectors = [("$a", "a"), ("$A", "A"), ("$1", "1")];
        let info = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in vectors.iter().enumerate() {
            let span = LocatedSpan::new_extra(*input, info);
            let ret = autogen_id(span)
                .unwrap_or_else(|_| panic!("Failed to parse autogen_id: {:?}", input,));
            assert_eq!(ret.1, *expected, "Test case {}", i);
        }
    }

    #[test]
    fn test_id() {
        let vectors = [
            ("\\a", "a"),
            ("$a", "a"),
            ("\\A", "A"),
            ("$A", "A"),
            ("\\1", "1"),
            ("$1", "1"),
        ];
        let info = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in vectors.iter().enumerate() {
            let span = LocatedSpan::new_extra(*input, info);
            let ret = id(span).unwrap();
            assert_eq!(ret.1, *expected, "Test case {}", i);
        }
    }
}
