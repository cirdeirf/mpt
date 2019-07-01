use crate::pta::{Transition, PTA};
use nom::{
    alt, alt_complete, call, complete, delimited, do_parse, escaped, expr_res,
    is_not, is_space, many0, map_res, named, one_of, opt, rest, tag,
    take_while, IResult,
};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::str::{from_utf8, FromStr};

impl<Q, T> FromStr for PTA<Q, T>
where
    Q: Eq + Hash + Clone + FromStr,
    T: Eq + Hash + Clone + FromStr + Display,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut root_pr = HashMap::new();
        let mut transitions: Vec<Transition<Q, T>> = Vec::new();

        for l in s.lines() {
            if l.trim_start().starts_with("root:") {
                match parse_root_pr(l.trim_start().as_bytes()) {
                    Ok((_, (q, w))) => {
                        if root_pr.insert(q, w).is_some() {
                            return Err(format!(
                                "State has multiple root probabilities \
                                 assigned: {}",
                                l
                            ));
                        };
                    }
                    _ => {
                        return Err(format!(
                            "Malformed root probability declaration: {}",
                            l
                        ));
                    }
                }
            } else if !l.is_empty() && !l.trim_start().starts_with('%') {
                let t: Transition<Q, T> = l.trim().parse()?;
                transitions.push(t);
            }
        }
        match (root_pr, transitions) {
            (ref r, ref tr) if r.is_empty() || tr.is_empty() => Err(
                "Incomplete pta definition (root weights and transitions are \
                 necessary)"
                    .to_string(),
            ),
            (root_pr, transitions) => Ok(PTA::new(root_pr, transitions)),
        }
    }
}

impl<Q, T> FromStr for Transition<Q, T>
where
    Q: FromStr,
    T: FromStr,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_transition(s.as_bytes()) {
            Ok((_, result)) => Ok(result),
            _ => Err(format!("Could not parse: {}", s)),
        }
    }
}

/// Parses a transition.
/// A transition has to be of the form `transition: q0 -> NP(q1, q2)`.
fn parse_transition<Q, T>(input: &[u8]) -> IResult<&[u8], Transition<Q, T>>
where
    Q: FromStr,
    T: FromStr,
{
    do_parse!(
        input,
        tag!("transition:")
            >> take_while!(is_space)
            >> source_state: parse_token
            >> take_while!(is_space)
            >> alt!(tag!("->") | tag!("→"))
            >> take_while!(is_space)
            >> symbol: parse_token
            >> take_while!(is_space)
            >> target_states:
                call!(|x| parse_vec(x, parse_token, "(", ")", ","))
            >> take_while!(is_space)
            >> probability: parse_weight
            >> opt!(complete!(do_parse!(
                take_while!(is_space) >> parse_comment >> ()
            )))
            >> (Transition {
                source_state: source_state,
                symbol: symbol,
                target_states: target_states,
                probability: probability,
            })
    )
}

/// Parses a token (i.e. a terminal symbol or a non-terminal symbol).
/// A *token* can be of one of the following two forms:
///
/// * It is a string containing neither of the symbols
/// `'"'`, `' '`, `'-'`, `'→'`, `','`, `';'`, `')'`, `']'`, `'%'`.
/// * It is delimited by the symbol `'"'` on both sides
/// and each occurrence of `'\\'` or `'"'` inside the delimiters is escaped.
///
/// ([rustomata](https://github.com/tud-fop/rustomata)/src/util/parsing.rs)
fn parse_token<A>(input: &[u8]) -> IResult<&[u8], A>
where
    A: FromStr,
{
    named!(
        parse_token_s<&str>,
        map_res!(
            alt!(
                delimited!(
                    tag!("\""),
                    escaped!(is_not!("\"\\"), '\\', one_of!("\\\"")),
                    tag!("\"")
                ) | is_not!(" \\\"-→,;()]%#")
            ),
            from_utf8
        )
    );

    do_parse!(
        input,
        output: parse_token_s >> token: expr_res!(output.parse()) >> (token)
    )
}

/// Parses the `input` into a `Vec<A>` given an `inner_parser` for type `A`, an
/// `opening` delimiter, a `closing` delimiter, and a `separator`.
/// The `inner_parser` must not consume the `separator`s or the `closing`
/// delimiter of the given `input`.
///
/// ([rustomata](https://github.com/tud-fop/rustomata)/src/util/parsing.rs)
fn parse_vec<'a, A, P>(
    input: &'a [u8],
    inner_parser: P,
    opening: &str,
    closing: &str,
    separator: &str,
) -> IResult<&'a [u8], Vec<A>>
where
    P: Fn(&'a [u8]) -> IResult<&'a [u8], A>,
{
    do_parse!(
        input,
        tag!(opening)
            >> take_while!(is_space)
            >> result:
                many0!(do_parse!(
                    opt!(tag!(separator))
                        >> take_while!(is_space)
                        >> the_token: inner_parser
                        >> take_while!(is_space)
                        >> (the_token)
                ))
            >> tag!(closing)
            >> (result)
    )
}

/// Parses a weight annotation, i.e., a single `#` followed by a floating point
/// number (e.g. `# 0.42`).
fn parse_weight<W>(input: &[u8]) -> IResult<&[u8], W>
where
    W: FromStr,
    W::Err: Debug,
{
    do_parse!(
        input,
        w: complete!(do_parse!(
            tag!("#")
                >> take_while!(is_space)
                >> pr: map_res!(
                    alt_complete!(is_not!(" \n") | rest),
                    from_utf8
                )
                >> (pr.parse().unwrap())
        )) >> (w)
    )
}

/// Parses a string of the form `root: q0 # 0.43`.
///
/// ([rustomata](https://github.com/tud-fop/rustomata)/src/util/parsing.rs)
fn parse_root_pr<Q, W>(input: &[u8]) -> IResult<&[u8], (Q, W)>
where
    W: FromStr,
    W::Err: Debug,
    Q: FromStr,
{
    do_parse!(
        input,
        tag!("root:")
            >> take_while!(is_space)
            >> q: parse_token
            >> take_while!(is_space)
            >> w: parse_weight
            >> opt!(complete!(do_parse!(
                take_while!(is_space) >> parse_comment >> ()
            )))
            >> (q, w)
    )
}

/// Consumes any string that begins with the character `%`.
///
/// ([rustomata](https://github.com/tud-fop/rustomata)/src/util/parsing.rs)
fn parse_comment(input: &[u8]) -> IResult<&[u8], ()> {
    do_parse!(input, tag!("%") >> take_while!(|_| true) >> (()))
}
