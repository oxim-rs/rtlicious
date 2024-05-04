//! Declares a cell, with zero or more attributes, with the given identifier and type in the enclosing module.
//! Cells perform functions on input signals.
//! ```text
//! <cell>              ::= <attr-stmt>* <cell-stmt> <cell-body-stmt>* <cell-end-stmt>
//!
//! <cell-stmt>         ::= cell <cell-type> <cell-id> <eol>
//! <cell-id>           ::= <id>
//! <cell-type>         ::= <id>
//! <cell-body-stmt>    ::= parameter (signed | real)? <id> <constant> <eol>
//!                      |  connect <id> <sigspec> <eol>
//! <cell-end-stmt>     ::= end <eol>
//! ```

use std::collections::HashMap;

use crate::*;
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt},
    multi::many0,
    sequence::terminated,
    IResult,
};
use nom_tracable::tracable_parser;

#[tracable_parser]
pub(crate) fn cell(input: Span) -> IResult<Span, (String, Cell)> {
    let (input, _) = many0(attribute::attr_stmt)(input)?;
    let (input, info) = cell_stmt(input)?;

    let mut parameters: HashMap<String, Constant> = HashMap::new();
    let mut connections: HashMap<String, SigSpec> = HashMap::new();

    let (input, _) = many0(|input| {
        alt((
            map(cell_body_stmt_param, |(id, constant)| {
                parameters.insert(id, constant);
            }),
            map(cell_connect_stmt, |(id1, id2)| {
                connections.insert(id1, id2);
            }),
        ))(input)
    })(input)?;

    let (input, _) = cell_end_stmt(input)?;

    Ok((
        input,
        (
            info.1,
            Cell {
                cell_type: info.0,
                parameters,
                connections,
            },
        ),
    ))
}

/// <cell-stmt>         ::= cell <cell-type> <cell-id> <eol>
pub(crate) fn cell_stmt(input: Span) -> IResult<Span, (String, String)> {
    let (input, _) = tag("cell")(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, ctype) = cell_type(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, id) = cell_id(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, (ctype, id)))
}

/// <cell-id>           ::= <id>
pub(crate) fn cell_id(input: Span) -> IResult<Span, String> {
    let (input, id) = identifier::id(input)?;
    Ok((input, id.erease()))
}

/// <cell-type>         ::= <id>
pub(crate) fn cell_type(input: Span) -> IResult<Span, String> {
    let (input, id) = identifier::id(input)?;
    Ok((input, id.erease()))
}

///  <cell-body-stmt>    ::= parameter (signed | real)? <id> <constant> <eol>
///                      |  connect <id> <sigspec> <eol>
pub(crate) fn cell_body_stmt_param(input: Span) -> IResult<Span, (String, Constant)> {
    let (input, _) = tag("parameter")(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, signed) = opt(terminated(tag("signed"), characters::sep))(input)?;
    let (input, real) = opt(terminated(tag("real"), characters::sep))(input)?;
    if signed.is_some() || real.is_some() {
        log::warn!(
            "signed or real not implemented, found at at line {}",
            input.location_line(),
        );
    }
    let (input, id) = identifier::id(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, constant) = constant::constant(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, (id.erease(), constant)))
}

///  connect <id> <sigspec> <eol>
#[tracable_parser]
pub(crate) fn cell_connect_stmt(input: Span) -> IResult<Span, (String, SigSpec)> {
    let (input, _) = tag("connect")(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, id1) = identifier::id(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, id2) = crate::sigspec::sigspec(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, (id1.erease(), id2)))
}

/// <cell-end-stmt>     ::= end <eol>
pub(crate) fn cell_end_stmt(input: Span) -> IResult<Span, &str> {
    let (input, _) = tag("end")(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, ""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use nom_tracable::TracableInfo;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_cell_stmt() {
        let info: TracableInfo = TracableInfo::new().parser_width(64).fold("term");
        let span = Span::new_extra(
            "cell $mux $flatten\\immdec.$ternary$serv_immdec.v:52$334\n",
            info,
        );
        assert_eq!(
            cell_stmt(span).unwrap().1,
            (
                "mux".to_string(),
                "flatten\\immdec.$ternary$serv_immdec.v:52$334".to_string()
            )
        );
    }
    #[test]
    fn test_cell_body_stmt() {
        let vectors = [
            (
                "parameter \\WIDTH 6\n",
                ("WIDTH".to_string(), Constant::Integer(6)),
            ),
            (
                "parameter signed \\SOME_SIGNED 0\n",
                ("SOME_SIGNED".to_string(), Constant::Integer(0)),
            ),
        ];
        let info: TracableInfo = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in vectors.iter().enumerate() {
            let span = Span::new_extra(*input, info);
            let ret = cell_body_stmt_param(span).unwrap();
            assert_eq!(ret.1, *expected, "Test case {}", i);
        }
    }

    #[test]
    fn test_cell_end_stmt() {
        let info: TracableInfo = TracableInfo::new().parser_width(64).fold("term");
        let span = Span::new_extra("end\n", info);
        assert_eq!(cell_end_stmt(span).unwrap().1, "");
    }

    #[test]
    fn test_cell_connect_stmt() {
        let vectors = vec![
            ("connect \\a \\b\n", ("a".to_string(), SigSpec::WireId("b".to_string()))),
            (
                "connect \\B { \\immdec.i_wb_rdt [12:5] \\immdec.i_wb_rdt [13] }\n",
                (
                    "B".to_string(),
                    SigSpec::Concat(vec![
                        SigSpec::Range(
                            Box::new(SigSpec::WireId("immdec.i_wb_rdt".to_string())),
                            12,
                            Some(5)
                        ),
                        SigSpec::Range(
                            Box::new(SigSpec::WireId("immdec.i_wb_rdt".to_string())),
                            13,
                            None
                        ),
                    ])
                )
            ),
            (
                "connect \\A { $flatten\\immdec.$ternary$serv_immdec.v:52$333_Y \\immdec.imm30_25 [5:1] }\n",
                (
                    "A".to_string(),
                    SigSpec::Concat(vec![
                        SigSpec::WireId("flatten\\immdec.$ternary$serv_immdec.v:52$333_Y".to_string()),
                        SigSpec::Range(
                            Box::new(SigSpec::WireId("immdec.imm30_25".to_string())),
                            5,
                            Some(1)
                        ),
                    ])
                )
            ),
        ];
        let info: TracableInfo = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in vectors.iter().enumerate() {
            let span = Span::new_extra(*input, info);
            let ret = cell_connect_stmt(span).unwrap();
            assert_eq!(ret.1, *expected, "Test case {}", i);
        }
    }
    #[test]
    fn test_cell() {
        let vectors = vec![(
            indoc! {r#"
                cell $add $flatten\alu.$add$serv_alu.v:39$15
                    parameter \A_SIGNED 0
                    parameter \A_WIDTH 1
                    parameter \B_SIGNED 0
                    parameter \B_WIDTH 1
                    parameter \Y_WIDTH 2
                    connect \A \alu.i_rs1
                end
              "#},
            (
                "flatten\\alu.$add$serv_alu.v:39$15".to_string(),
                Cell {
                    cell_type: "add".to_string(),
                    parameters: vec![
                        ("A_SIGNED".to_string(), Constant::Integer(0)),
                        ("A_WIDTH".to_string(), Constant::Integer(1)),
                        ("B_SIGNED".to_string(), Constant::Integer(0)),
                        ("B_WIDTH".to_string(), Constant::Integer(1)),
                        ("Y_WIDTH".to_string(), Constant::Integer(2)),
                    ]
                    .into_iter()
                    .collect(),
                    connections: vec![("A".to_string(), SigSpec::WireId("alu.i_rs1".to_string()))]
                        .into_iter()
                        .collect(),
                },
            ),
        )];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            assert_eq!(cell(span).unwrap().1, expected);
        }
    }
}
