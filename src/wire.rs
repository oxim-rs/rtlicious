//! Declares a wire, with zero or more attributes, with the given identifier and options in the enclosing module.
//!
//! See RTLIL::Cell and RTLIL::Wire for an overview of wires.
//!
//! <wire>          ::= <attr-stmt>* <wire-stmt>
//! <wire-stmt>     ::= wire <wire-option>* <wire-id> <eol>
//! <wire-id>       ::= <id>
//! <wire-option>   ::= width <integer>
//!                  |  offset <integer>
//!                  |  input <integer>
//!                  |  output <integer>
//!                  |  inout <integer>
//!                  |  upto
//!                  |  signed

use std::collections::HashMap;

use crate::*;
use nom::{bytes::complete::tag, multi::many0, sequence::terminated, IResult};
use nom_tracable::tracable_parser;

impl Default for Wire {
    fn default() -> Self {
        Self {
            width: 1,
            offset: 0,
            input: false,
            output: false,
            inout: false,
            upto: false,
            signed: false,
            attributes: HashMap::new(),
        }
    }
}

/// <wire>          ::= <attr-stmt>* <wire-stmt>
#[tracable_parser]
pub fn wire(input: Span) -> IResult<Span, (String, Wire)> {
    let (input, attrs) = many0(attribute::attr_stmt)(input)?;
    let (input, mut wire) = wire_stmt(input)?;
    wire.1.attributes = attrs.into_iter().collect();
    Ok((input, wire))
}

/// <wire-stmt>     ::= wire <wire-option>* <wire-id> <eol>
pub fn wire_stmt(input: Span) -> IResult<Span, (String, Wire)> {
    let (input, _) = tag("wire")(input)?;
    let (input, _) = characters::sep(input)?;
    // with sep for each
    let (input, wire_options) = many0(terminated(wire_option, characters::sep))(input)?;
    let (input, id) = identifier::id(input)?;
    let (input, _) = characters::eol(input)?;
    let mut wire = Wire::default();
    for option in wire_options {
        match option {
            WireOption::Width(width) => wire.width = width,
            WireOption::Offset(offset) => wire.offset = offset,
            WireOption::Input => wire.input = true,
            WireOption::Output => wire.output = true,
            WireOption::Inout => wire.inout = true,
            WireOption::Upto => wire.upto = true,
            WireOption::Signed => wire.signed = true,
        }
    }
    Ok((input, (id.to_string(), wire)))
}

#[derive(Debug, Clone, PartialEq)]
enum WireOption {
    Width(usize),
    Offset(usize),
    Input,
    Output,
    Inout,
    Upto,
    Signed,
}

fn wire_option(input: Span) -> IResult<Span, WireOption> {
    let (input, option) = nom::branch::alt((
        tag("width"),
        tag("offset"),
        tag("input"),
        tag("output"),
        tag("inout"),
        tag("upto"),
        tag("signed"),
    ))(input)?;
    // sep on width, offset, input, output, inout
    let input = match *option.fragment() {
        "width" | "offset" | "input" | "output" | "inout" => {
            let (_input, _) = characters::sep(input)?;
            _input
        }
        _ => input,
    };
    match *option {
        "width" => {
            let (input, width) = value::integer(input)?;
            Ok((input, WireOption::Width(width as usize)))
        }
        "offset" => {
            let (input, offset) = value::integer(input)?;
            Ok((input, WireOption::Offset(offset as usize)))
        }
        "input" => {
            let (input, _input_val) = value::integer(input)?;
            Ok((input, WireOption::Input))
        }
        "output" => {
            let (input, _output) = value::integer(input)?;
            Ok((input, WireOption::Output))
        }
        "inout" => {
            let (input, _inout) = value::integer(input)?;
            Ok((input, WireOption::Inout))
        }
        "upto" => Ok((input, WireOption::Upto)),
        "signed" => Ok((input, WireOption::Signed)),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_wire_stmt() {
        let vectors = [
            (
                "wire $a\n",
                (
                    "a".to_string(),
                    Wire {
                        width: 1,
                        offset: 0,
                        input: false,
                        output: false,
                        inout: false,
                        upto: false,
                        signed: false,
                        attributes: HashMap::new(),
                    },
                ),
            ),
            (
                "wire width 1 $a\n",
                (
                    "a".to_string(),
                    Wire {
                        width: 1,
                        offset: 0,
                        input: false,
                        output: false,
                        inout: false,
                        upto: false,
                        signed: false,
                        attributes: HashMap::new(),
                    },
                ),
            ),
            (
                "wire offset 1 $a\n",
                (
                    "a".to_string(),
                    Wire {
                        width: 1,
                        offset: 1,
                        input: false,
                        output: false,
                        inout: false,
                        upto: false,
                        signed: false,
                        attributes: HashMap::new(),
                    },
                ),
            ),
            (
                "wire input 10 $a\n",
                (
                    "a".to_string(),
                    Wire {
                        width: 1,
                        offset: 0,
                        input: true,
                        output: false,
                        inout: false,
                        upto: false,
                        signed: false,
                        attributes: HashMap::new(),
                    },
                ),
            ),
            (
                "wire output 5 $a\n",
                (
                    "a".to_string(),
                    Wire {
                        width: 1,
                        offset: 0,
                        input: false,
                        output: true,
                        inout: false,
                        upto: false,
                        signed: false,
                        attributes: HashMap::new(),
                    },
                ),
            ),
            (
                "wire inout 5 $a\n",
                (
                    "a".to_string(),
                    Wire {
                        width: 1,
                        offset: 0,
                        input: false,
                        output: false,
                        inout: true,
                        upto: false,
                        signed: false,
                        attributes: HashMap::new(),
                    },
                ),
            ),
            (
                "wire upto $a\n",
                (
                    "a".to_string(),
                    Wire {
                        width: 1,
                        offset: 0,
                        input: false,
                        output: false,
                        inout: false,
                        upto: true,
                        signed: false,
                        attributes: HashMap::new(),
                    },
                ),
            ),
        ];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = wire_stmt(span);
            assert!(ret.is_ok(), "failed: {}", input);
            assert_eq!(ret.unwrap().1, expected, "failed: {}", input);
        }
    }

    #[test]
    fn test_wire_option() {
        let vectors = vec![
            ("width 1", WireOption::Width(1)),
            ("offset 0", WireOption::Offset(0)),
            ("input 1", WireOption::Input),
            ("output 1", WireOption::Output),
            ("inout 1", WireOption::Inout),
            ("upto", WireOption::Upto),
            ("signed", WireOption::Signed),
        ];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = wire_option(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }
}
