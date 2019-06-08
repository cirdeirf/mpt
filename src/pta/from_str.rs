use crate::pta::{Transition, PTA};
use log_domain::LogDomain;
use nom::{
    alt, call, delimited, do_parse, escaped, expr_res, is_not, is_space, many0,
    map_res, named, one_of, opt, tag, take_while, IResult,
};
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::{from_utf8, FromStr};

impl FromStr for PTA {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut root_pr = Vec::new();
        let mut transitions: HashMap<char, Vec<Transition>> = HashMap::new();

        let mut it = s.lines();

        while let Some(l) = it.next() {
            if l.trim_start().starts_with("root:") {
                match parse_root_pr(l.trim_start().as_bytes()) {
                    Ok((_, result)) => {
                        root_pr = result;
                    }
                    _ => {
                        return Err(format!(
                            "Malformed final declaration: {}",
                            l
                        ));
                    }
                }
            } else if !l.is_empty() && !l.trim_start().starts_with("%") {
                let t: Transition = l.trim().parse()?;
                // TODO maybe 2 calls on transitions is enough
                if transitions.contains_key(&t.symbol) {
                    transitions.get_mut(&t.symbol).unwrap().push(t);
                } else {
                    transitions.insert(t.symbol, vec![t]);
                }
            }
        }
        match (root_pr, transitions) {
            (ref r, ref tr) if r.len() == 0 || tr.len() == 0 => {
                Err(format!("foo"))
            }
            (r, tr) => Ok(PTA::new(r, tr)),
        }
    }
}

impl FromStr for Transition {
    type Err = String;

    fn from_str(st: &str) -> Result<Self, Self::Err> {
        let e: String = "Malformed state.".to_string();
        let s = st.replace(",", "");
        let s = s.replace("#", "");
        let v: Vec<&str> = s.split(|c| c == '(' || c == ')').collect();
        let source_and_symbol: Vec<&str> = v[0].split_whitespace().collect();
        let t: Vec<&str> = v[1].split_whitespace().collect();
        let mut targets: Vec<usize> = Vec::new();
        for q in t {
            targets.push(q.parse().map_err(|_| e.clone())?);
        }
        let pr: LogDomain<f64> = v[2].trim().parse().map_err(|_| e.clone())?;
        if v.len() == 3 && source_and_symbol.len() == 4 {
            match source_and_symbol[2] {
                "->" | "→" => Ok(Transition {
                    source_state: source_and_symbol[1]
                        .parse()
                        .map_err(|_| e.clone())?,
                    symbol: source_and_symbol[3]
                        .parse()
                        .map_err(|_| e.clone())?,
                    target_states: targets,
                    probability: pr,
                }),
                _ => Err(format!("Transition malformed: {}", st)),
            }
        } else {
            Err(format!("Transition malformed: {}", st))
        }
    }
}

/// Parses a token (i.e. a terminal symbol or a non-terminal symbol).
/// A *token* can be of one of the following two forms:
///
/// * It is a string containing neither of the symbols `'"'`, `' '`, `'-'`, `'→'`, `','`, `';'`, `')'`, `']'`, `'%'`.
/// * It is delimited by the symbol `'"'` on both sides and each occurrence of `'\\'` or `'"'` inside the delimiters is escaped.
pub fn parse_token<A>(input: &[u8]) -> IResult<&[u8], A>
where
    A: FromStr,
    A::Err: Debug,
{
    named!(
        parse_token_s<&str>,
        map_res!(
            alt!(
                delimited!(
                    tag!("\""),
                    escaped!(is_not!("\"\\"), '\\', one_of!("\\\"")),
                    tag!("\"")
                ) | is_not!(" \\\"-→,;)]%#")
            ),
            from_utf8
        )
    );

    do_parse!(
        input,
        output: parse_token_s >> token: expr_res!(output.parse()) >> (token)
    )
}

/// Parses the `input` into a `Vec<A>` given an `inner_parser` for type `A`, an `opening` delimiter, a `closing` delimiter, and a `separator`.
/// The `inner_parser` must not consume the `separator`s or the `closing` delimiter of the given `input`.
pub fn parse_vec<'a, A, P>(
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

/// TODO mention rustomata
/// Parses a string of the form `finals: [...]` as a vector of final symbols of type `I`.
pub fn parse_root_pr<I>(input: &[u8]) -> IResult<&[u8], Vec<I>>
where
    I: FromStr,
    I::Err: Debug,
{
    do_parse!(
        input,
        tag!("root:")
            >> take_while!(is_space)
            >> result: call!(|x| parse_vec(x, parse_token, "[", "]", ","))
            >> (result)
    )
}
