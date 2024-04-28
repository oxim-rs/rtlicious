//! Constant parser
//! ```text
//! <constant>          ::= <value> | <integer> | <string>
//! ```

use crate::{string, value, Constant, Span};
use nom::{branch::alt, combinator::map, IResult};
use nom_tracable::tracable_parser;

/// <constant>          ::= <value> | <integer> | <string>
#[tracable_parser]
pub(crate) fn constant(input: Span) -> IResult<Span, Constant> {
    // map the result of the alt combinator to the Constant enum
    let (input, constant) = alt((
        // if the input is a value, return a Constant::Value
        map(value::value, Constant::Value),
        // if the input is an integer, return a Constant::Integer
        map(value::integer, Constant::Integer),
        // if the input is a string, return a Constant::String
        map(string::string, Constant::String),
    ))(input)?;
    Ok((input, constant))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constant() {
        let vectors = [
            ("-129", Constant::Integer(-129)),
            (
                "\"hello world\"",
                Constant::String("hello world".to_string()),
            ),
            ("4'x", Constant::Value(vec!['x', 'x', 'x', 'x'])),
        ];
        for (input, expected) in vectors.iter() {
            let input = Span::new_extra(input, Default::default());
            let result = constant(input);
            let ret = result.unwrap_or_else(|e| {
                panic!(
                    "Failed to parse input: {:?}, error: {:?}",
                    input.fragment(),
                    e
                )
            });
            assert_eq!(ret.1, *expected);
        }
    }
}
