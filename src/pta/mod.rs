mod from_str;
pub mod tree;

use log_domain::LogDomain;
use num_traits::Zero;
use priority_queue::PriorityQueue;
use std::cmp;
use std::collections::HashMap;
use std::hash::Hash;
use tree::Tree;

pub struct TreePrMap<T> {
    prefix_probability: HashMap<Tree<T>, Vec<LogDomain<f64>>>,
    probability: HashMap<Tree<T>, Vec<LogDomain<f64>>>,
}

impl<T> TreePrMap<T>
where
    T: Eq + Hash,
{
    pub fn get_probability(
        &self,
        xi: &Tree<T>,
        prefix: bool,
    ) -> Option<&Vec<LogDomain<f64>>> {
        match prefix {
            true => self.prefix_probability.get(&xi),
            false => self.probability.get(&xi),
        }
    }

    pub fn contains_key(&self, xi: &Tree<T>, prefix: bool) -> bool {
        match prefix {
            true => self.prefix_probability.contains_key(&xi),
            false => self.probability.contains_key(&xi),
        }
    }

    pub fn insert_probability(
        &mut self,
        xi: Tree<T>,
        probability: Vec<LogDomain<f64>>,
        prefix: bool,
    ) {
        match prefix {
            true => self.prefix_probability.insert(xi, probability),
            false => self.probability.insert(xi, probability),
        };
    }
}

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
        xi: &Tree<T>,
        mut tree_pr_map: &mut TreePrMap<T>,
    ) -> LogDomain<f64> {
        self.rec_probability_root(xi, false, &mut tree_pr_map)
    }

    pub fn prefix_probability(
        &self,
        xi: &Tree<T>,
        mut tree_pr_map: &mut TreePrMap<T>,
    ) -> LogDomain<f64> {
        self.rec_probability_root(xi, true, &mut tree_pr_map)
    }

    fn potential_probability(
        &self,
        xi: &Tree<T>,
        mut tree_pr_map: &mut TreePrMap<T>,
    ) -> LogDomain<f64> {
        cmp::min(
            self.prefix_probability(xi, &mut tree_pr_map),
            LogDomain::new(
                self.number_states.pow(2) as f64 / xi.get_height() as f64,
            )
            .unwrap(),
        )
    }

    fn rec_probability_root(
        &self,
        xi: &Tree<T>,
        prefix: bool,
        mut tree_pr_map: &mut TreePrMap<T>,
    ) -> LogDomain<f64> {
        self.rec_probability(xi, prefix, &mut tree_pr_map)
            .iter()
            .zip(&self.root_weights)
            .map(|(&p_q, &root_q)| p_q * root_q)
            .sum()
    }

    fn rec_probability(
        &self,
        xi: &Tree<T>,
        prefix: bool,
        mut tree_pr_map: &mut TreePrMap<T>,
        // mut b: &mut TreeMap<T>,
    ) -> Vec<LogDomain<f64>> {
        if tree_pr_map.contains_key(xi, prefix) {
            return tree_pr_map.get_probability(xi, prefix).unwrap().clone();
        }
        // if b.contains_key(&xi) && !prefix {
        //     return b.get(xi).unwrap().clone();
        // }
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
                    match xi.children.get(i) {
                        Some(t_i) => {
                            p_t *= self
                                .rec_probability(t_i, prefix, &mut tree_pr_map) //, &mut b)
                                [*q_i]
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
        if !tree_pr_map.contains_key(&xi, prefix) {
            tree_pr_map.insert_probability(xi.clone(), ret.clone(), prefix);
        }
        // if !b.contains_key(&xi) && !prefix {
        //     &mut b.insert(xi.clone(), ret.clone());
        // }
        ret
    }

    pub fn most_probable_tree(&self) -> (Tree<T>, LogDomain<f64>) {
        let mut current_prop = LogDomain::zero();
        let mut current_best;
        let mut tree_pr_map: TreePrMap<T> = TreePrMap {
            prefix_probability: HashMap::new(),
            probability: HashMap::new(),
        };
        // let mut b: TreeMap<T> = HashMap::new();

        let mut q = PriorityQueue::new();
        for (s, _) in &self.sigma {
            let t = Tree {
                root: s.clone(),
                children: Vec::new(),
            };
            let pp = self.potential_probability(&t, &mut tree_pr_map); //, &mut b);
            q.push(t, pp);
        }

        current_best = q.peek().unwrap().0.clone();

        while !q.is_empty() {
            // println!("[");
            // for (t, pp) in &q {
            //     println!("\t({} \t {})", pp, t);
            // }
            // println!("]");

            let (t, pp) = q.pop().unwrap();
            // println!("PP({}) \t {}", t, pp);
            // return (current_best, current_prop);

            if pp > current_prop {
                let p = self.probability(&t, &mut tree_pr_map); //a, &mut b);
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
                    let pp_t_s =
                        self.potential_probability(&t_s, &mut tree_pr_map); //a, &mut b);
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
