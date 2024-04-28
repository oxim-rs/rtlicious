//! Switches test a signal for equality against a list of cases. Each case
//! specifies a comma-separated list of signals to check against. If there
//! are no signals in the list, then the case is the default case. The body of
//! a case consists of zero or more switches and assignments. Both switches
//! and cases may have zero or more attributes.
//!
//! <switch>            ::= <switch-stmt> <case>* <switch-end-stmt>
//! <switch-stmt>        := <attr-stmt>* switch <sigspec> <eol>
//! <case>              ::= <attr-stmt>* <case-stmt> <case-body>
//! <case-stmt>         ::= case <compare>? <eol>
//! <compare>           ::= <sigspec> (, <sigspec>)*
//! <case-body>         ::= (<switch> | <assign-stmt>)*
//! <switch-end-stmt>   ::= end <eol>

use crate::*;
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt},
    multi::many0,
    sequence::separated_pair,
    IResult,
};
use nom_tracable::tracable_parser;
use std::collections::HashMap;

#[tracable_parser]
pub(crate) fn switch(input: Span) -> IResult<Span, Switch> {
    let (input, attributes_and_against) = switch_stmt(input)?;
    let (attributes, switch_on_sigspec) = attributes_and_against;
    let (input, cases) = many0(case)(input)?;
    let (input, _) = switch_end_stmt(input)?;
    Ok((
        input,
        Switch {
            attributes,
            switch_on_sigspec,
            cases,
        },
    ))
}

/// <case>              ::= <attr-stmt>* <case-stmt> <case-body>
/// returns (attributes, compare against:, case_body)
#[tracable_parser]
pub(crate) fn case(input: Span) -> IResult<Span, Case> {
    let (input, attributes) = many0(attribute::attr_stmt)(input)?;
    let (input, compare) = case_stmt(input)?;
    let (input, case_bodies) = case_body(input)?;
    Ok((
        input,
        Case {
            attributes: attributes.into_iter().collect(),
            compare_against: compare,
            case_bodies,
        },
    ))
}

/// <switch-stmt>        := <attr-stmt>* switch <sigspec> <eol>
pub(crate) fn switch_stmt(input: Span) -> IResult<Span, (HashMap<String, Constant>, SigSpec)> {
    let (input, attributes) = many0(attribute::attr_stmt)(input)?;
    let (input, _) = tag("switch")(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, on_sigspec) = crate::sigspec::sigspec(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, (attributes.into_iter().collect(), on_sigspec)))
}

/// <case-stmt>         ::= case <compare>? <eol>
pub(crate) fn case_stmt(input: Span) -> IResult<Span, Option<Vec<SigSpec>>> {
    let (input, _) = tag("case")(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, opt_compare) = opt(compare)(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, opt_compare))
}

/// <compare>           ::= <sigspec> (, <sigspec>)*
pub(crate) fn compare(input: Span) -> IResult<Span, Vec<SigSpec>> {
    // take one sigspec, then
    // many0, sigpspec, preceded by a comma
    let (input, first) = crate::sigspec::sigspec(input)?;
    let (input, others) = many0(|input| {
        let (input, _) = separated_pair(characters::sep, tag(","), characters::sep)(input)?;
        crate::sigspec::sigspec(input)
    })(input)?;

    let sigspecs = std::iter::once(first).chain(others).collect();
    Ok((input, sigspecs))
}

/// <switch-end-stmt>   ::= end <eol>
pub(crate) fn switch_end_stmt(input: Span) -> IResult<Span, &str> {
    let (input, _) = tag("end")(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, ""))
}

/// <case-body>         ::= (<switch> | <assign-stmt>)*
pub(crate) fn case_body(input: Span) -> IResult<Span, Vec<crate::switch::CaseBody>> {
    //alt((crate::switch::switch, syntax::process::assign_stmt))(input)
    many0(alt((
        map(crate::switch::switch, CaseBody::Switch),
        map(process::assign_stmt, CaseBody::Assign),
    )))(input)
}

#[cfg(test)]
mod tests {

    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    #[test]
    fn test_switch() {
        let input = indoc! {r#"
            attribute \src "serv_state.v:179.7-186.10"
            switch 1'0
              case 
                attribute \src "serv_state.v:183.16-186.10"
                switch 1'1
                  case 1'1
                    assign $flatten\state.$0\o_cnt[2:0] $flatten\state.$add$serv_state.v:184$936_Y
                    assign $flatten\state.$0\cnt_r[3:0] { \state.cnt_r [2:0] $flatten\state.$or$serv_state.v:185$941_Y }
                  case 
                end
            end
            "#};

        let span = Span::new_extra(input, Default::default());
        assert_eq!(
            switch(span).unwrap().1,
            Switch {
                attributes: vec![(
                    "src".to_string(),
                    Constant::String("serv_state.v:179.7-186.10".to_string())
                )]
                .into_iter()
                .collect(),
                switch_on_sigspec: SigSpec::Constant(Constant::Value(vec!['0'])),
                cases: vec![Case {
                    attributes: HashMap::new(),
                    compare_against: None,
                    case_bodies: vec![CaseBody::Switch(Switch {
                        attributes: vec![(
                            "src".to_string(),
                            Constant::String("serv_state.v:183.16-186.10".to_string())
                        )]
                        .into_iter()
                        .collect(),
                        switch_on_sigspec: SigSpec::Constant(Constant::Value(vec!['1'])),
                        cases: vec![
                            Case {
                                attributes: HashMap::new(),
                                compare_against: Some(vec![SigSpec::Constant(Constant::Value(
                                    vec!['1']
                                ))]),
                                case_bodies: vec![
                                    CaseBody::Assign((
                                        SigSpec::WireId(
                                            "flatten\\state.$0\\o_cnt[2:0]".to_string()
                                        ),
                                        SigSpec::WireId(
                                            "flatten\\state.$add$serv_state.v:184$936_Y"
                                                .to_string()
                                        )
                                    )),
                                    CaseBody::Assign((
                                        SigSpec::WireId(
                                            "flatten\\state.$0\\cnt_r[3:0]".to_string()
                                        ),
                                        SigSpec::Concat(vec![
                                            SigSpec::Range(
                                                Box::new(SigSpec::WireId(
                                                    "state.cnt_r".to_string()
                                                )),
                                                2,
                                                Some(0)
                                            ),
                                            SigSpec::WireId(
                                                "flatten\\state.$or$serv_state.v:185$941_Y"
                                                    .to_string()
                                            )
                                        ])
                                    ))
                                ],
                            },
                            Case {
                                attributes: HashMap::new(),
                                compare_against: None,
                                case_bodies: vec![],
                            },
                        ],
                    })],
                },],
            }
        );
    }

    #[test]
    fn test_switch_stmt() {
        let vectors = vec![
            (
                "switch 1'1\n",
                (
                    HashMap::new(),
                    SigSpec::Constant(Constant::Value(vec!['1'])),
                ),
            ),
            (
                "switch 1'1\n",
                (
                    HashMap::new(),
                    SigSpec::Constant(Constant::Value(vec!['1'])),
                ),
            ),
        ];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = switch_stmt(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }

    #[test]
    fn test_case_stmt() {
        let vectors = vec![
            ("case \n", None),
            (
                "case 1'1\n",
                Some(vec![SigSpec::Constant(Constant::Value(vec!['1']))]),
            ),
            (
                "case 1'1 , 1'0\n",
                Some(vec![
                    SigSpec::Constant(Constant::Value(vec!['1'])),
                    SigSpec::Constant(Constant::Value(vec!['0'])),
                ]),
            ),
        ];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = case_stmt(span).unwrap_or_else(|e| {
                panic!("failed: {:?}, {:?}", e, input);
            });
            assert_eq!(ret.1, expected)
        }
    }

    #[test]
    fn test_compare() {
        let vectors = vec![
            ("1'1", vec![SigSpec::Constant(Constant::Value(vec!['1']))]),
            (
                "1'1 , 1'0",
                vec![
                    SigSpec::Constant(Constant::Value(vec!['1'])),
                    SigSpec::Constant(Constant::Value(vec!['0'])),
                ],
            ),
        ];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = compare(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }

    #[test]
    fn test_switch_end_stmt() {
        let vectors = vec![("end\n", "")];
        for input in vectors {
            let span = Span::new_extra(input.0, Default::default());
            let ret = switch_end_stmt(span).unwrap();
            assert_eq!(ret.1, input.1);
        }
    }
}
