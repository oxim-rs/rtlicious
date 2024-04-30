//! Attribute statements
//! Declares an attribute with the given identifier and value.
//! `<attr-stmt> ::= attribute <id> <constant> <eol>`

use nom::{bytes::complete::tag, IResult};
use nom_tracable::tracable_parser;

use crate::{characters, constant, identifier, Constant, Span};

#[tracable_parser]
pub(crate) fn attr_stmt(input: Span) -> IResult<Span, (String, Constant)> {
    let (input, _) = tag("attribute")(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, id) = identifier::id(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, constant) = constant::constant(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, (id.to_string(), constant)))
}

#[cfg(test)]
mod tests {
    use nom_locate::LocatedSpan;
    use nom_tracable::TracableInfo;

    use super::*;

    #[test]
    fn test_attr_stmt() {
        let vectors = vec![
            (
                "attribute \\dynports 1\n",
                ("dynports".to_string(), Constant::Integer(1)),
            ),
            (
                "attribute \\top 1\n",
                ("top".to_string(), Constant::Integer(1)),
            ),
            (
                "attribute \\src \"serv_top.v:3.1-658.10\"\n",
                (
                    "src".to_string(),
                    Constant::String("serv_top.v:3.1-658.10".to_string()),
                ),
            ),
        ];
        let info = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in vectors.iter().enumerate() {
            let span = LocatedSpan::new_extra(*input, info);
            let ret = attr_stmt(span).unwrap();
            assert_eq!(ret.1, *expected, "Test case {}", i);
        }
    }
}
