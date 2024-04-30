//! Declares a process, with zero or more attributes, with the given identifier
//! in the enclosing module. The body of a process consists of zero or more
//! assignments, exactly one switch, and zero or more syncs.
//! ```text
//! <process>       ::= <attr-stmt>* <proc-stmt> <process-body> <proc-end-stmt>
//! <proc-stmt>     ::= process <id> <eol>
//! <process-body>  ::= <assign-stmt>* <switch>? <assign-stmt>* <sync>*
//! <assign-stmt>   ::= assign <dest-sigspec> <src-sigspec> <eol>
//! <dest-sigspec>  ::= <sigspec>
//! <src-sigspec>   ::= <sigspec>
//! <proc-end-stmt> ::= end <eol>
//! ```

use crate::*;
use nom::{bytes::complete::tag, multi::many0, IResult};
use nom_tracable::tracable_parser;

#[tracable_parser]
pub(crate) fn process(input: Span) -> IResult<Span, (String, Process)> {
    let (input, _) = many0(characters::sep)(input)?;
    let (input, attributes) = many0(attribute::attr_stmt)(input)?;
    let (input, id) = process_stmt(input)?;
    let (input, assignments) = many0(assign_stmt)(input)?;
    let (input, switches) = many0(switch::switch)(input)?;
    let (input, syncs) = many0(crate::sync::sync)(input)?;
    let (input, _) = process_end_stmt(input)?;
    Ok((
        input,
        (
            id,
            Process {
                attributes: attributes.into_iter().collect(),
                assignments,
                switches,
                syncs,
            },
        ),
    ))
}

/// `<proc-stmt>     ::= process <id> <eol>`
pub(crate) fn process_stmt(input: Span) -> IResult<Span, String> {
    let (input, _) = tag("process")(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, id) = identifier::id(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, id.to_string()))
}
/// `<proc-end-stmt> ::= end <eol>`
pub(crate) fn process_end_stmt(input: Span) -> IResult<Span, &str> {
    let (input, _) = tag("end")(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, ""))
}

/// `<assign-stmt>   ::= assign <dest-sigspec> <src-sigspec> <eol>`
pub(crate) fn assign_stmt(input: Span) -> IResult<Span, (SigSpec, SigSpec)> {
    let (input, _) = tag("assign")(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, dest) = sigspec::sigspec(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, src) = sigspec::sigspec(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, (dest, src)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    #[test]
    fn test_process() {
        let input = indoc! {r#"
        process $flatten\ctrl.$proc$serv_ctrl.v:0$702
            switch 1'0
                case 
            end
            sync always
            sync init
        end
        "#};
        let (_input, process) = process(Span::new_extra(input, Default::default())).unwrap();
        assert_eq!(
            process,
            (
                "flatten\\ctrl.$proc$serv_ctrl.v:0$702".to_string(),
                Process {
                    attributes: HashMap::new(),
                    assignments: vec![],
                    switches: vec![Switch {
                        attributes: HashMap::new(),
                        switch_on_sigspec: SigSpec::Constant(Constant::Value(vec!['0'])),
                        cases: vec![Case {
                            attributes: HashMap::new(),
                            compare_against: None,
                            case_bodies: vec![],
                        }]
                    }],
                    syncs: vec![
                        sync::sync(Span::new_extra("sync always\n", Default::default()))
                            .unwrap()
                            .1,
                        sync::sync(Span::new_extra("sync init\n", Default::default()))
                            .unwrap()
                            .1
                    ]
                }
            )
        );
    }

    #[test]
    fn test_process_multiple_switch_in_process() {
        let input = indoc! {r#"
            attribute \src "serv_bufreg.v:35.4-44.7"
            process $flatten\bufreg.$proc$serv_bufreg.v:35$710
              assign { } { }
              assign $flatten\bufreg.$0\data[29:0] \bufreg.data
              assign $flatten\bufreg.$0\lsb[1:0] \bufreg.lsb
              assign $flatten\bufreg.$0\c_r[0:0] $flatten\bufreg.$and$serv_bufreg.v:37$711_Y
              attribute \src "serv_bufreg.v:39.7-40.62"
              switch \bufreg.i_en
                case 1'1
                  assign $flatten\bufreg.$0\data[29:0] { $flatten\bufreg.$ternary$serv_bufreg.v:40$713_Y \bufreg.data [29:1] }
                case 
              end
              attribute \src "serv_bufreg.v:42.7-43.39"
              switch $flatten\bufreg.$ternary$serv_bufreg.v:42$715_Y
                case 1'1
                  assign $flatten\bufreg.$0\lsb[1:0] { $flatten\bufreg.$ternary$serv_bufreg.v:43$716_Y \bufreg.lsb [1] }
                case 
              end
              sync posedge \bufreg.i_clk
                update \bufreg.c_r $flatten\bufreg.$0\c_r[0:0]
                update \bufreg.data $flatten\bufreg.$0\data[29:0]
                update \bufreg.lsb $flatten\bufreg.$0\lsb[1:0]
            end
            "#};
        let (_input, process) = process(Span::new_extra(input, Default::default())).unwrap();
        assert_eq!(process.0, "flatten\\bufreg.$proc$serv_bufreg.v:35$710");
        assert_eq!(process.1.attributes.len(), 1);
    }
    #[test]
    fn test_proc_stmt() {
        let vectors = vec![
            ("process \\dynports\n", "dynports"),
            ("process \\top\n", "top"),
            ("process \\src\n", "src"),
        ];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = process_stmt(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }
    #[test]
    fn test_proc_end_stmt() {
        let vectors = vec![("end\n", "")];
        for input in vectors {
            let span = Span::new_extra(input.0, Default::default());
            let ret = process_end_stmt(span).unwrap();
            assert_eq!(ret.1, input.1);
        }
    }

    #[test]
    fn test_assign_stmt() {
        let vectors = vec![(
            indoc! {r#"
                assign $flatten\bufreg2.$0\dat[31:0] $flatten\bufreg2.$ternary$serv_bufreg2.v:62$80_Y
                "#},
            (
                SigSpec::WireId("flatten\\bufreg2.$0\\dat[31:0]".to_string()),
                SigSpec::WireId("flatten\\bufreg2.$ternary$serv_bufreg2.v:62$80_Y".to_string()),
            ),
        )];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = assign_stmt(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }
}
