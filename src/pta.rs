use log_domain::LogDomain;
use num_traits::Zero;
use priority_queue::PriorityQueue;
use std::cmp;
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

type TransitionMap = HashMap<
    char,
    HashMap<usize, Vec<(usize, char, Vec<usize>, LogDomain<f64>)>>,
>;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Tree {
    pub root: char,
    pub children: Vec<Tree>,
}

impl Tree {
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

    pub fn extend(&mut self, s: char, sigma: &HashMap<char, usize>) {
        let mut t_stack = Vec::new();
        t_stack.push(self);
        while !t_stack.is_empty() {
            let t = t_stack.pop().unwrap();
            if t.children.len() < sigma[&t.root] {
                t.children.push(Tree {
                    root: s,
                    children: Vec::new(),
                });
                break;
            } else {
                for t_i in &mut t.children {
                    t_stack.push(t_i);
                }
            }
        }
    }

    fn to_string(&self) -> String {
        let mut ret = self.root.to_string();
        if !self.children.is_empty() {
            ret.push_str("( ");
            for t_i in &self.children {
                ret.push_str(&t_i.to_string());
                ret.push_str(", ");
            }
            ret.pop();
            ret.pop();
            ret.push_str(" )");
        }
        ret
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub struct PTA {
    pub sigma: HashMap<char, usize>,
    pub number_states: usize,
    pub root_weights: Vec<LogDomain<f64>>,
    pub transitions: TransitionMap,
}

impl PTA {
    pub fn new(
        sigma: HashMap<char, usize>,
        root_weights: Vec<LogDomain<f64>>,
        transitions: TransitionMap,
    ) -> PTA {
        PTA {
            sigma: sigma,
            number_states: root_weights.len(),
            root_weights: root_weights,
            transitions: transitions,
        }
    }

    pub fn probability(
        &self,
        xi: &Tree,
        mut a: &mut HashMap<Tree, Vec<LogDomain<f64>>>,
        mut b: &mut HashMap<Tree, Vec<LogDomain<f64>>>,
    ) -> LogDomain<f64> {
        self.rec_probability_root(xi, false, &mut a, &mut b)
    }

    pub fn prefix_probability(
        &self,
        xi: &Tree,
        mut a: &mut HashMap<Tree, Vec<LogDomain<f64>>>,
        mut b: &mut HashMap<Tree, Vec<LogDomain<f64>>>,
    ) -> LogDomain<f64> {
        self.rec_probability_root(xi, true, &mut a, &mut b)
    }

    fn potential_probability(
        &self,
        xi: &Tree,
        mut a: &mut HashMap<Tree, Vec<LogDomain<f64>>>,
        mut b: &mut HashMap<Tree, Vec<LogDomain<f64>>>,
    ) -> LogDomain<f64> {
        cmp::min(
            self.prefix_probability(xi, &mut a, &mut b),
            LogDomain::new(
                self.number_states.pow(2) as f64 / xi.get_height() as f64,
            )
            .unwrap(),
        )
    }

    fn rec_probability_root(
        &self,
        xi: &Tree,
        prefix: bool,
        mut a: &mut HashMap<Tree, Vec<LogDomain<f64>>>,
        mut b: &mut HashMap<Tree, Vec<LogDomain<f64>>>,
    ) -> LogDomain<f64> {
        self.rec_probability(xi, prefix, &mut a, &mut b)
            .iter()
            .zip(&self.root_weights)
            .map(|(&p_q, &root_q)| p_q * root_q)
            .sum()
    }

    fn rec_probability(
        &self,
        xi: &Tree,
        prefix: bool,
        mut a: &mut HashMap<Tree, Vec<LogDomain<f64>>>,
        mut b: &mut HashMap<Tree, Vec<LogDomain<f64>>>,
    ) -> Vec<LogDomain<f64>> {
        if a.contains_key(&xi) && prefix {
            return a.get(xi).unwrap().clone();
        }
        if b.contains_key(&xi) && !prefix {
            return b.get(xi).unwrap().clone();
        }
        // println!("{}", xi);
        let transitions = self.transitions.get(&xi.root).unwrap();

        let mut ret: Vec<LogDomain<f64>> = Vec::new();
        for q in 0..self.number_states {
            let mut p_q = LogDomain::zero();
            if let Some(tr_q) = transitions.get(&q) {
                for t in tr_q {
                    let mut p_t = t.3;
                    for (i, q_i) in t.2.iter().enumerate() {
                        match xi.children.get(i) {
                            Some(t_i) => {
                                p_t *= self.rec_probability(
                                    t_i, prefix, &mut a, &mut b,
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
            }
            ret.push(p_q);
        }
        // for q_p in &ret {
        //     print!("{}\t", q_p);
        // }
        // println!("");
        if !a.contains_key(&xi) && prefix {
            &mut a.insert(xi.clone(), ret.clone());
        }
        if !b.contains_key(&xi) && !prefix {
            &mut b.insert(xi.clone(), ret.clone());
        }
        ret
    }

    pub fn most_probable_tree(&self) -> (Tree, LogDomain<f64>) {
        let start = Instant::now();
        let mut current_prop = LogDomain::zero();
        let mut current_best;
        let mut a: HashMap<Tree, Vec<LogDomain<f64>>> = HashMap::new();
        let mut b: HashMap<Tree, Vec<LogDomain<f64>>> = HashMap::new();

        let mut q = PriorityQueue::new();
        for (s, _) in &self.sigma {
            let t = Tree {
                root: s.clone(),
                children: Vec::new(),
            };
            let pp = self.potential_probability(&t, &mut a, &mut b);
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
                let p = self.probability(&t, &mut a, &mut b);
                // println!("P({}) \t {}", t, p);
                if p > current_prop {
                    current_prop = p;
                    current_best = t.clone();
                }

                // println!("\nt: {}", t);
                for (s, _) in &self.sigma {
                    let mut t_s = t.clone();
                    t_s.extend(*s, &self.sigma);
                    // println!("t_s: {}", t_s);
                    let pp_t_s =
                        self.potential_probability(&t_s, &mut a, &mut b);
                    // println!("t_s: {}\t{}", t_s, pp_t_s);
                    if pp_t_s > current_prop {
                        q.push(t_s, pp_t_s);
                    }
                }
            } else {
                println!("time: {:?}", start.elapsed());
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
        let pta = PTA::new(
            hashmap!['a' => 0, 'b' => 0, 's' => 2],
            vec![
                LogDomain::new(0.7).unwrap(),
                LogDomain::new(0.2).unwrap(),
                LogDomain::new(0.1).unwrap(),
            ],
            hashmap![
        'a' => hashmap![1 => vec![(1, 'a', vec![], LogDomain::new(0.5).unwrap())],
                        2 => vec![(2, 'a', vec![], LogDomain::new(0.4).unwrap())]],
        'b' => hashmap![1 => vec![(1, 'b', vec![], LogDomain::new(0.2).unwrap())],
                        2 => vec![(2, 'b', vec![], LogDomain::new(0.6).unwrap())]],
        's' => hashmap![0 => vec![(0, 's', vec![1, 1], LogDomain::new(0.9).unwrap()),
                                  (0, 's', vec![2, 2], LogDomain::new(0.1).unwrap())],
                        1 => vec![(1, 's', vec![1, 2], LogDomain::new(0.3).unwrap())]]],
        );
        let mpt = pta.most_probable_tree();
        assert_eq!(
            mpt.0,
            Tree {
                root: 's',
                children: vec![
                    Tree {
                        root: 'a',
                        children: vec![]
                    },
                    Tree {
                        root: 'a',
                        children: vec![]
                    }
                ]
            }
        );
        assert_eq!(mpt.1, LogDomain::new(0.1807).unwrap());
    }

    #[test]
    fn test_probability() {
        let pta = PTA::new(
            hashmap!['a' => 0, 'b' => 0, 's' => 2],
            vec![
                LogDomain::new(0.7).unwrap(),
                LogDomain::new(0.2).unwrap(),
                LogDomain::new(0.1).unwrap(),
            ],
            hashmap![
        'a' => hashmap![1 => vec![(1, 'a', vec![], LogDomain::new(0.5).unwrap())],
                        2 => vec![(2, 'a', vec![], LogDomain::new(0.4).unwrap())]],
        'b' => hashmap![1 => vec![(1, 'b', vec![], LogDomain::new(0.2).unwrap())],
                        2 => vec![(2, 'b', vec![], LogDomain::new(0.6).unwrap())]],
        's' => hashmap![0 => vec![(0, 's', vec![1, 1], LogDomain::new(0.9).unwrap()),
                                  (0, 's', vec![2, 2], LogDomain::new(0.1).unwrap())],
                        1 => vec![(1, 's', vec![1, 2], LogDomain::new(0.3).unwrap())]]],
        );
        let xi = Tree {
            root: 's',
            children: vec![
                Tree {
                    root: 'a',
                    children: vec![],
                },
                Tree {
                    root: 'b',
                    children: vec![],
                },
            ],
        };
        assert_eq!(pta.probability(&xi), LogDomain::new(0.0978).unwrap());
    }

    #[test]
    fn test_potential_probability() {
        let pta = PTA::new(
            hashmap!['a' => 0, 'b' => 0, 's' => 2],
            vec![
                LogDomain::new(0.7).unwrap(),
                LogDomain::new(0.2).unwrap(),
                LogDomain::new(0.1).unwrap(),
            ],
            hashmap![
        'a' => hashmap![1 => vec![(1, 'a', vec![], LogDomain::new(0.5).unwrap())],
                        2 => vec![(2, 'a', vec![], LogDomain::new(0.4).unwrap())]],
        'b' => hashmap![1 => vec![(1, 'b', vec![], LogDomain::new(0.2).unwrap())],
                        2 => vec![(2, 'b', vec![], LogDomain::new(0.6).unwrap())]],
        's' => hashmap![0 => vec![(0, 's', vec![1, 1], LogDomain::new(0.9).unwrap()),
                                  (0, 's', vec![2, 2], LogDomain::new(0.1).unwrap())],
                        1 => vec![(1, 's', vec![1, 2], LogDomain::new(0.3).unwrap())]]],
        );
        let xi = Tree {
            root: 's',
            children: vec![
                Tree {
                    root: 'a',
                    children: vec![],
                },
                Tree {
                    root: 's',
                    children: vec![],
                },
            ],
        };
        assert_eq!(
            pta.potential_probability(&xi),
            LogDomain::new(0.0945).unwrap()
        );
    }
}
