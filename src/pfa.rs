use log_domain::LogDomain;
use nalgebra::{DMatrix, DVector, RowDVector};
use num_traits::Zero;
use std::cmp;
use std::collections::{BinaryHeap, HashMap};

pub struct PFA {
    pub sigma: Vec<char>,
    pub number_states: usize,
    pub initial_states: RowDVector<f64>,
    pub final_states: DVector<f64>,
    pub transitions: HashMap<char, DMatrix<f64>>,
    pub m_sigma_star: DMatrix<f64>,
}

impl PFA {
    pub fn new(
        initial_states: RowDVector<f64>,
        final_states: DVector<f64>,
        transitions: HashMap<char, DMatrix<f64>>,
    ) -> PFA {
        let number_states = initial_states.len();
        let m_sigma_star = (DMatrix::identity(number_states, number_states)
            - transitions.values().sum::<DMatrix<f64>>())
        .try_inverse()
        .unwrap();
        PFA {
            sigma: transitions.keys().map(|sigma| sigma.clone()).collect(),
            number_states: initial_states.len(),
            initial_states: initial_states,
            final_states: final_states,
            transitions: transitions,
            m_sigma_star: m_sigma_star,
        }
    }

    pub fn probability(&self, w: &Vec<char>) -> LogDomain<f64> {
        let mut prefix = &self.initial_states
            * DMatrix::identity(self.number_states, self.number_states);
        for c in w {
            prefix = prefix * self.transitions.get(&c).unwrap();
        }
        let result = prefix * &self.final_states;

        LogDomain::new(*result.get(0).unwrap()).unwrap()
    }

    pub fn prefix_probability(&self, w: &Vec<char>) -> LogDomain<f64> {
        // TODO save prefix as part of queue item
        let mut prefix = &self.initial_states
            * DMatrix::identity(self.number_states, self.number_states);
        for c in w {
            prefix = prefix * self.transitions.get(&c).unwrap();
        }
        let result = prefix * &self.m_sigma_star * &self.final_states;

        LogDomain::new(*result.get(0).unwrap()).unwrap()
    }

    pub fn potential_probability(&self, w: &Vec<char>) -> LogDomain<f64> {
        cmp::min(
            self.prefix_probability(w),
            LogDomain::new(self.number_states.pow(2) as f64).unwrap(),
        )
    }

    pub fn most_probable_string(&self) -> Vec<char> {
        let mut current_prop = LogDomain::zero();
        let mut current_best = vec![];

        let mut q = BinaryHeap::new();
        q.push((self.potential_probability(&vec![]), vec![]));

        while !q.is_empty() {
            print!("\n[");
            for (pp, w) in &q {
                print!("({}, {}), ", w.iter().collect::<String>(), pp);
            }
            println!("]");
            let (pp, w) = q.pop().unwrap();
            println!("PP({}) \t {}", w.iter().collect::<String>(), pp);

            if pp > current_prop {
                let p = self.probability(&w);
                println!("P({}) \t {}", w.iter().collect::<String>(), p);
                if p > current_prop {
                    current_prop = p;
                    current_best = w.clone();
                }

                for a in &self.sigma {
                    let mut wa = w.clone();
                    wa.push(*a);
                    let pp_wa = self.potential_probability(&wa);
                    println!(
                        "PP({}) \t {} \t {}",
                        wa.iter().collect::<String>(),
                        pp_wa,
                        self.number_states.pow(2) as f64 / wa.len() as f64
                    );
                    if pp_wa > current_prop {
                        q.push((pp_wa, wa));
                    }
                }
            } else {
                return current_best;
            }
        }
        return current_best;
    }
}
