#[macro_use]
mod pta;

use log_domain::LogDomain;
use pta::PTA;

fn main() {
    let pta = PTA::new(
        hashmap!['a' => 0, 'g' => 1, 's' => 2],
        vec![
            LogDomain::new(0.0).unwrap(),
            LogDomain::new(0.0).unwrap(),
            LogDomain::new(1.0).unwrap(),
        ],
        hashmap![
        'a' => hashmap![0 => vec![(0, 'a', vec![], LogDomain::new(0.8).unwrap())],
                        1 => vec![(1, 'a', vec![], LogDomain::new(0.6).unwrap())]],
        'g' => hashmap![0 => vec![(0, 'g', vec![0], LogDomain::new(0.2).unwrap())],
                        1 => vec![(0, 'g', vec![1], LogDomain::new(0.1).unwrap())],
                        1 => vec![(1, 'g', vec![1], LogDomain::new(0.3).unwrap())]],
        's' => hashmap![2 => vec![(2, 's', vec![1, 1], LogDomain::new(0.9).unwrap()),
                                  (2, 's', vec![1, 2], LogDomain::new(0.1).unwrap())]]],
    );
    let mpt = pta.most_probable_tree();
    println!("{}\t{}", mpt.1, mpt.0);
}
