mod pta;

use pta::PTA;
use std::time::Instant;

fn main() {
    // // prime automaton for Psi = {2, 3}
    let pta_string = "root: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0]\n\
                      transition: 2 -> a() # 0.109\n\
                      transition: 5 -> a() # 0.109\n\
                      transition: 0 -> g(1) # 0.5\n\
                      transition: 0 -> g(3) # 0.5\n\
                      transition: 1 -> g(2) # 1.0\n\
                      transition: 2 -> g(1) # 0.891\n\
                      transition: 3 -> g(4) # 1.0\n\
                      transition: 4 -> g(5) # 1.0\n\
                      transition: 5 -> g(3) # 0.891";
    let pta: PTA = pta_string.parse().unwrap();
    let start_time = Instant::now();
    let mpt = pta.most_probable_tree();
    println!("{}\t{}", mpt.1, mpt.0);
    println!("time: {:?}", start_time.elapsed());
}
