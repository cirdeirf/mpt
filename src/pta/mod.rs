mod from_str;
pub mod tree;

use log_domain::LogDomain;
use num_traits::Zero;
use priority_queue::PriorityQueue;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use tree::Tree;

pub struct PTA<T> {
    pub sigma: HashMap<T, usize>,
    pub number_states: usize,
    pub root_weights: Vec<LogDomain<f64>>,
    pub transitions: HashMap<T, Vec<Transition<T>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Transition<T> {
    source_state: usize,
    symbol: T,
    target_states: Vec<usize>,
    probability: LogDomain<f64>,
}

impl<T> PTA<T>
where
    T: Eq + Hash + Clone,
{
    pub fn new(
        root_weights: Vec<LogDomain<f64>>,
        transitions: HashMap<T, Vec<Transition<T>>>,
    ) -> PTA<T> {
        let mut sigma = HashMap::new();
        for k in transitions.keys() {
            sigma.insert(
                k.clone(),
                transitions.get(&k).unwrap()[0].target_states.len(),
            );
        }
        PTA {
            sigma: sigma,
            number_states: root_weights.len(),
            root_weights: root_weights,
            transitions: transitions,
        }
    }

    pub fn probability(
        &self,
        xi: &mut Tree<T>,
        mut pp_set: &mut HashSet<Tree<T>>,
        mut pr_set: &mut HashSet<Tree<T>>,
    ) -> LogDomain<f64> {
        self.rec_probability_root(xi, false, &mut pp_set, &mut pr_set)
    }

    pub fn prefix_probability(
        &self,
        xi: &mut Tree<T>,
        mut pp_set: &mut HashSet<Tree<T>>,
        mut pr_set: &mut HashSet<Tree<T>>,
    ) -> LogDomain<f64> {
        self.rec_probability_root(xi, true, &mut pp_set, &mut pr_set)
    }

    fn potential_probability(
        &self,
        xi: &mut Tree<T>,
        mut pp_set: &mut HashSet<Tree<T>>,
        mut pr_set: &mut HashSet<Tree<T>>,
    ) -> LogDomain<f64> {
        cmp::min(
            self.prefix_probability(xi, &mut pp_set, &mut pr_set),
            LogDomain::new(
                self.number_states.pow(2) as f64 / xi.get_height() as f64,
            )
            .unwrap(),
        )
    }

    fn rec_probability_root(
        &self,
        xi: &mut Tree<T>,
        prefix: bool,
        mut pp_set: &mut HashSet<Tree<T>>,
        mut pr_set: &mut HashSet<Tree<T>>,
    ) -> LogDomain<f64> {
        self.rec_probability(xi, prefix, &mut pp_set, &mut pr_set)
            .iter()
            .zip(&self.root_weights)
            .map(|(&p_q, &root_q)| p_q * root_q)
            .sum()
    }

    fn rec_probability(
        &self,
        xi: &mut Tree<T>,
        prefix: bool,
        mut pp_set: &mut HashSet<Tree<T>>,
        mut pr_set: &mut HashSet<Tree<T>>,
    ) -> Vec<LogDomain<f64>> {
        if prefix && pp_set.contains(&xi) {
            return pp_set.get(&xi).unwrap().prefix_pr.clone();
        } else if !prefix && pr_set.contains(&xi) {
            return pr_set.get(&xi).unwrap().probability.clone();
        }
        let transitions = self.transitions.get(&xi.root).unwrap();

        let mut ret: Vec<LogDomain<f64>> = Vec::new();
        for q in 0..self.number_states {
            let mut p_q = LogDomain::zero();

            for t in transitions
                .iter()
                .filter(|&t| t.source_state == q)
                .collect::<Vec<&Transition<T>>>()
            {
                let mut p_t = t.probability;
                for (i, q_i) in t.target_states.iter().enumerate() {
                    match xi.children.get_mut(i) {
                        Some(t_i) => {
                            p_t *= self.rec_probability(
                                t_i,
                                prefix,
                                &mut pp_set,
                                &mut pr_set,
                            )[*q_i]
                        }
                        None if prefix => continue,
                        None => {
                            p_t = LogDomain::zero();
                            break;
                        }
                    };
                }
                p_q += p_t;
            }
            ret.push(p_q);
        }
        if prefix && !pp_set.contains(&xi) {
            xi.prefix_pr = ret.clone();
            pp_set.insert(xi.clone());
        } else if !prefix && !pr_set.contains(&xi) {
            xi.probability = ret.clone();
            pr_set.insert(xi.clone());
        }
        ret
    }

    pub fn most_probable_tree(&self) -> (Tree<T>, LogDomain<f64>) {
        let mut current_prop = LogDomain::zero();
        let mut current_best;
        let mut pp_set = HashSet::new();
        let mut pr_set = HashSet::new();

        let mut q = PriorityQueue::new();
        for (s, _) in &self.sigma {
            let mut t = Tree::new(s.clone());
            let pp =
                self.potential_probability(&mut t, &mut pp_set, &mut pr_set);
            q.push(t, pp);
        }

        current_best = q.peek().unwrap().0.clone();

        while !q.is_empty() {
            // println!("[");
            // for (t, pp) in &q {
            //     println!("\t({} \t {})", pp, t);
            // }
            // println!("]");
            let (mut t, pp) = q.pop().unwrap();
            // println!("PP({}) \t {}", t, pp);
            // return (current_best, current_prop);

            if pp > current_prop {
                let p = self.probability(&mut t, &mut pp_set, &mut pr_set);
                // println!("P({}) \t {}", t, p);
                if p > current_prop {
                    current_prop = p;
                    current_best = t.clone();
                }

                // println!("\nt: {}", t);
                for (s, _) in &self.sigma {
                    let mut t_s = t.clone();
                    t_s.extend(s.clone(), &self.sigma);
                    // println!("t_s: {}", t_s);
                    let pp_t_s = self.potential_probability(
                        &mut t_s,
                        &mut pp_set,
                        &mut pr_set,
                    );
                    // println!("t_s: {}\t{}", t_s, pp_t_s);
                    if pp_t_s > current_prop {
                        q.push(t_s, pp_t_s);
                    }
                }
            } else {
                return (current_best, current_prop);
            }
        }
        return (current_best, current_prop);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_most_probable_tree() {
        let pta_string = "root: [0.7, 0.2, 0.1]\n\
                          transition: 1 -> a() # 0.5\n\
                          transition: 2 -> a() # 0.4\n\
                          transition: 1 -> b() # 0.2\n\
                          transition: 2 -> b() # 0.6\n\
                          transition: 0 -> s(1, 1) # 0.9\n\
                          transition: 0 -> s(2, 2) # 0.1\n\
                          transition: 1 -> s(1, 2) # 0.3";
        let pta: PTA<char> = pta_string.parse().unwrap();
        let mpt = pta.most_probable_tree();
        assert_eq!(mpt.0, "(s (a) (a))".parse().unwrap());
        assert_eq!(mpt.1, LogDomain::new(0.1807).unwrap());
    }

    #[test]
    fn test_probability() {
        let pta_string = "root: [0.7, 0.2, 0.1]\n\
                          transition: 1 -> a() # 0.5\n\
                          transition: 2 -> a() # 0.4\n\
                          transition: 1 -> b() # 0.2\n\
                          transition: 2 -> b() # 0.6\n\
                          transition: 0 -> s(1, 1) # 0.9\n\
                          transition: 0 -> s(2, 2) # 0.1\n\
                          transition: 1 -> s(1, 2) # 0.3";
        let pta: PTA<char> = pta_string.parse().unwrap();
        let xi = "(s (a) (b))".parse().unwrap();
        assert_eq!(
            pta.probability(
                &xi,
                &mut TreePrMap {
                    prefix_probability: HashMap::new(),
                    probability: HashMap::new(),
                }
            ),
            LogDomain::new(0.0978).unwrap()
        );
    }

    #[test]
    fn test_potential_probability() {
        let pta_string = "root: [0.7, 0.2, 0.1]\n\
                          transition: 1 -> a() # 0.5\n\
                          transition: 2 -> a() # 0.4\n\
                          transition: 1 -> b() # 0.2\n\
                          transition: 2 -> b() # 0.6\n\
                          transition: 0 -> s(1, 1) # 0.9\n\
                          transition: 0 -> s(2, 2) # 0.1\n\
                          transition: 1 -> s(1, 2) # 0.3";
        let pta: PTA<char> = pta_string.parse().unwrap();
        let xi = "(s (a) (s))".parse().unwrap();
        assert_eq!(
            pta.potential_probability(
                &xi,
                &mut TreePrMap {
                    prefix_probability: HashMap::new(),
                    probability: HashMap::new(),
                }
            ),
            LogDomain::new(0.0945).unwrap()
        );
    }
}
