//! A value consists of a width in bits and a bit representation, most significant bit first. Bits may be any of:
//! * 0: A logic zero value
//! * 1: A logic one value
//! * x: An unknown logic value (or don’t care in case patterns)
//! * z: A high-impedance value (or don’t care in case patterns)
//! * m: A marked bit (internal use only)
//! * -: A don’t care value

use nom::{
    bytes::complete::tag,
    character::complete::one_of,
    combinator::opt,
    multi::{many0, many1},
    IResult,
};

use crate::Span;

/// `<decimal-digit> ::= 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9`
fn decimal_digit(input: Span) -> IResult<Span, char> {
    one_of("0123456789")(input)
}

/// `<binary-digit>  ::= 0 | 1 | x | z | m | -`
fn binary_digit(input: Span) -> IResult<Span, char> {
    one_of("01xzXZmM-")(input)
}

/// <integer>       ::= -? <decimal-digit>+
pub(crate) fn integer(input: Span) -> IResult<Span, i32> {
    let (input, sign) = opt(tag("-"))(input)?;
    // use decimal_digit
    let (input, digits) = many1(decimal_digit)(input)?;
    // parse the digits as a string and then parse the string as an i64
    let digits: String = digits.into_iter().collect();
    let integer = digits.parse::<i32>().unwrap();
    // if the sign is present, negate the integer
    let integer = if sign.is_some() { -integer } else { integer };
    Ok((input, integer))
}

/// <value>         ::= <decimal-digit>+ ' <binary-digit>*
pub(crate) fn value(input: Span) -> IResult<Span, Vec<char>> {
    let (input, digits) = many1(decimal_digit)(input)?;
    let (input, _) = tag("'")(input)?;
    let (input, binary_digits) = many0(binary_digit)(input)?;
    let parsed_size = digits.iter().collect::<String>().parse::<i64>().unwrap();
    if parsed_size != binary_digits.len() as i64 {
        // TODO: will assume that when there only 1 digi, extend. Otherwise, panic
        if binary_digits.len() == 1 {
            let binary_digits = vec![binary_digits[0]; parsed_size as usize];
            return Ok((input, binary_digits));
        } else {
            dbg!(parsed_size, binary_digits);
            unimplemented!("Size of value does not match the number of bits");
        }
    }
    let binary_digits: Vec<char> = binary_digits.into_iter().rev().collect();
    Ok((input, binary_digits))
}

#[cfg(test)]
mod tests {
    use nom_locate::LocatedSpan;
    use nom_tracable::TracableInfo;

    use super::*;

    #[test]
    fn test_decimal_digit() {
        let inputs = [
            ("0", '0'),
            ("1", '1'),
            ("2", '2'),
            ("3", '3'),
            ("4", '4'),
            ("5", '5'),
            ("6", '6'),
            ("7", '7'),
            ("8", '8'),
            ("9", '9'),
        ];
        let info = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in inputs.iter().enumerate() {
            let span = LocatedSpan::new_extra(*input, info);
            let ret = decimal_digit(span).unwrap();
            assert_eq!(ret.1, *expected, "Test case {}", i);
        }
        // test not:
        let span = LocatedSpan::new_extra("a9", info);
        let ret = decimal_digit(span);
        assert!(ret.is_err());
    }

    #[test]
    fn test_binary_digit() {
        let inputs = [
            ("0", '0'),
            ("1", '1'),
            ("x", 'x'),
            ("X", 'X'),
            ("z", 'z'),
            ("Z", 'Z'),
            ("m", 'm'),
            ("M", 'M'),
            ("-", '-'),
        ];
        let info = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in inputs.iter().enumerate() {
            let span = LocatedSpan::new_extra(*input, info);
            let ret = binary_digit(span).unwrap();
            assert_eq!(ret.1, *expected, "Test case {}", i);
        }
    }

    #[test]
    fn test_integer() {
        let inputs = [
            ("0", 0),
            ("1", 1),
            ("2", 2),
            ("3", 3),
            ("4", 4),
            ("5", 5),
            ("6", 6),
            ("7", 7),
            ("8", 8),
            ("9", 9),
            ("-0", 0),
            ("-1", -1),
            ("-2", -2),
            ("-3", -3),
            ("-4", -4),
            ("-5", -5),
            ("-6", -6),
            ("-7", -7),
            ("-8", -8),
            ("-9", -9),
            ("00", 0),
            ("01", 1),
            ("02", 2),
            ("03", 3),
            ("04", 4),
            ("05", 5),
            ("06", 6),
            ("07", 7),
            ("08", 8),
            ("09", 9),
            ("-00", 0),
            ("-01", -1),
            ("-02", -2),
            ("-03", -3),
            ("-04", -4),
            ("-05", -5),
            ("1234567890", 1234567890),
        ];
        let info = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in inputs.iter().enumerate() {
            let span = LocatedSpan::new_extra(*input, info);
            let ret = integer(span).unwrap();
            assert_eq!(ret.1, *expected, "Test case {}", i);
        }
    }

    #[test]
    fn test_value() {
        let vectors = [
            ("1'0", vec!['0']),
            ("1'1", vec!['1']),
            ("1'x", vec!['x']),
            ("1'z", vec!['z']),
            ("1'm", vec!['m']),
            ("1'-", vec!['-']),
            ("2'01", vec!['1', '0']),
            ("3'101", vec!['1', '0', '1']),
            ("4'010x", vec!['x', '0', '1', '0']),
            ("0'x", vec![]), // no vec since constant is 0-wide
        ];
        let info: TracableInfo = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in vectors.iter().enumerate() {
            let span = LocatedSpan::new_extra(*input, info);
            let ret = value(span).unwrap();
            assert_eq!(ret.1, *expected, "Test case {}", i);
        }
    }
    // should fail if the number of bits does not match the number of bits
    #[test]
    #[should_panic]
    fn test_value_panic() {
        let info = TracableInfo::new().parser_width(64).fold("term");
        let span = LocatedSpan::new_extra("3'01", info);
        let _ = value(span).unwrap();
    }
}
