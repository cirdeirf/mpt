//! Module for the probabilistic tree automaton, its transitions and trees that
//! can be recognised by a pta. The most probable tree and best parse algorithms
//! are part of the pta implementation.

pub mod experiments;
mod from_str;
mod transition;
mod tree;

use integeriser::{HashIntegeriser, Integeriser};
use log_domain::LogDomain;
use num_traits::Zero;
use priority_queue::PriorityQueue;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{self, Display, Write};
use std::fs;
use std::hash::Hash;
use std::path::Path;
use std::str::FromStr;
use transition::{Integerisable, Transition};
use tree::Tree;

/// A probabilistic tree automaton A = (Q, Σ, μ, ν).
pub struct PTA<Q, T>
where
    Q: Eq + Hash, // states (Q)
    T: Eq + Hash, // symbols (Σ)
{
    /// maps states (Q) to integers (usize) und vice versa
    q_integeriser: HashIntegeriser<Q>,
    /// maps symbols (T) to integers (usize) und vice versa
    t_integeriser: HashIntegeriser<T>,
    /// ranked alphabet Σ of tree symbols and their corresponding rank
    sigma: HashMap<T, usize>,
    /// number of states/size of the automaton |Q|
    number_states: usize,
    /// root weights ν: Q → [0, 1] represented as a list of probabilities
    root_weights: Vec<LogDomain<f64>>,
    /// transitions μ represented as mapping: T → Q → Transition<Q, T>
    transitions: HashMap<usize, HashMap<usize, Vec<Transition<usize, usize>>>>,
}

impl<Q, T> PTA<Q, T>
where
    Q: Eq + Hash + Clone,
    T: Eq + Hash + Clone + Display,
{
    /// Instantiates a new PTA from all non-null root weights and a list of
    /// transitions.
    /// TODO consistency check
    fn new(
        root_weight_map: HashMap<Q, LogDomain<f64>>,
        transitions_vec: Vec<Transition<Q, T>>,
    ) -> PTA<Q, T> {
        let mut q_integeriser = HashIntegeriser::new();
        let mut t_integeriser = HashIntegeriser::new();
        let mut sigma = HashMap::new();
        let mut root_weights: Vec<LogDomain<f64>> = Vec::new();
        let mut transitions: HashMap<
            usize,
            HashMap<usize, Vec<Transition<usize, usize>>>,
        > = HashMap::new();

        // construct the ranked alphabet by iterating over all transitions
        // the rank for a symbol σ is determined by the number of target states
        // in a transition for σ
        for t in &transitions_vec {
            sigma
                .entry(t.symbol.clone())
                .or_insert_with(|| t.target_states.len());
        }

        // fill the hashmap of transitions and integerise states and symbols
        for t in transitions_vec
            .into_iter()
            .map(|t| t.integerise(&mut q_integeriser, &mut t_integeriser))
        {
            transitions
                .entry(t.symbol)
                .or_insert_with(HashMap::new)
                .entry(t.source_state)
                .or_insert_with(Vec::new)
                .push(t);
        }

        // fill the root weight vector
        // (states not mentioned are assumed to have a null probability)
        for q in q_integeriser.values() {
            match root_weight_map.get(q) {
                Some(pr_q) => root_weights.push(*pr_q),
                None => root_weights.push(LogDomain::zero()),
            }
        }

        PTA {
            q_integeriser,
            t_integeriser,
            sigma,
            number_states: root_weights.len(),
            root_weights,
            transitions,
        }
    }

    /// Instantiate a new PTA from specifications given in a file.
    pub fn from_file(path: &Path) -> (PTA<Q, T>, String)
    where
        Q: FromStr,
        T: FromStr,
    {
        let pta_string = match fs::read_to_string(path) {
            Ok(file) => file,
            Err(e) => panic!(
                "Could not read pta file {}: {}.",
                path.display(),
                e.description()
            ),
        };
        if pta_string.starts_with('%') {
            (
                pta_string.parse().unwrap(),
                pta_string.lines().next().unwrap().to_string(),
            )
        } else {
            (pta_string.parse().unwrap(), "".to_string())
        }
    }

    /// Recursively computes the cumulative probability for all runs on ξ with
    /// the same state at the root position, i.e.,
    /// ∀ q ∈ Q : ∑_{κ ∈ R(ξ) ∶ κ(ε) = q} Pr(κ).
    fn probability_rec(
        &self,
        xi: &mut Tree<T>,
        mut known_trees: &mut HashSet<Tree<T>>,
    ) -> Vec<LogDomain<f64>> {
        // get probabilities for tree xi if they have been calculated before
        if known_trees.contains(&xi) {
            known_trees.get(&xi).unwrap().run.clone()
        } else {
            // gather all transitions that have xi.root as a symbol
            let transitions = self
                .transitions
                .get(&self.t_integeriser.find_key(&xi.root).unwrap())
                .unwrap();

            let mut ret: Vec<LogDomain<f64>> = Vec::new();
            // ∀ q ∈ Q (go from 0 to |Q| because states have been integerised)
            for q in 0..self.number_states {
                let mut p_q = LogDomain::zero();
                // check if there are any transitions with q as source state
                if let Some(v) = transitions.get(&q) {
                    // ∑_{κ ∈ R(ξ)∶ κ(ε) = q} Pr(κ)
                    for t in v {
                        // μ_σ(κ(1), ..., κ(k)) ⋅ ...
                        let mut p_t = t.probability;
                        // wt(κ|_1) ⋅ ... ⋅ wt(κ|_k)
                        for (i, q_i) in t.target_states.iter().enumerate() {
                            // ξ|_i ∈ X implies wt(κ|_i) = 1
                            if let Some(t_i) = xi.children.get_mut(i) {
                                p_t *= self
                                    .probability_rec(t_i, &mut known_trees)
                                    [*q_i];
                            }
                        }
                        p_q += p_t;
                    }
                }
                ret.push(p_q);
            }
            // save probabilities in the tree xi
            xi.run = ret.clone();
            known_trees.insert(xi.clone());
            ret
        }
    }

    /// Calculates the probability of a (prefix-)tree ξ ∈ T_Σ(X).
    /// Base case for the recursive computation done in fn probability_rec and
    /// applies root weights.
    fn probability(
        &self,
        xi: &mut Tree<T>,
        mut known_trees: &mut HashSet<Tree<T>>,
    ) -> LogDomain<f64> {
        // multiply the probability of runs ending in a state q by the
        // probability of a run ending in that state (root weight)
        self.probability_rec(xi, &mut known_trees)
            .iter()
            .zip(&self.root_weights)
            .map(|(&p_q, &root_q)| p_q * root_q)
            .sum()
    }

    /// Compute the potential probability PP(ξ) = min(|Q|²/height(ξ), Pr(ξ)).
    /// This is supposed to take the bound of Theorem X (TODO) into account
    /// similar to what is done in Definition 2 by de la Higuera and Oncina 2013
    /// [Definition 2, dlHO13b]. Currently not in use since the bound is not
    /// tight enough to affect the outcome.
    fn _potential_probability(
        &self,
        xi: &mut Tree<T>,
        mut known_trees: &mut HashSet<Tree<T>>,
    ) -> LogDomain<f64> {
        cmp::min(
            self.probability(xi, &mut known_trees),
            LogDomain::new(
                self.number_states.pow(2) as f64 / xi._get_height() as f64,
            )
            .unwrap(),
        )
    }

    /// Calculates the most probable tree.
    /// The algorithm, corresponding analysis and evaluation can be found in
    /// Section X (TODO) of my master's thesis. This is based on an algorithm
    /// for probabilistic finite state automaton provided by Algorithm 1 in
    /// ["Computing the Most Probable String with a Probabilistic Finite State
    /// Machine" by de la Higuera and Oncina,
    /// 2013](https://www.aclweb.org/anthology/W13-1801) [dlHO13b, Algorithm 1].
    pub fn most_probable_tree(
        &self,
    ) -> Result<(Tree<T>, LogDomain<f64>, usize), &str> {
        // priority queue of explored trees ξ ∈ T_Σ(X), sorted w.r.t. Pr(ξ)
        let mut q = PriorityQueue::new();
        let mut insertion_count = 0;
        // set of trees whose probability has already been calculated once
        let mut known_trees = HashSet::new();
        // the best complete tree ξ ∈ T_Σ (no variables) and its Pr in the queue
        // (this is to prevent exploring prefix trees with a worse Pr than the
        // current best because extending a tree never improves the probability)
        let mut current_best;
        let mut current_prop = LogDomain::zero();

        // initially fill the queue with trees consisting of one symbol since we
        // cannot start with an empty tree
        for (sigma, rank) in &self.sigma {
            let mut xi = Tree::new(sigma.clone());
            // since sigma has a rank of 0, xi is a complete tree/no prefix-tree
            xi.is_prefix = rank != &0;
            let pr = self.probability(&mut xi, &mut known_trees);
            q.push(xi, pr);
            insertion_count += 1;
        }
        // initialise with an arbitrary value (save the overhead of looking for
        // the current best complete tree consiting of one symbol)
        current_best = q.peek().unwrap().0.clone();

        while !q.is_empty() {
            let (xi, pr) = q.pop().unwrap();

            // ξ ∈ T_Σ
            if !xi.is_prefix {
                current_best = xi;
                current_prop = pr;
                break;
            }
            // ξ ∉ T_Σ (contains variables, i.e., is a prefix-tree/context)
            else {
                // extend ξ with every σ ∈ Σ
                for s in self.sigma.keys() {
                    let mut xi_s = xi.clone();

                    // replace the first occurence (breadth first) of x in ξ
                    // with σ and return wether it still contains any vaiables x
                    xi_s.is_prefix = xi_s.extend(s, &self.sigma);
                    let pr_xi_s = self.probability(&mut xi_s, &mut known_trees);

                    // do not add (prefix-)trees to the queue that are worse
                    // than the current best complete tree (extending trees can
                    // only result in the same or worse probability)
                    if pr_xi_s > current_prop {
                        // ξ ∈ T_Σ (t_s complete + better than the current best)
                        if !xi_s.is_prefix {
                            current_best = xi_s.clone();
                            current_prop = pr_xi_s;
                        }
                        q.push(xi_s, pr_xi_s);
                        insertion_count += 1;
                        // if insertion_count % 1000 == 0 {
                        //     eprintln!("{} \t {}", insertion_count, q.len());
                        // }
                        if insertion_count > 2e+7 as usize {
                            // eprintln!("abort");
                            return Err(
                                "Maximum number of insertions (20⁷) exceeded. \
                                 Calculation of most probable tree aborted.",
                            );
                        }
                    }
                }
            }
        }
        Ok((current_best, current_prop, insertion_count))
    }

    /// Dertermines the best/most probable parse.
    /// Return the corrresponding tree and the run's probability.
    /// This implementation is based on the BestParse algorithm depicted in
    /// Figure 3 of ["Parsing Algorithms based on Tree Automata" by Maletti and
    /// Satta, 2009](https://www.aclweb.org/anthology/W09-3801)
    /// [MS09, Figure 3].
    pub fn best_parse(&self) -> (Tree<T>, LogDomain<f64>) {
        // flatten HashMaps, gather all transitions in one vector
        let transitions: Vec<Transition<usize, usize>> = self
            .transitions
            .values()
            .flat_map(|h| {
                h.values()
                    .cloned()
                    .collect::<Vec<Vec<Transition<usize, usize>>>>()
            })
            .flatten()
            .collect();

        // get all root states (states with non-null root weight)
        let root_states: HashSet<usize> = self
            .root_weights
            .iter()
            .enumerate()
            .filter(|(_, &p)| p != LogDomain::zero())
            .map(|(q, _)| q)
            .collect();

        // set of states available for application in new transitions
        let mut explored_states: HashSet<usize> = HashSet::new();
        // probabilities that can be achieved for a run that ends in given state
        let mut best_probabilities: Vec<LogDomain<f64>> =
            vec![LogDomain::zero(); self.number_states];
        // best trees that can be obtained for a run that ends in given state
        let mut best_trees: Vec<Option<Tree<T>>> =
            vec![None; self.number_states];

        // apply transitions until all root states are explored
        while !root_states.is_subset(&explored_states) {
            // set of states that are not yet explored but can be in one step
            let reachable_states: HashSet<usize> = transitions
                .iter()
                .filter(|t| {
                    !explored_states.contains(&t.source_state)
                        && t.target_states
                            .iter()
                            .cloned()
                            .collect::<HashSet<usize>>()
                            .is_subset(&explored_states)
                })
                .map(|t| t.source_state)
                .collect();

            for q in &reachable_states {
                let mut best_probabilities_max = LogDomain::zero();
                // determine the transition that yields the best probability for
                // a state (go through all transitions whose child states are
                // explored but whose source state is not)
                for t in transitions.iter().filter(|t| {
                    t.target_states
                        .iter()
                        .cloned()
                        .collect::<HashSet<usize>>()
                        .is_subset(&explored_states)
                        && t.source_state == *q
                }) {
                    // calculate the probability of applying transition t given
                    // probabilities for each child state
                    let pr = t.probability
                        * t.target_states
                            .iter()
                            .map(|q_i| best_probabilities[*q_i])
                            .product();
                    // determine the best reachable probability
                    if pr > best_probabilities_max {
                        best_probabilities_max = pr;
                        // construct the corresponding tree
                        best_trees[*q] = Some(Tree::new_with_children(
                            self.t_integeriser
                                .find_value(t.symbol)
                                .unwrap()
                                .clone(),
                            t.target_states
                                .iter()
                                .map(|q_i| best_trees[*q_i].clone().unwrap())
                                .collect(),
                        ));
                    }
                }
                best_probabilities[*q] = best_probabilities_max;
                // add only the state to the set of explored states with the
                // best probability among all unexplored states
                explored_states.insert(
                    *reachable_states
                        .iter()
                        .max_by(|&q_1, &q_2| {
                            best_probabilities[*q_1]
                                .cmp(&best_probabilities[*q_2])
                        })
                        .unwrap(),
                );
            }
        }

        // apply root weights
        best_probabilities = best_probabilities
            .iter()
            .zip(&self.root_weights)
            .map(|(q, p)| *q * *p)
            .collect::<Vec<LogDomain<f64>>>();

        // return (tree, probability)-pair with maximal probability
        best_probabilities
            .iter()
            .zip(best_trees)
            .max_by(|(&p_1, _), (p_2, _)| p_1.cmp(p_2))
            .map(|(p, t)| (t.unwrap(), *p))
            .unwrap()
    }
}

// Pretty print of PTA
impl<Q, T> Display for PTA<Q, T>
where
    Q: Eq + Hash + Clone + Display,
    T: Eq + Hash + Clone + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret: String = String::new();
        // format non-null root weights
        for (q, p) in self.root_weights.iter().enumerate() {
            if p != &LogDomain::zero() {
                writeln!(
                    &mut ret,
                    "root: {} # {}",
                    self.q_integeriser.find_value(q).unwrap(),
                    p
                )?;
            }
        }

        // format transitions (with non-null transitions)
        for s_hashmap in self.transitions.values() {
            for transitions in s_hashmap.values() {
                for t in transitions {
                    // get a pretty string representation of target states
                    let mut target_states_str = String::new();
                    for q in &t.target_states {
                        target_states_str.push_str(
                            &self
                                .q_integeriser
                                .find_value(*q)
                                .unwrap()
                                .to_string(),
                        );
                        target_states_str.push_str(", ");
                    }
                    target_states_str.pop();
                    target_states_str.pop();

                    writeln!(
                        &mut ret,
                        "transition: {} -> {}({}) # {}",
                        self.q_integeriser.find_value(t.source_state).unwrap(),
                        self.t_integeriser.find_value(t.symbol).unwrap(),
                        target_states_str,
                        t.probability,
                    )?;
                }
            }
        }
        write!(f, "{}", ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_most_probable_tree() {
        let pta_string = "root: 0 # 0.7\n\
                          root: 1 # 0.2\n\
                          root: 2 # 0.1\n\
                          transition: 1 -> a() # 0.5\n\
                          transition: 2 -> a() # 0.4\n\
                          transition: 1 -> b() # 0.2\n\
                          transition: 2 -> b() # 0.6\n\
                          transition: 0 -> s(1, 1) # 0.9\n\
                          transition: 0 -> s(2, 2) # 0.1\n\
                          transition: 1 -> s(1, 2) # 0.3";
        let pta: PTA<usize, char> = pta_string.parse().unwrap();
        let mpt = pta.most_probable_tree();
        assert_eq!(mpt.0, "(s (a) (a))".parse().unwrap());
        assert_eq!(mpt.1, LogDomain::new(0.1807).unwrap());
    }

    #[test]
    fn test_probability() {
        let pta_string = "root: 0 # 0.7\n\
                          root: 1 # 0.2\n\
                          root: 2 # 0.1\n\
                          transition: 1 -> a() # 0.5\n\
                          transition: 2 -> a() # 0.4\n\
                          transition: 1 -> b() # 0.2\n\
                          transition: 2 -> b() # 0.6\n\
                          transition: 0 -> s(1, 1) # 0.9\n\
                          transition: 0 -> s(2, 2) # 0.1\n\
                          transition: 1 -> s(1, 2) # 0.3";
        let pta: PTA<usize, char> = pta_string.parse().unwrap();
        let mut xi: Tree<char> = "(s (a) (b))".parse().unwrap();
        xi.is_prefix = true;
        assert_eq!(
            pta.probability(&mut xi, &mut HashSet::new()),
            LogDomain::new(0.0978).unwrap()
        );
    }

    // TODO find example where height bound matters
    #[test]
    fn test_potential_probability() {
        let pta_string = "root: 0 # 0.7\n\
                          root: 1 # 0.2\n\
                          root: 2 # 0.1\n\
                          transition: 1 -> a() # 0.5\n\
                          transition: 2 -> a() # 0.4\n\
                          transition: 1 -> b() # 0.2\n\
                          transition: 2 -> b() # 0.6\n\
                          transition: 0 -> s(1, 1) # 0.9\n\
                          transition: 0 -> s(2, 2) # 0.1\n\
                          transition: 1 -> s(1, 2) # 0.3";
        let pta: PTA<usize, char> = pta_string.parse().unwrap();
        let mut xi: Tree<char> = "(s (a) (s))".parse().unwrap();
        xi.is_prefix = true;
        assert_eq!(
            pta._potential_probability(&mut xi, &mut HashSet::new(),),
            LogDomain::new(0.0945).unwrap()
        );
    }

    #[test]
    fn test_best_parse() {
        let pta_string = "root: 0 # 0.7\n\
                          root: 1 # 0.2\n\
                          root: 2 # 0.1\n\
                          transition: 1 -> a() # 0.5\n\
                          transition: 2 -> a() # 0.4\n\
                          transition: 1 -> b() # 0.2\n\
                          transition: 2 -> b() # 0.6\n\
                          transition: 0 -> s(1, 1) # 0.9\n\
                          transition: 0 -> s(2, 2) # 0.1\n\
                          transition: 1 -> s(1, 2) # 0.3";
        let pta: PTA<usize, char> = pta_string.parse().unwrap();
        let best_parse = pta.best_parse();
        assert_eq!(best_parse.0, "(s (a) (a))".parse().unwrap());
        assert_eq!(best_parse.1, LogDomain::new(0.1575).unwrap());
    }
}
