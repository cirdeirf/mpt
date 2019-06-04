use log_domain::LogDomain;
use num_traits::Zero;
use std::collections::HashMap;

type TransitionMap = HashMap<
    char,
    HashMap<usize, Vec<(usize, char, Vec<usize>, LogDomain<f64>)>>,
>;

#[derive(Debug)]
pub struct Tree {
    pub root: char,
    pub children: Vec<Tree>,
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

    pub fn probability(&self, xi: &Tree) -> LogDomain<f64> {
        self.rec_probability_root(xi, false)
    }

    pub fn prefix_probability(&self, xi: &Tree) -> LogDomain<f64> {
        self.rec_probability_root(xi, true)
    }

    fn rec_probability_root(&self, xi: &Tree, prefix: bool) -> LogDomain<f64> {
        self.rec_probability(xi, prefix)
            .iter()
            .zip(&self.root_weights)
            .map(|(&p_q, &root_q)| p_q * root_q)
            .fold(LogDomain::zero(), |acc, x| acc + x)
    }

    fn rec_probability(&self, xi: &Tree, prefix: bool) -> Vec<LogDomain<f64>> {
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
                                p_t *= self.rec_probability(t_i, prefix)[*q_i]
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
        ret
    }

    // pub fn most_probable_tree(&self) -> Tree {
    //     let mut current_prop = LogDomain::zero();
    //     let mut current_best =
    // }
}
