//! A string is a series of characters delimited by double-quote characters. Within a string, any character except ASCII NUL (0) may be used. In addition, certain escapes can be used:
//! * \n: A newline
//! * \t: A tab
//! * \ooo: A character specified as a one, two, or three digit octal value
//! All other characters may be escaped by a backslash, and become the following character. Thus:
//! * \\: A backslash
//! * \": A double-quote
//! * \r: An ‘r’ character
//! Comments
//! A comment starts with a # character and proceeds to the end of the line. All comments are ignored.

use crate::Span;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while, take_while_m_n},
    character::complete::{char, multispace1},
    combinator::{map, value, verify},
    error::{ErrorKind, FromExternalError, ParseError},
    multi::fold_many0,
    sequence::{delimited, preceded},
    AsChar, IResult, Parser,
};
use nom_tracable::tracable_parser;

// parser combinators are constructed from the bottom up:
// first we write parsers for the smallest elements (escaped characters),
// then combine them into larger parsers.

/// Parse a seq of octal
fn parse_seq<'a, E>(input: Span<'a>) -> IResult<Span, char, E>
where
    E: ParseError<Span<'a>>,
{
    // `take_while_m_n` parses between `m` and `n` bytes (inclusive) that match
    // a predicate. `parse_oct` here parses between 1 and 3 oct digits.
    let (input, seq) = take_while_m_n(1, 3, |c: char| c.is_oct_digit())(input)?;
    let seq = seq.fragment();
    match u8::from_str_radix(seq, 8) {
        Ok(v) => Ok((input, v as char)),
        Err(_e) => Err(nom::Err::Failure(E::from_error_kind(
            input,
            ErrorKind::IsNot,
        ))),
    }
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
fn parse_escaped_char<'a, E>(input: Span<'a>) -> IResult<Span, char, E>
where
    E: ParseError<Span<'a>> + FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    preceded(
        char('\\'),
        // `alt` tries each parser in sequence, returning the result of
        // the first successful match
        alt((
            parse_seq,
            // The `value` parser returns a fixed value (the first argument) if its
            // parser (the second argument) succeeds. In these cases, it looks for
            // the marker characters (n, r, t, etc) and returns the matching
            // character (\n, \r, \t, etc).
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('"', char('"')),
        )),
    )
    .parse(input)
}

/// Parse a backslash, followed by any amount of whitespace. This is used later
/// to discard any escaped whitespace.
fn parse_escaped_whitespace<'a, E: ParseError<Span<'a>>>(
    input: Span<'a>,
) -> IResult<Span, Span, E> {
    preceded(char('\\'), multispace1).parse(input)
}

/// Parse a non-empty block of text that doesn't include \ or "
fn parse_literal<'a, E: ParseError<Span<'a>>>(input: Span<'a>) -> IResult<Span, Span, E> {
    // `is_not` parses a string of 0 or more characters that aren't one of the
    // given characters.
    let not_quote_slash = is_not("\"\\");

    // `verify` runs a parser, then runs a verification function on the output of
    // the parser. The verification function accepts out output only if it
    // returns true. In this case, we want to ensure that the output of is_not
    // is non-empty.
    verify(not_quote_slash, |s: &Span| !s.is_empty()).parse(input)
}

/// A string fragment contains a fragment of a string being parsed: either
/// a non-empty Literal (a series of non-escaped characters), a single
/// parsed escaped character, or a block of escaped whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
}

/// Combine parse_literal, parse_escaped_whitespace, and parse_escaped_char
/// into a StringFragment.
fn parse_fragment<'a, E>(input: Span<'a>) -> IResult<Span, StringFragment<'a>, E>
where
    E: ParseError<Span<'a>> + FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    alt((
        // The `map` combinator runs a parser, then applies a function to the output
        // of that parser.
        map(parse_literal, |f| StringFragment::Literal(f.fragment())),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, parse_escaped_whitespace),
    ))
    .parse(input)
}

/// Parse a string. Use a loop of parse_fragment and push all of the fragments
/// into an output string.
fn parse_string<'a, E>(input: Span<'a>) -> IResult<Span, String, E>
where
    E: ParseError<Span<'a>> + FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    // fold is the equivalent of iterator::fold. It runs a parser in a loop,
    // and for each output value, calls a folding function on each output value.
    let build_string = fold_many0(
        // Our parser function– parses a single string fragment
        parse_fragment,
        // Our init value, an empty string
        String::new,
        // Our folding function. For each fragment, append the fragment to the
        // string.
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s),
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        },
    );

    // Finally, parse the string. Note that, if `build_string` could accept a raw
    // " character, the closing delimiter " would never match. When using
    // `delimited` with a looping parser (like fold), be sure that the
    // loop won't accidentally match your closing delimiter!
    delimited(char('"'), build_string, char('"')).parse(input)
}

/// A string is a series of characters delimited by double-quote characters. Within a string, any character except ASCII NUL (0) may be used. In addition, certain escapes can be used:
/// * \n: A newline
/// * \t: A tab
/// * \ooo: A character specified as a one, two, or three digit octal value
#[tracable_parser]
pub fn string(s: Span) -> IResult<Span, String> {
    let (input, this_string) = parse_string(s)?;
    Ok((input, this_string))
}

#[tracable_parser]
#[inline]
pub fn comment(input: Span) -> IResult<Span, String> {
    let (input, _) = tag("#")(input)?;
    let (input, this_comment) = take_while(|c| c != '\n' && c != '\r')(input)?;
    let (input, _) = alt((tag("\n"), tag("\r\n"), tag("\r")))(input)?;
    Ok((input, this_comment.fragment().to_string()))
}

#[cfg(test)]
mod tests {
    use nom_locate::LocatedSpan;
    use nom_tracable::TracableInfo;

    use super::*;

    #[test]
    fn test_string() {
        let vectors = vec![
            ("\"hello\"", "hello"),
            ("\"A\"", "A"),
            ("\"1\"", "1"),
            ("\" \"", " "),
            ("\"\"", ""),
            ("\"\\\"\"", "\""),
            ("\"\\n\"", "\n"),
            ("\"\\t\"", "\t"),
            ("\"\\r\"", "\r"),
        ];
        let info = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in vectors.iter().enumerate() {
            let span = LocatedSpan::new_extra(*input, info);
            let ret = string(span).unwrap();
            assert_eq!(ret.1, *expected, "Test case {}", i);
        }
    }

    #[test]
    fn test_comment() {
        let vectors = vec![
            ("#a\n", "a"),
            ("#A\n", "A"),
            ("#1\r", "1"),
            ("# \r", " "),
            ("#\n", ""),
            ("#\r", ""),
            ("#\r\n", ""),
            (
                "# Generated by Yosys 0.39 (git sha1 00338082b00, clang++ 15.0.0 -fPIC -Os)\n",
                " Generated by Yosys 0.39 (git sha1 00338082b00, clang++ 15.0.0 -fPIC -Os)",
            ),
        ];
        let info = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in vectors.iter().enumerate() {
            let span = LocatedSpan::new_extra(*input, info);
            let ret = comment(span).unwrap();
            assert_eq!(ret.1, *expected, "Test case {}", i);
        }
    }

    #[test]
    fn test_string_escaped() {
        let vectors = [
            ("\"\\000\"", "\0"),
            ("\"\\001\"", "\x01"),
            ("\"\\010\"", "\x08"),
            ("\"\\010 \"", "\x08 "),
        ];
        let info = TracableInfo::new().parser_width(64).fold("term");
        for (i, (input, expected)) in vectors.iter().enumerate() {
            let span = LocatedSpan::new_extra(*input, info);
            let ret = string(span).unwrap();
            assert_eq!(ret.1, *expected, "Test case {}", i);
        }
    }
}
