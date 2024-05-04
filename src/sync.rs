//! Syncs
//! Syncs update signals with other signals when an event happens. Such an event may be:
//!     An edge or level on a signal
//!     Global clock ticks
//!     Initialization
//!     Always
//! ```text
//! <sync>          ::= <sync-stmt> <update-stmt>*
//! <sync-stmt>     ::= sync <sync-type> <sigspec> <eol>
//!                  |  sync global <eol>
//!                  |  sync init <eol>
//!                  |  sync always <eol>
//! <sync-type>     ::= low | high | posedge | negedge | edge
//! <update-stmt>   ::= update <dest-sigspec> <src-sigspec> <eol>
//! ```

use crate::*;
use nom::{branch::alt, bytes::complete::tag, combinator::map, multi::many0, IResult};
use nom_tracable::tracable_parser;

/// `<sync> ::= <sync-stmt> <update-stmt>*`
#[tracable_parser]
pub(crate) fn sync(input: Span) -> IResult<Span, Sync> {
    let (input, sync_event) = sync_stmt(input)?;
    let (input, updates) = many0(update_stmt)(input)?;
    let (input, memwrs) = many0(memwr_stmt)(input)?;
    Ok((
        input,
        Sync {
            sync_event,
            updates,
            memwrs: memwrs.into_iter().collect(),
        },
    ))
}

/// ```text
/// <sync-stmt>     ::= sync <sync-type> <sigspec> <eol>
///                  |  sync global <eol>
///                  |  sync init <eol>
///                  |  sync always <eol>
/// ```
#[tracable_parser]
pub(crate) fn sync_stmt(input: Span) -> IResult<Span, SyncOn> {
    let (input, _) = tag("sync")(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, sync_on) = alt((
        map(tag("global"), |_| SyncOn::Global),
        map(tag("init"), |_| SyncOn::Init),
        map(tag("always"), |_| SyncOn::Always),
        map(
            |input| {
                let (input, sync_type) = sync_type(input)?;
                let (input, _) = characters::sep(input)?;
                let (input, sigspec) = crate::sigspec::sigspec(input)?;
                Ok((input, SyncOn::Signal(sync_type, sigspec)))
            },
            |sync_on| sync_on,
        ),
    ))(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, sync_on))
}

/// `<sync-type>     ::= low | high | posedge | negedge | edge`
pub(crate) fn sync_type(input: Span) -> IResult<Span, SignalSync> {
    let (input, sync_type) = alt((
        map(tag("low"), |_| SignalSync::Low),
        map(tag("high"), |_| SignalSync::High),
        map(tag("posedge"), |_| SignalSync::Posedge),
        map(tag("negedge"), |_| SignalSync::Negedge),
        map(tag("edge"), |_| SignalSync::Edge),
    ))(input)?;
    Ok((input, sync_type))
}

/// `<update-stmt>   ::= update <dest-sigspec> <src-sigspec> <eol>`
#[tracable_parser]
pub(crate) fn update_stmt(input: Span) -> IResult<Span, (SigSpec, SigSpec)> {
    let (input, _) = tag("update")(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, dest) = crate::sigspec::sigspec(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, src) = crate::sigspec::sigspec(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((input, (dest, src)))
}

/// Undocumented memwr statement. looks like
/// `<memwr-stmt> ::= memwr <memid: id> <address: sigspec> <data: sigspec> <enable: sigspec> <priority_mask: sigspec> <eol>`
#[tracable_parser]
pub(crate) fn memwr_stmt(input: Span) -> IResult<Span, (String, Memwr)> {
    let (input, attributes) = many0(attribute::attr_stmt)(input)?;
    let (input, _) = tag("memwr")(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, memid) = identifier::id(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, address) = crate::sigspec::sigspec(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, data) = crate::sigspec::sigspec(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, enable) = crate::sigspec::sigspec(input)?;
    let (input, _) = characters::sep(input)?;
    let (input, priority_mask) = crate::sigspec::sigspec(input)?;
    let (input, _) = characters::eol(input)?;
    Ok((
        input,
        (
            memid.erease(),
            Memwr {
                attributes: attributes.into_iter().collect(),
                address,
                data,
                enable,
                priority_mask,
            },
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    #[test]
    fn test_sync() {
        let vectors = vec![(
            indoc! {r#"
                sync global
                update $a $b
                update $c $d
            "#},
            Sync {
                sync_event: SyncOn::Global,
                updates: vec![
                    (
                        SigSpec::WireId("a".to_string()),
                        SigSpec::WireId("b".to_string()),
                    ),
                    (
                        SigSpec::WireId("c".to_string()),
                        SigSpec::WireId("d".to_string()),
                    ),
                ],
                memwrs: HashMap::new(),
            },
        )];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = sync(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }

    #[test]
    fn test_sync_stmt() {
        let vectors = vec![
            ("sync global\n", SyncOn::Global),
            ("sync init\n", SyncOn::Init),
            ("sync always\n", SyncOn::Always),
            (
                "sync low \\EVENT\n",
                SyncOn::Signal(SignalSync::Low, SigSpec::WireId("EVENT".to_string())),
            ),
        ];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = sync_stmt(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }

    #[test]
    fn test_sync_type() {
        let vectors = vec![
            ("low", SignalSync::Low),
            ("high", SignalSync::High),
            ("posedge", SignalSync::Posedge),
            ("negedge", SignalSync::Negedge),
            ("edge", SignalSync::Edge),
        ];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = sync_type(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }

    #[test]
    fn test_update_stmt() {
        let vectors = vec![(
            "update \\DEST \\SRC\n",
            (
                SigSpec::WireId("DEST".to_string()),
                SigSpec::WireId("SRC".to_string()),
            ),
        )];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = update_stmt(span).unwrap();
            assert_eq!(ret.1, expected);
        }
    }

    #[test]
    fn test_memwr_stmt() {
        let vectors = vec![(
            indoc! {r#"
                memwr \ID $ADDR $DATA $EN 0'x
            "#},
            (
                "ID".to_string(),
                Memwr {
                    attributes: HashMap::new(),
                    address: SigSpec::WireId("ADDR".to_string()),
                    data: SigSpec::WireId("DATA".to_string()),
                    enable: SigSpec::WireId("EN".to_string()),
                    priority_mask: SigSpec::Constant(Constant::Value(vec![])), // no vec since constant is 0-wide
                },
            ),
        )];
        for (input, expected) in vectors {
            let span = Span::new_extra(input, Default::default());
            let ret = memwr_stmt(span).unwrap();
            assert_eq!(ret.0.fragment(), &"");
            assert_eq!(ret.1, expected);
        }
    }
}
