mod pta;

use pta::PTA;
use std::time::Instant;

fn main() {
    // let pta_string = "root: [0.7, 0.2, 0.1]\n\
    //                   transition: 1 -> a() # 0.5\n\
    //                   transition: 2 -> a() # 0.4\n\
    //                   transition: 1 -> b() # 0.2\n\
    //                   transition: 2 -> b() # 0.6\n\
    //                   transition: 0 -> s(1, 1) # 0.9\n\
    //                   transition: 0 -> s(2, 2) # 0.1\n\
    //                   transition: 1 -> s(1, 2) # 0.3";
    // prime automaton for Psi = {2, 3}
    let pta_string = "root: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0]\n\
                      transition: 2 -> ax() # 0.109\n\
                      transition: 5 -> ax() # 0.109\n\
                      transition: 0 -> gr(1) # 0.5\n\
                      transition: 0 -> gr(3) # 0.5\n\
                      transition: 1 -> gr(2) # 1.0\n\
                      transition: 2 -> gr(1) # 0.891\n\
                      transition: 3 -> gr(4) # 1.0\n\
                      transition: 4 -> gr(5) # 1.0\n\
                      transition: 5 -> gr(3) # 0.891";
    let pta: PTA<String> = pta_string.parse().unwrap();
    let start_time = Instant::now();
    let mpt = pta.most_probable_tree();
    println!("{}\t{}", mpt.1, mpt.0);
    println!("time: {:?}", start_time.elapsed());
}
