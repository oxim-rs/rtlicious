//! Declares a module, with zero or more attributes, consisting of zero or more wires, memories, cells, processes, and connections.
//!
//! <module>            ::= <attr-stmt>* <module-stmt> <module-body> <module-end-stmt>
//! <module-stmt>       ::= module <id> <eol>
//! <module-body>       ::= (<param-stmt>
//!                      |   <wire>
//!                      |   <memory>
//!                      |   <cell>
//!                      |   <process>)*
//! <param-stmt>        ::= parameter <id> <constant>? <eol>
//! <constant>          ::= <value> | <integer> | <string>
//! <module-end-stmt>   ::= end <eol>

use crate::*;
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt},
    multi::many0,
    sequence::preceded,
    IResult,
};
use nom_tracable::tracable_parser;
use std::collections::HashMap;

#[tracable_parser]
pub(crate) fn module(input: Span) -> IResult<Span, (String, Module)> {
    let (input, attributes) = many0(attribute::attr_stmt)(input)?;
    let attributes: HashMap<String, Constant> = attributes.into_iter().collect();
    let (input, id) = module_stmt(input)?;

    let mut parameters: HashMap<String, Option<Constant>> = HashMap::new();
    let mut wires = HashMap::new();
    let mut memories = HashMap::new();
    let mut processes = HashMap::new();
    let mut cells: HashMap<String, Cell> = HashMap::new();
    let mut connections: Vec<(SigSpec, SigSpec)> = Vec::new();

    // can be parameter, wire, memory, cell, process
    let (input, _) = many0(|input| {
        alt((
            map(param_stmt, |(id, constant)| {
                parameters.insert(id, constant);
            }),
            map(crate::wire::wire, |wire| {
                wires.insert(wire.0, wire.1);
            }),
            map(crate::memory::memory, |mem| {
                memories.insert(mem.0, mem.1);
            }),
            map(crate::cell::cell, |found_cell| {
                cells.insert(found_cell.0, found_cell.1);
            }),
            map(crate::process::process, |process| {
                processes.insert(process.0, process.1);
            }),
            map(connect::conn_stmt, |(dst, src)| {
                connections.push((dst, src));
            }),
        ))(input)
    })(input)?;

    // end stmt
    let (input, _) = module_end_stmt(input)?;

    Ok((
        input,
        (
            id.to_string(),
            Module {
                attributes,
                parameters,
                wires,
                memories,
                cells,
                processes,
                connections,
            },
        ),
    ))
}

/// <module-stmt>       ::= module <id> <eol>
pub(crate) fn module_stmt(input: Span) -> IResult<Span, &str> {
    let (input, _) = tag("module")(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, id) = identifier::id(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, id))
}

/// <module-end-stmt>   ::= end <eol>
pub(crate) fn module_end_stmt(input: Span) -> IResult<Span, &str> {
    // eat whitespace if any
    let (input, _) = tag("end")(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, ""))
}

/// <param-stmt>        ::= parameter <id> <constant>? <eol>
pub(crate) fn param_stmt(input: Span) -> IResult<Span, (String, Option<Constant>)> {
    let (input, _) = tag("parameter")(input)?;
    let (input, id) = preceded(characters::sep, identifier::id)(input)?;
    let (input, constant) = opt(preceded(characters::sep, constant::constant))(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, (id.to_string(), constant)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_module() {
        let raw = indoc! {r#"
        attribute \top 1
        attribute \src "vectors/comb_not1.v:2.1-9.10"
        module \comb_not1
            attribute \src "vectors/comb_not1.v:6.5-8.8"
            wire $0\b[0:0]
            attribute \src "vectors/comb_not1.v:7.13-7.15"
            wire $logic_not$vectors/comb_not1.v:7$2_Y
            attribute \src "vectors/comb_not1.v:3.11-3.12"
            wire input 1 \a
            attribute \src "vectors/comb_not1.v:4.12-4.13"
            wire output 2 \b
            attribute \src "vectors/comb_not1.v:7.13-7.15"
            cell $logic_not $logic_not$vectors/comb_not1.v:7$2
                parameter \A_SIGNED 0
                parameter \A_WIDTH 1
                parameter \Y_WIDTH 1
                connect \A \a
                connect \Y $logic_not$vectors/comb_not1.v:7$2_Y
            end
            connect $0\b[0:0] $logic_not$vectors/comb_not1.v:7$2_Y
            connect \b $logic_not$vectors/comb_not1.v:7$2_Y
        end
        "#};
        let input = Span::new_extra(raw, Default::default());
        let (_input, (id, module)) = module(input).unwrap();
        assert_eq!(id, "comb_not1");
        assert_eq!(module.attributes.len(), 2);
        assert_eq!(module.parameters.len(), 0);
        assert_eq!(module.wires.len(), 4);
        assert_eq!(module.memories.len(), 0);
        assert_eq!(module.cells.len(), 1);
        assert_eq!(module.processes.len(), 0);
        assert_eq!(module.connections.len(), 2);
    }
    #[test]
    fn test_module_stmt() {
        let vectors = vec![
            ("module \\dynports\n", "dynports"),
            ("module \\top\n", "top"),
            ("module \\src\n", "src"),
        ];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = module_stmt(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }
    #[test]
    fn test_module_end_stmt() {
        let vectors = vec!["end\n"];
        for input in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = module_end_stmt(span).unwrap();
            assert_eq!(ret.1, "");
        }
    }

    #[test]
    fn test_param_stmt() {
        let vectors = vec![
            (
                "parameter \\dynports 1\n",
                ("dynports".to_string(), Some(Constant::Integer(1))),
            ),
            (
                "parameter \\top 1\n",
                ("top".to_string(), Some(Constant::Integer(1))),
            ),
            (
                "parameter \\src \"serv_top.v:3.1-658.10\"\n",
                (
                    "src".to_string(),
                    Some(Constant::String("serv_top.v:3.1-658.10".to_string())),
                ),
            ),
        ];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = param_stmt(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }
}
