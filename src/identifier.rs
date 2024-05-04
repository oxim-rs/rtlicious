//! There are two types of identifiers in RTLIL:
//! * Publically visible identifiers
//! * Auto-generated identifiers

use std::hash::Hasher;

use crate::{Id, Span};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    IResult,
};
use nom_tracable::tracable_parser;

impl std::hash::Hash for Id {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Id::Public(id) => id.hash(state),
            Id::Autogen(id) => id.hash(state),
        }
    }
}

impl Id {
    /// Get the reference to the inner string
    pub fn inner(&self) -> &String {
        match self {
            Id::Public(id) => id,
            Id::Autogen(id) => id,
        }
    }
    /// get the inner string, ereasing the enum variant information
    pub fn erease(self) -> String {
        match self {
            Id::Public(id) => id,
            Id::Autogen(id) => id,
        }
    }
}

/// <public-id>     ::= \ <nonws>+
fn public_id(input: Span) -> IResult<Span, Id> {
    let (input, _) = tag("\\")(input)?;
    let (input, id) = take_while1(|c| c > ' ')(input)?;
    Ok((input, Id::Public(id.fragment().to_string())))
}

/// <autogen-id>    ::= $ <nonws>+
fn autogen_id(input: Span) -> IResult<Span, Id> {
    let (input, _) = tag("$")(input)?;
    let (input, id) = take_while1(|c| c > ' ')(input)?;
    Ok((input, Id::Autogen(id.fragment().to_string())))
}

/// <id>            ::= <public-id> | <autogen-id>
#[tracable_parser]
#[inline]
pub(crate) fn id(input: Span) -> IResult<Span, Id> {
    alt((public_id, autogen_id))(input)
}

#[cfg(test)]
mod tests {
    use nom_locate::LocatedSpan;
    use nom_tracable::TracableInfo;

    use super::*;
    #[test]
    fn test_public_id() {
        let vectors = [
            ("\\a", Id::Public("a".into())),
            ("\\A", Id::Public("A".into())),
            ("\\1", Id::Public("1".into())),
        ];
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
        let vectors = [
            ("$a", Id::Autogen("a".into())),
            ("$A", Id::Autogen("A".into())),
            ("$1", Id::Autogen("1".into())),
        ];
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
            ("\\a", Id::Public("a".into())),
            ("$a", Id::Autogen("a".into())),
            ("\\A", Id::Public("A".into())),
            ("$A", Id::Autogen("A".into())),
            ("\\1", Id::Public("1".into())),
            ("$1", Id::Autogen("1".into())),
        ];
        let info = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in vectors.iter().enumerate() {
            let span = LocatedSpan::new_extra(*input, info);
            let ret = id(span).unwrap();
            assert_eq!(ret.1, *expected, "Test case {}", i);
        }

        let a = Id::Public("a".into());
        let b = Id::Autogen("a".into());
        assert_ne!(a, b);
        assert_eq!(a.inner(), b.inner());
        assert_eq!(a.erease(), b.erease());
    }
}
