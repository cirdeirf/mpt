use log_domain::LogDomain;
use nom::simple_errors::Context;
use nom::{
    alt, char, do_parse, many0, many1, named, separated_nonempty_list, tag,
    take_until_either, Err,
};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

/// A tree ξ ∈ T_Σ(X) for a ranked alphabet Σ and a set of variables X.
#[derive(Debug, Eq, Clone)]
pub struct Tree<A> {
    pub root: A,
    pub children: Vec<Tree<A>>,
    /// each entry represents the probability of recognising this tree and
    /// ending up in the corresponding state, i.e.,
    /// ∀ q ∈ Q : ∑_{κ ∈ R(ξ) ∶ κ(ε) = q} Pr(κ).
    pub run: Vec<LogDomain<f64>>,
    /// ξ contains variables, i.e., ξ ∉ T_Σ
    pub is_prefix: bool,
}

/// `impl` of `PartialEq` that ignores everything except `root` and `children`
/// (to conform to the `impl` of `Hash`).
impl<A> PartialEq for Tree<A>
where
    A: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root && self.children == other.children
    }
}

/// `impl` of `Hash` that ignores everything except `root` and `children`
/// because floats like f64 are not hashable. This has to be done in order to
/// ensure that each tree's run probabilities are only calculated once.
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
    A: Eq + Hash + Clone,
{
    /// Instantiates a new Tree with a symbol at the root position and no
    /// children. It is assumed to be a prefix (normally this will be set
    /// immediately after instantiation if necessary)
    pub fn new(root: A) -> Tree<A> {
        Tree {
            root,
            children: Vec::new(),
            run: Vec::new(),
            is_prefix: true,
        }
    }

    /// Instantiates a new Tree with a symbol at the root position and a list of
    /// children.
    pub fn new_with_children(
        root_symbol: A,
        children: Vec<Tree<A>>,
    ) -> Tree<A> {
        let mut tree = Tree::new(root_symbol);
        tree.children = children;
        tree
    }

    /// Determines the height of the tree, i.e., the amount of nodes on the
    /// longest path from the root to a leaf.
    pub fn _get_height(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            self.children
                .iter()
                .map(|t| t._get_height() + 1)
                .max()
                .unwrap()
        }
    }

    /// Searches for the first variable in a breadth-first manner and replaces
    /// it with the given symbol σ. Returns true if the resulting tree remains a
    /// prefix (still contains variables) and false otherwise (ξ ∈ T_Σ).
    pub fn extend(&mut self, s: &A, sigma: &HashMap<A, usize>) -> bool {
        let mut prefix = false;
        let mut extended = false;
        let mut xi_stack = Vec::new();

        xi_stack.push(self);
        while !xi_stack.is_empty() {
            let xi = xi_stack.pop().unwrap();
            // there is at least one direct child such that ξ(i) ∈ X
            if xi.children.len() < *sigma.get(&xi.root).unwrap() {
                // in case ξ already has been extended and another variable is
                // found we know that ξ ∉ T_Σ
                if extended {
                    prefix = true;
                    break;
                } else {
                    xi.children.push(Tree::new((*s).clone()));
                    xi_stack.push(xi);
                }
                // only extend once
                extended = true;
            } else {
                // look at all children
                for xi_i in &mut xi.children {
                    xi_stack.push(xi_i);
                }
            }
        }
        prefix
    }

    /// Creates a tree from an S-expression.
    /// (Credit to Felix Wittwer)
    fn from_sexp(sexp: SExp) -> Tree<char> {
        let mut content = Vec::new();
        if let SExp::List(a) = sexp {
            content = a.to_vec();
        }
        let mut children: Vec<Tree<char>> = Vec::new();
        let mut root = 'a';
        for sxp in content {
            match sxp {
                SExp::Atom(s) => root = s.chars().collect::<Vec<char>>()[0],
                SExp::List(s) => {
                    children.push(Tree::<char>::from_sexp(SExp::List(s)))
                }
            }
        }
        Tree {
            root,
            children,
            run: Vec::new(),
            is_prefix: true,
        }
    }
}

/// Pretty print for trees.
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

/// Parse an S-expression.
/// (Credit to Felix Wittwer)
impl FromStr for SExp {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.as_bytes();

        named!(list<&[u8],SExp>,
            do_parse!(
                   many0!(tag!(" "))
                >> char!('(')
                >> many0!(tag!(" "))
                >> conts: separated_nonempty_list!(many1!(tag!(" ")), sxpr)
                >> many0!(tag!(" "))
                >> char!(')')

                >> (SExp::List(conts))
            )
        );

        named!(atom<&[u8],SExp>,
            do_parse!(
                   aa: take_until_either!(" )")
                >> (SExp::Atom(String::from_utf8(aa.to_vec()).unwrap()))
            )
        );

        named!(sxpr<&[u8],SExp>, alt!(list | atom));

        match sxpr(input) {
            Ok(ex) => Ok(ex.1),
            #[cold]
            Err(e) => {
                match &e {
                    Err::Incomplete(_) => eprintln!(
                        "[Error] Parsing did not succeed: \
                         Incomplete Input Sequence!"
                    ),
                    Err::Error(ref rest) | Err::Failure(ref rest) => {
                        eprintln!(
                            "[Error] Could not parse input string due to \
                             error: {}",
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
