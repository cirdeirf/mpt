use log_domain::LogDomain;
use num_traits::Zero;
use std::collections::HashMap;

// type TransitionMap =
//     HashMap<char, Vec<(usize, char, Vec<usize>, LogDomain<f64>)>>;

type TransitionMap = HashMap<
    char,
    HashMap<usize, Vec<(usize, char, Vec<usize>, LogDomain<f64>)>>,
>;

#[derive(Debug)]
pub struct Tree {
    pub root: char,
    pub children: Vec<Tree>,
}

// #[derive(Debug)]
// pub struct Transition {
//     pub sigma: char,
//     pub target_state: usize,
//     pub source_states: Vec<usize>,
//     pub weight: LogDomain<f64>,
// }

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
        // let mut a: Vec<(usize, char, Vec<usize>, LogDomain<f64>)> = Vec::new();
        // for trs in transitions.values() {
        //     a.append(&mut trs.clone());
        // }
        // let mut qtransitions = HashMap::new();
        // for i in 0..(root_weights.len()) {
        //     let mut b = Vec::new();
        //     for t in &a {
        //         if t.0 == i {
        //             b.push(t.clone());
        //         }
        //     }
        //     qtransitions.insert(i, b);
        // }
        PTA {
            // sigma: transitions.keys().map(|sigma| sigma.clone()).collect(),
            sigma: sigma,
            number_states: root_weights.len(),
            root_weights: root_weights,
            transitions: transitions,
        }
    }

    pub fn probability(&self, xi: &Tree) -> Vec<LogDomain<f64>> {
        // println!("{}", xi.root);
        // println!("{:#?}", xi.children);
        let transitions = self.transitions.get(&xi.root).unwrap();

        let mut ret: Vec<LogDomain<f64>> = Vec::new();
        for q in 0..self.number_states {
            println!("{}: {}", xi.root, q);
            let mut p_q = LogDomain::zero();
            if let Some(tr_q) = transitions.get(&q) {
                // let tr_q = transitions.get(&q).unwrap();
                for t in tr_q {
                    let mut p_t = t.3;
                    for (i, q_i) in t.2.iter().enumerate() {
                        match xi.children.get(i) {
                            Some(t_i) => p_t *= self.probability(t_i)[*q_i],
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
        for p in &ret {
            print!("{}\t", p);
        }
        println!("");
        ret
    }

    pub fn prefix_probability(&self, xi: &Tree) -> Vec<LogDomain<f64>> {
        // println!("bar: {}", xi.root);
        let transitions = self.transitions.get(&xi.root).unwrap();

        let mut ret: Vec<LogDomain<f64>> = Vec::new();
        for q in 0..self.number_states {
            println!("{}: {}", xi.root, q);
            let mut p_q = LogDomain::new(0.0).unwrap();
            if let Some(tr_q) = transitions.get(&q) {
                // let tr_q = transitions.get(&q).unwrap();
                for t in tr_q {
                    let mut p_t = t.3;
                    for (i, q_i) in t.2.iter().enumerate() {
                        if let Some(t_i) = &xi.children.get(i) {
                            p_t *=
                                self.prefix_probability(&xi.children[i])[*q_i];
                        }
                    }
                    p_q += p_t;
                }
            }
            ret.push(p_q);
        }
        for p in &ret {
            print!("{}\t", p);
        }
        println!("");
        ret

        // if *self.sigma.get(&xi.root).unwrap() == 0 {
        //     return vec![
        //         LogDomain::new(0.0).unwrap(),
        //         LogDomain::new(0.3).unwrap(),
        //         LogDomain::new(0.4).unwrap(),
        //     ];
        // }

        // let transitions: HashMap<
        //     usize,
        //     Vec<(usize, char, Vec<usize>, LogDomain<f64>)>,
        // > = self.qtransitions.iter().fil
        // for q in 0..(self.number_states - 1) {}

        // for (q, k) in run.iter().enumerate() {
        //     let transitions = self
        //         .transitions
        //         .get(&xi.root)
        //         .unwrap()
        //         .iter()
        //         .filter(|t| t.0 == q)
        //         .collect::<Vec<&(usize, char, Vec<usize>, LogDomain<f64>)>>();
        //     for t in transitions {
        //         // k *
        //     }
        // }

        // println!("{:#?}", xi);
        // let mut p = 0.0;
        // // for (q, &qp) in self.root_weights.iter().enumerate() {
        // for trs in self.transitions.get(&xi.root) {
        //     for t in trs {
        //         p += t.3.value() * self.root_weights[t.0].value();
        //     }
        // }
        // println!("{}", p);
        // vec![LogDomain::new(0.5).unwrap()]
    }

    fn prefix_probability_rec(&self, xi: &Tree) -> LogDomain<f64> {
        LogDomain::new(0.2).unwrap()
    }

    // pub fn most_probable_tree(&self) -> Tree {
    //     let mut current_prop = LogDomain::zero();
    //     let mut current_best =
    // }
}
