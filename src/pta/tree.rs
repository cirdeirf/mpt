use log_domain::LogDomain;
use nom::simple_errors::Context;
use nom::{
    alt, char, do_parse, many0, many1, named, separated_nonempty_list, tag,
    take_until_either, Err,
};
use num_traits::Zero;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[derive(Debug, Eq, Clone)]
pub struct Tree<A> {
    pub root: A,
    pub children: Vec<Tree<A>>,
    pub run: Vec<LogDomain<f64>>,
    pub probability: LogDomain<f64>,
    pub is_prefix: Option<bool>,
}

/// `impl` of `PartialEq` that ignores the `weight` (to conform to the `impl` of `Hash`)
impl<A> PartialEq for Tree<A>
where
    A: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root && self.children == other.children
    }
}

impl<A> Hash for Tree<A>
where
    A: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.root.hash(state);
        self.children.hash(state);
    }
}

impl<A> Tree<A>
where
    A: Eq + Hash,
{
    pub fn new(root_symbol: A) -> Tree<A> {
        Tree {
            root: root_symbol,
            children: Vec::new(),
            run: Vec::new(),
            is_prefix: None,
            probability: LogDomain::zero(),
        }
    }

    pub fn new_with_children(
        root_symbol: A,
        children: Vec<Tree<A>>,
    ) -> Tree<A> {
        let mut tree = Tree::new(root_symbol);
        tree.children = children;
        tree
    }

    pub fn get_height(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            self.children
                .iter()
                .map(|t| t.get_height() + 1)
                .max()
                .unwrap()
        }
    }

    pub fn extend(&mut self, s: A, sigma: &HashMap<A, usize>) -> bool {
        let mut t_stack = Vec::new();
        t_stack.push(self);
        loop {
            if t_stack.is_empty() {
                return false;
            } else {
                let t = t_stack.pop().unwrap();
                if &t.children.len() < sigma.get(&t.root).unwrap() {
                    t.children.push(Tree::new(s));
                    return true;
                } else {
                    for t_i in &mut t.children {
                        t_stack.push(t_i);
                    }
                }
            }
        }
    }

    // TODO use generics/shorten
    pub fn from_sexp(sexp: SExp) -> Tree<char> {
        let mut content = Vec::new();
        if let SExp::List(a) = sexp {
            content = a.to_vec();
        }
        let mut children: Vec<Tree<char>> = Vec::new();
        let mut symbol = 'a';
        for sxp in content {
            match sxp {
                SExp::Atom(s) => symbol = s.chars().collect::<Vec<char>>()[0],
                SExp::List(s) => {
                    children.push(Tree::<char>::from_sexp(SExp::List(s)))
                }
            }
        }
        Tree {
            root: symbol,
            children: children,
            run: Vec::new(),
            is_prefix: None,
            probability: LogDomain::zero(),
        }
    }
}

impl<A> fmt::Display for Tree<A>
where
    A: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn to_string<A>(xi: &Tree<A>) -> String
        where
            A: fmt::Display,
        {
            let mut ret = xi.root.to_string();
            if !xi.children.is_empty() {
                ret.push_str("( ");
                for t_i in &xi.children {
                    ret.push_str(&to_string(&t_i));
                    ret.push_str(", ");
                }
                ret.pop();
                ret.pop();
                ret.push_str(" )");
            }
            ret
        }
        write!(f, "{}", to_string(self))
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum SExp {
    Atom(String),
    List(Vec<SExp>),
}

// TODO malformed strings, etc., credit to felix
impl FromStr for SExp {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.as_bytes();

        named!(list<&[u8],SExp>,
            do_parse!(
                many0!(tag!(" ")) >>
                char!('(') >>
                many0!(tag!(" ")) >>
                conts: separated_nonempty_list!(many1!(tag!(" ")), sxpr) >> // originally: take_while!(is_space)
                many0!(tag!(" ")) >>
                char!(')') >>

                (SExp::List(conts))
            )
        );

        named!(atom<&[u8],SExp>, do_parse!(aa: take_until_either!(" )") >> (SExp::Atom(String::from_utf8(aa.to_vec()).unwrap()))));

        named!(sxpr<&[u8],SExp>, alt!(list | atom));

        match sxpr(input) {
            Ok(ex) => Ok(ex.1),
            #[cold]
            Err(e) => {
                match &e {
                    Err::Incomplete(_) => {
                        eprintln!("[Error] Parsing did not succeed: Incomplete Input Sequence!")
                    }
                    Err::Error(ref rest) | Err::Failure(ref rest) => {
                        eprintln!(
                            "[Error] Could not parse input string due to error: {}",
                            e.description()
                        );
                        let Context::Code(c, _) = rest;
                        eprintln!(
                            "[Error] Next to parse was: {}",
                            String::from_utf8(c.to_vec()).unwrap()
                        );
                    }
                }
                Err(e.to_string())
            }
        }
    }
}

impl FromStr for Tree<char> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Tree::<char>::from_sexp(s.parse()?))
    }
}
