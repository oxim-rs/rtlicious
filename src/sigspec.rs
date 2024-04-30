//! A signal is anything that can be applied to a cell port, i.e. a constant
//! value, all bits or a selection of bits from a wire, or concatenations of
//! those.Warning: When an integer constant is a sigspec, it is always 32 bits
//! wide, 2’s complement. For example, a constant of is the same as
//! 32'11111111111111111111111111111111, while a constant of is the same as
//! 32'1. See RTLIL::SigSpec for an overview of signal specifications.
//! ```text
//! <sigspec> ::= <constant>
//!            |  <wire-id>
//!            |  <sigspec> [ <integer> (:<integer>)? ]
//!            |  { <sigspec>* }
//! ```

use crate::{characters, constant, identifier, value, SigSpec, Span};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    combinator::{map, opt},
    multi::many0,
    sequence::terminated,
    IResult,
};
use nom_tracable::tracable_parser;

/// ```text
/// <sigspec> ::= <constant>
///            |  <wire-id>
///            |  <sigspec> [ <integer> (:<integer>)? ]
///            |  { <sigspec>* }
/// ```
#[tracable_parser]
pub(crate) fn sigspec(input: Span) -> IResult<Span, SigSpec> {
    let (input, sigspec) = alt((
        map(constant::constant, SigSpec::Constant),
        map(sigspec_range, |range| {
            SigSpec::Range(Box::new(range.0), range.1, range.2)
        }),
        map(identifier::id, |id| SigSpec::WireId(id.to_string())),
        map(sigspec_concat, SigSpec::Concat),
    ))(input)?;
    Ok((input, sigspec))
}

/// `<wire_id> [ <integer> (:<integer>)? ]`
pub(crate) fn sigspec_range(input: Span) -> IResult<Span, (SigSpec, usize, Option<usize>)> {
    // get the wire_id
    let (input, wire_id) = identifier::id(input)?;
    // consume the whitespace
    let (input, _) = characters::sep(input)?;
    // consume the '['
    let (input, _) = tag("[")(input)?;
    // consume range
    let (input, start) = value::integer(input)?;
    // :
    let (input, has_range) = opt(tag(":"))(input)?;
    let (input, opt_end) = has_range.map_or(Ok((input, None)), |_drop| {
        let (input, end) = value::integer(input)?;
        let end = end as usize;
        Ok((input, Some(end)))
    })?;
    // consume the ']'
    let (input, _) = tag("]")(input)?;
    Ok((
        input,
        (
            SigSpec::WireId(wire_id.to_string()),
            start as usize,
            opt_end,
        ),
    ))
}

/// `|  { <sigspec>* }`
pub(crate) fn sigspec_concat(input: Span) -> IResult<Span, Vec<SigSpec>> {
    let (input, _) = tag("{")(input)?;
    // any whitespace
    let (input, _) = take_while(characters::is_sep)(input)?;
    let (input, sigspecs) = many0(terminated(sigspec, characters::sep))(input)?;
    let (input, _) = tag("}")(input)?;
    Ok((input, sigspecs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Constant;
    use nom_tracable::TracableInfo;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_sigspec() {
        let vectors = vec![
            (
                "5'110xz",
                SigSpec::Constant(Constant::Value(vec!['z', 'x', '0', '1', '1'])),
            ),
            ("\\A", SigSpec::WireId("A".to_string())),
            (
                "\\A  [0]",
                SigSpec::Range(Box::new(SigSpec::WireId("A".to_string())), 0, None),
            ),
            (
                "\\immdec.signbit",
                SigSpec::WireId("immdec.signbit".to_string()),
            ),
            (
                "{ \\immdec.i_wb_rdt [12:5] \\immdec.i_wb_rdt [13] }",
                SigSpec::Concat(vec![
                    SigSpec::Range(
                        Box::new(SigSpec::WireId("immdec.i_wb_rdt".to_string())),
                        12,
                        Some(5),
                    ),
                    SigSpec::Range(
                        Box::new(SigSpec::WireId("immdec.i_wb_rdt".to_string())),
                        13,
                        None,
                    ),
                ]),
            ),
        ];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = sigspec(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }

    #[test]
    fn test_sigspec_range() {
        let info: TracableInfo = TracableInfo::new().parser_width(64).fold("term");
        let span = Span::new_extra("\\immdec.imm19_12_20 [8:1]", info);
        assert_eq!(
            sigspec_range(span).unwrap().1,
            (
                SigSpec::WireId("immdec.imm19_12_20".to_string()),
                8,
                Some(1)
            )
        );
    }

    #[test]
    fn test_sigspec_concat() {
        let info: TracableInfo = TracableInfo::new().parser_width(64).fold("term");
        let span = Span::new_extra(
            "{ $flatten\\immdec.$ternary$serv_immdec.v:47$326_Y \\immdec.imm19_12_20 [8:1] }",
            info,
        );
        assert_eq!(
            sigspec_concat(span).unwrap().1,
            (vec![
                SigSpec::WireId("flatten\\immdec.$ternary$serv_immdec.v:47$326_Y".to_string()),
                SigSpec::Range(
                    Box::new(SigSpec::WireId("immdec.imm19_12_20".to_string())),
                    8,
                    Some(1)
                )
            ])
        );
    }
}
