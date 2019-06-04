mod pfa;
mod pta;

use log_domain::LogDomain;
use nalgebra::{DMatrix, DVector, RowDVector};
use pfa::PFA;
use pta::Tree;
use pta::PTA;

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

fn main() {
    let pfa = PFA::new(
        RowDVector::from_vec(vec![1., 0., 0.]),
        DVector::from_vec(vec![0., 0.0, 1.0]),
        hashmap![
            'a' =>
            DMatrix::from_vec(3, 3, vec![0., 0., 0., 0.3, 0., 0., 0., 0.2, 0.]),
            'b' =>
            DMatrix::from_vec(3, 3, vec![0., 0., 0., 0.7, 0., 0., 0., 0.8, 0.])],
    );
    // println!("m_a: {}", pfa.transitions.get(&'a').unwrap());
    // println!("m_b: {}", pfa.transitions.get(&'b').unwrap());
    // println!("alg: {:?}", pfa.most_probable_string());

    let pta = PTA::new(
        hashmap!['a' => 0, 'b' => 0, 's' => 2],
        vec![
            LogDomain::new(1.0).unwrap(),
            LogDomain::new(0.0).unwrap(),
            LogDomain::new(0.0).unwrap(),
        ],
        hashmap![
        'a' => hashmap![1 => vec![(1, 'a', vec![], LogDomain::new(0.5).unwrap())],
                        2 => vec![(1, 'a', vec![], LogDomain::new(0.4).unwrap())]],
        'b' => hashmap![1 => vec![(1, 'b', vec![], LogDomain::new(0.2).unwrap())],
                        2 => vec![(1, 'b', vec![], LogDomain::new(0.6).unwrap())]],
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
    pta.prefix_probability(&xi);
    println!("");
    pta.probability(&xi);
}
