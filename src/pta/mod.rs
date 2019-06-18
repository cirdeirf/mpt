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
        root_weights: HashMap<Q, LogDomain<f64>>,
        transitions: Vec<Transition<Q, T>>,
    ) -> PTA<Q, T> {
        let mut q_inter = HashIntegeriser::new();
        let mut t_inter = HashIntegeriser::new();

        let mut sigma = HashMap::new();
        let mut root_pr: Vec<LogDomain<f64>> = Vec::new();
        let mut transition_map: HashMap<
            usize,
            HashMap<usize, Vec<Transition<usize, usize>>>,
        > = HashMap::new();

        for t in &transitions {
            sigma
                .entry(t.symbol.clone())
                .or_insert_with(|| t.target_states.len());
        }

        for t in transitions
            .into_iter()
            .map(|t| t.integerise(&mut q_inter, &mut t_inter))
        {
            transition_map
                .entry(t.symbol)
                .or_insert_with(HashMap::new)
                .entry(t.source_state)
                .or_insert_with(Vec::new)
                .push(t);
        }

        for q in q_inter.values() {
            match root_weights.get(q) {
                Some(pr_q) => root_pr.push(*pr_q),
                None => root_pr.push(LogDomain::zero()),
            }
        }

        PTA {
            q_integeriser: q_inter,
            t_integeriser: t_inter,
            sigma: sigma,
            number_states: root_pr.len(),
            root_weights: root_pr,
            transitions: transition_map,
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
            t.is_prefix = Some(rank != &0);
            let pp = self.potential_probability(&mut t, &mut known_trees);
            q.push(t, pp);
        }

        current_best = q.peek().unwrap().0.clone();

        while !q.is_empty() {
            let (mut t, pp) = q.pop().unwrap();

            if pp > current_prop {
                if !t.is_prefix.unwrap() {
                    current_prop = pp;
                    current_best = t.clone();
                } else {
                    for (s, _) in &self.sigma {
                        let mut t_s = t.clone();
                        if !t_s.extend(s.clone(), &self.sigma) {
                            t.is_prefix = Some(false);
                            q.push(t, pp);
                            break;
                        }
                        let pp_t_s = self
                            .potential_probability(&mut t_s, &mut known_trees);
                        if pp_t_s > current_prop {
                            q.push(t_s, pp_t_s);
                        }
                    }
                }
            } else {
                return (current_best, current_prop);
            }
        }
        (current_best, current_prop)
    }
}

impl<Q, T> Display for PTA<Q, T>
where
    Q: Debug + Eq + Hash + Clone,
    T: Display + Eq + Hash + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret: String = String::new();
        for (q, p) in self.root_weights.iter().enumerate() {
            if p != &LogDomain::zero() {
                write!(
                    &mut ret,
                    "root: {:?} # {}\n",
                    self.q_integeriser.find_value(q).unwrap(),
                    p
                )?;
            }
        }

        for s_hashmap in self.transitions.values() {
            for transitions in s_hashmap.values() {
                for t in transitions {
                    write!(
                        &mut ret,
                        "transition: {:?} -> {}({:?}) # {}\n",
                        self.q_integeriser.find_value(t.source_state).unwrap(),
                        self.t_integeriser.find_value(t.symbol).unwrap(),
                        t.target_states
                            .iter()
                            .map(|q| self.q_integeriser.find_value(*q).unwrap())
                            .collect::<Vec<&Q>>(),
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
}
