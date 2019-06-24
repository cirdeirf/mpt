mod from_str;
mod transition;
pub mod tree;

use integeriser::{HashIntegeriser, Integeriser};
use log_domain::LogDomain;
use num_traits::Zero;
use priority_queue::PriorityQueue;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Debug, Display, Write};
use std::hash::Hash;
use transition::{Integerisable2, Transition};
use tree::Tree;

pub struct PTA<Q, T>
where
    Q: Eq + Hash,
    T: Eq + Hash,
{
    q_integeriser: HashIntegeriser<Q>,
    t_integeriser: HashIntegeriser<T>,
    sigma: HashMap<T, usize>,
    number_states: usize,
    root_weights: Vec<LogDomain<f64>>,
    transitions: HashMap<usize, HashMap<usize, Vec<Transition<usize, usize>>>>,
}

impl<Q, T> PTA<Q, T>
where
    Q: Eq + Hash + Clone + Debug,
    T: Eq + Hash + Clone + Debug + Display,
{
    fn new(
        root_weight_map: HashMap<Q, LogDomain<f64>>,
        transitions_vec: Vec<Transition<Q, T>>,
    ) -> PTA<Q, T> {
        let mut q_integeriser = HashIntegeriser::new();
        let mut t_integeriser = HashIntegeriser::new();

        let mut sigma = HashMap::new();
        for t in &transitions_vec {
            sigma
                .entry(t.symbol.clone())
                .or_insert_with(|| t.target_states.len());
        }

        let mut transitions: HashMap<
            usize,
            HashMap<usize, Vec<Transition<usize, usize>>>,
        > = HashMap::new();
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

        let mut root_weights: Vec<LogDomain<f64>> = Vec::new();
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

    fn probability_rec(
        &self,
        xi: &mut Tree<T>,
        mut known_trees: &mut HashSet<Tree<T>>,
    ) -> Vec<LogDomain<f64>> {
        if known_trees.contains(&xi) {
            known_trees.get(&xi).unwrap().run.clone()
        } else {
            let transitions = self
                .transitions
                .get(&self.t_integeriser.find_key(&xi.root).unwrap())
                .unwrap();

            let mut ret: Vec<LogDomain<f64>> = Vec::new();
            for q in 0..self.number_states {
                let mut p_q = LogDomain::zero();

                if let Some(v) = transitions.get(&q) {
                    for t in v {
                        let mut p_t = t.probability;
                        for (i, q_i) in t.target_states.iter().enumerate() {
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
            xi.run = ret.clone();
            known_trees.insert(xi.clone());
            ret
        }
    }

    fn potential_probability(
        &self,
        xi: &mut Tree<T>,
        mut known_trees: &mut HashSet<Tree<T>>,
    ) -> LogDomain<f64> {
        cmp::min(
            self.probability(xi, &mut known_trees),
            LogDomain::new(
                self.number_states.pow(2) as f64 / xi.get_height() as f64,
            )
            .unwrap(),
        )
    }

    fn probability(
        &self,
        xi: &mut Tree<T>,
        mut known_trees: &mut HashSet<Tree<T>>,
    ) -> LogDomain<f64> {
        let pr = self
            .probability_rec(xi, &mut known_trees)
            .iter()
            .zip(&self.root_weights)
            .map(|(&p_q, &root_q)| p_q * root_q)
            .sum();
        xi.probability = pr;
        pr
    }

    pub fn most_probable_tree(&self) -> (Tree<T>, LogDomain<f64>) {
        let mut current_prop = LogDomain::zero();
        let mut current_best;
        let mut known_trees = HashSet::new();

        let mut q = PriorityQueue::new();
        for (s, rank) in &self.sigma {
            let mut t = Tree::new(s.clone());
            // TODO comment
            //
            t.is_prefix = Some(rank != &0);
            let pp = self.potential_probability(&mut t, &mut known_trees);
            q.push(t, pp);
        }

        current_best = q.peek().unwrap().0.clone();

        while !q.is_empty() {
            let (t, pp) = q.pop().unwrap();

            // ξ ∈ T_Σ
            if !t.is_prefix.unwrap() {
                current_best = t;
                current_prop = pp;
                break;
            }
            // ξ ∉ T_Σ (contains variables, i.e., is a prefix-tree/context)
            else {
                for s in self.sigma.keys() {
                    let mut t_s = t.clone();

                    t_s.is_prefix = Some(t_s.extend(s, &self.sigma));
                    let pp_t_s =
                        self.potential_probability(&mut t_s, &mut known_trees);

                    // do not add (prefix-)trees to the queue that are worse
                    // than the current best complete tree
                    if pp_t_s > current_prop {
                        // ξ ∈ T_Σ (t_s complete + better than the current best)
                        if !t_s.is_prefix.unwrap() {
                            current_best = t_s.clone();
                            current_prop = pp_t_s;
                        }
                        q.push(t_s, pp_t_s);
                    }
                }
            }
        }
        (current_best, current_prop)
    }

    /// Dertermines the best/most probable parse.
    /// Return the corrresponding tree and the run's probability.
    /// This implementation is based on the BestParse algorithm depicted in
    /// Figure 3 of "Parsing Algorithms based on Tree Automata" by Maletti and
    /// Satta, 2009 [MS09, Figure 3] [MS09, Figure 3]
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

        // get all root states (with non-null root weight)
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
                            .copied()
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
                        .copied()
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

impl<Q, T> Display for PTA<Q, T>
where
    Q: Eq + Hash + Clone + Display + Debug,
    T: Eq + Hash + Clone + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret: String = String::new();
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

        for s_hashmap in self.transitions.values() {
            for transitions in s_hashmap.values() {
                for t in transitions {
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
        xi.is_prefix = Some(true);
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
        xi.is_prefix = Some(true);
        assert_eq!(
            pta.potential_probability(&mut xi, &mut HashSet::new(),),
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
