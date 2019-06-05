#[macro_use]
mod pta;

use log_domain::LogDomain;
use pta::PTA;

fn main() {
    // prime automaton for Psi = {2, 3}
    let pta = PTA::new(
        hashmap!['a' => 0, 'g' => 1],
        vec![
            LogDomain::new(1.0).unwrap(),
            LogDomain::new(0.0).unwrap(),
            LogDomain::new(0.0).unwrap(),
            LogDomain::new(0.0).unwrap(),
            LogDomain::new(0.0).unwrap(),
            LogDomain::new(0.0).unwrap(),
        ],
        hashmap![
        'a' => hashmap![2 => vec![(2, 'a', vec![], LogDomain::new(0.109).unwrap())],
                        5 => vec![(5, 'a', vec![], LogDomain::new(0.109).unwrap())]],
        'g' => hashmap![0 => vec![(0, 'g', vec![1], LogDomain::new(1.0/2.0).unwrap()),
                                  (0, 'g', vec![3], LogDomain::new(1.0/2.0).unwrap())],
                        1 => vec![(1, 'g', vec![2], LogDomain::new(1.0).unwrap())],
                        2 => vec![(2, 'g', vec![1], LogDomain::new(1.0-0.109).unwrap())],
                        3 => vec![(3, 'g', vec![4], LogDomain::new(1.0).unwrap())],
                        4 => vec![(4, 'g', vec![5], LogDomain::new(1.0).unwrap())],
                        5 => vec![(5, 'g', vec![3], LogDomain::new(1.0-0.109).unwrap())]]],
    );
    // let pta = PTA::new(
    //     hashmap!['a' => 0, 'b' => 0, 's' => 2],
    //     vec![
    //         LogDomain::new(0.7).unwrap(),
    //         LogDomain::new(0.2).unwrap(),
    //         LogDomain::new(0.1).unwrap(),
    //     ],
    //     hashmap![
    //     'a' => hashmap![1 => vec![(1, 'a', vec![], LogDomain::new(0.5).unwrap())],
    //                     2 => vec![(2, 'a', vec![], LogDomain::new(0.4).unwrap())]],
    //     'b' => hashmap![1 => vec![(1, 'b', vec![], LogDomain::new(0.2).unwrap())],
    //                     2 => vec![(2, 'b', vec![], LogDomain::new(0.6).unwrap())]],
    //     's' => hashmap![0 => vec![(0, 's', vec![1, 1], LogDomain::new(0.9).unwrap()),
    //                               (0, 's', vec![2, 2], LogDomain::new(0.1).unwrap())],
    //                     1 => vec![(1, 's', vec![1, 2], LogDomain::new(0.3).unwrap())]]],
    // );
    let mpt = pta.most_probable_tree();
    println!("{}\t{}", mpt.1, mpt.0);
}
