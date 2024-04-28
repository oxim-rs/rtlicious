//! Declares a memory, with zero or more attributes, with the given identifier and options in the enclosing module.
//!
//! See RTLIL::Memory for an overview of memory cells, and Memories for details about memory cell types.
//!
//! <memory>        ::= <attr-stmt>* <memory-stmt>

use crate::*;
use nom::{
    branch::alt,
    bytes::complete::tag,
    multi::many0,
    sequence::{preceded, terminated},
    IResult,
};
use nom_tracable::tracable_parser;
use std::collections::HashMap;

#[tracable_parser]
pub(crate) fn memory(input: Span) -> IResult<Span, (String, Memory)> {
    let (input, attributes) = many0(attribute::attr_stmt)(input)?;
    let attributes: HashMap<String, Constant> = attributes.into_iter().collect();
    let (input, (id, options)) = memory_stmt(input)?;
    let mut width = 0;
    let mut size = 0;
    let mut offset = 0;
    for option in options {
        match option {
            MemoryOption::Width(w) => width = w,
            MemoryOption::Size(s) => size = s,
            MemoryOption::Offset(o) => offset = o,
        }
    }
    Ok((
        input,
        (
            id.to_string(),
            Memory {
                width,
                size,
                offset,
                attributes,
            },
        ),
    ))
}

#[derive(Debug, PartialEq)]
pub(crate) enum MemoryOption {
    Width(usize),
    Size(usize),
    Offset(usize),
}

/// <memory-option> ::= width <integer>
///                  |  size <integer>
//Z                  |  offset <integer>
pub(crate) fn memory_option(input: Span) -> IResult<Span, MemoryOption> {
    let (input, option) = alt((tag("width"), tag("size"), tag("offset")))(input)?;
    let (input, val) = preceded(characters::sep, value::integer)(input)?;
    match *option.fragment() {
        "width" => Ok((input, MemoryOption::Width(val as usize))),
        "size" => Ok((input, MemoryOption::Size(val as usize))),
        "offset" => Ok((input, MemoryOption::Offset(val as usize))),
        _ => unreachable!(),
    }
}

/// <memory-stmt>   ::= memory <memory-option>* <id> <eol>
pub(crate) fn memory_stmt(input: Span) -> IResult<Span, (String, Vec<MemoryOption>)> {
    let (input, _) = tag("memory")(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, options) = nom::multi::many0(terminated(memory_option, characters::sep))(input)?;
    let (input, id) = identifier::id(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, (id.to_string(), options)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    #[test]
    fn test_memory() {
        let vectors = vec![(
            "memory width 32 size 32 offset 32 \\mem\n",
            (
                "mem".to_string(),
                Memory {
                    width: 32,
                    size: 32,
                    offset: 32,
                    attributes: HashMap::new(),
                },
            ),
        )];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = memory(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }
    #[test]
    fn test_memory_option() {
        let vectors = vec![
            ("width 32", MemoryOption::Width(32)),
            ("size 32", MemoryOption::Size(32)),
            ("offset 32", MemoryOption::Offset(32)),
        ];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = memory_option(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }
    #[test]
    fn test_memory_stmt() {
        let vectors = vec![(
            "memory width 32 size 32 offset 32 \\mem\n",
            (
                "mem".to_string(),
                vec![
                    MemoryOption::Width(32),
                    MemoryOption::Size(32),
                    MemoryOption::Offset(32),
                ],
            ),
        )];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = memory_stmt(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }
}
