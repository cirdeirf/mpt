use log_domain::LogDomain;
use nalgebra::{DMatrix, DVector, RowDVector};
use std::collections::{HashMap, HashSet};

type TransitionMap =
    HashMap<char, Vec<(usize, char, Vec<usize>, LogDomain<f64>)>>;

// #[derive(Debug)]
// pub struct Transition {
//     pub sigma: char,
//     pub target_state: usize,
//     pub source_states: Vec<usize>,
//     pub weight: LogDomain<f64>,
// }

pub struct PTA {
    pub sigma: HashSet<char>,
    pub number_states: usize,
    pub root_weights: Vec<LogDomain<f64>>,
    pub transitions: TransitionMap,
}

impl PTA {
    pub fn new(
        root_weights: Vec<LogDomain<f64>>,
        transitions: TransitionMap,
    ) -> PTA {
        PTA {
            sigma: transitions.keys().map(|sigma| sigma.clone()).collect(),
            number_states: root_weights.len(),
            root_weights: root_weights,
            transitions: transitions,
        }
    }

    pub fn inside_weight(&self, state: &usize) -> LogDomain<f64> {
        LogDomain::new(0.2).unwrap()
    }

    pub fn matrix(&self) -> f64 {
        // let m_e = RowDVector::from_vec(vec![0., 0.1, 1., 1.]);
        // let f = DVector::from_vec(vec![1., 0., 0., 0.]);
        // let m_a = DMatrix::from_vec(
        //     4,
        //     4,
        //     vec![
        //         0., 0.2, 0., 0., 0., 0., 0.3, 0., 0., 0., 0., 0., 0., 0., 0.,
        //         0.,
        //     ],
        // );
        // let m_b = DMatrix::from_vec(
        //     4,
        //     4,
        //     vec![
        //         0., 0.8, 0., 0., 0., 0., 0.5, 0., 0., 0., 0., 0., 0., 0., 0.,
        //         0.,
        //     ],
        // );
        // let m_s = DMatrix::from_vec(
        //     4,
        //     4,
        //     vec![
        //         0., 0., 0., 0., 0., 0., 0., 0.1, 0., 0., 0., 0., 0., 0., 0., 0.,
        //     ],
        // );
        // let m_sigma_star = (DMatrix::identity(4, 4) - (&m_a + &m_b + &m_s))
        //     .try_inverse()
        //     .unwrap();

        let m_e = RowDVector::from_vec(vec![0., 0.8, 0.8]);
        let m_h = RowDVector::from_vec(vec![0., 0.2, 0.2]);
        let f = DVector::from_vec(vec![1., 0., 0.]);
        let m_a =
            DMatrix::from_vec(3, 3, vec![0., 0.6, 0., 0., 0., 0., 0., 0., 0.]);
        let m_s =
            DMatrix::from_vec(3, 3, vec![0., 0., 0.4, 0., 0., 0., 0., 0., 0.]);
        let m_sigma_star = (DMatrix::identity(3, 3) - (&m_a + &m_s))
            .try_inverse()
            .unwrap();

        println!("\nm_e: {}", &m_e);
        println!("m_a: {}", &m_a);
        // println!("m_b: {}", &m_b);
        println!("m_s: {}", &m_s);
        println!("{}", &m_e * &m_sigma_star * &m_s * &f);
        // println!("{}", &m_e * &m_sigma_star * &m_s * &m_b * &f);
        2.
    }

    pub fn fixpoint(&self) -> Vec<f64> {
        let mut v = vec![0., 0.];
        // let mut v = vec![0., 0.];
        println!("\n{:?}", v);
        for _ in 0..200 {
            // let w = vec![
            //     0.2 * v[0] + 0.8,
            //     0.3 * v[1] + 0.1 * v[0] + 0.6,
            //     0.9 * v[1] * v[1] + 0.1 * v[1] * v[2],
            // ];
            // let w = vec![0.3 * v[1], 0.4 * v[1] + 0.6];
            let w = vec![0.6 * v[1] + 0.4 * v[1] * v[1], 0.8 + 0.2];
            v = w;
            println!("{:?}", v);
        }
        v
    }
}
