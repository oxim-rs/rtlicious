use crate::*;
use nom::{bytes::complete::tag, IResult};
use nom_tracable::tracable_parser;

///  <conn-stmt> ::= connect <sigspec> <sigspec> <eol>
#[tracable_parser]
pub(crate) fn conn_stmt(input: Span) -> IResult<Span, (SigSpec, SigSpec)> {
    let (input, _) = tag("connect")(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, sig1) = sigspec::sigspec(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, sig2) = sigspec::sigspec(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, (sig1, sig2)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom_tracable::TracableInfo;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_connect_mystery() {
        let input =
            "connect \\decode.co_immdec_ctrl [0] $flatten\\decode.$eq$serv_decode.v:213$842_Y\n";
        let info = TracableInfo::new().parser_width(64).fold("term");
        let span = Span::new_extra(input, info);
        assert_eq!(
            conn_stmt(span).unwrap().1,
            (
                SigSpec::Range(
                    Box::new(SigSpec::WireId("decode.co_immdec_ctrl".to_string())),
                    0,
                    None
                ),
                SigSpec::WireId("flatten\\decode.$eq$serv_decode.v:213$842_Y".to_string())
            )
        );
    }
}
