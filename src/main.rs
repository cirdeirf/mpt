mod pta;

use pta::PTA;
// use std::collections::HashSet;
use std::time::Instant;

fn main() {
    let pta_string = "root: 0 # 0.7\n\
                      root: 1 # 0.2\n\
                      root: 2 # 0.1\n\
                      transition: 1 -> a() # 0.5\n\
                      transition: 2 -> a() # 0.4\n\
                      transition: 1 -> b() # 0.2\n\
                      transition: 2 -> b() # 0.6\n\
                      transition: 0 -> s(1, 1) # 0.9\n\
                      transition: 0 -> s(2, 2) # 0.1\n\
                      transition: 1 -> s(1, 2) # 0.3";
    let pta: PTA<usize, char> = pta_string.parse().unwrap();
    println!("{}", pta);

    let start_time = Instant::now();
    let mpt = pta.most_probable_tree();
    println!("{}\t{}", mpt.1, mpt.0);
    println!("time: {:?}", start_time.elapsed());

    println!("\nbest_parse");
    let best_parse = pta.best_parse();
    println!("{}\t{}", best_parse.1, best_parse.0);
    println!("\n");

    // prime automaton for Psi = {2, 3}
    let pta_string = "root: 0 # 1.0\n\
                      transition: 2 -> a() # 0.109\n\
                      transition: 5 -> a() # 0.109\n\
                      transition: 0 -> g(1) # 0.5\n\
                      transition: 0 -> g(3) # 0.5\n\
                      transition: 1 -> g(2) # 1.0\n\
                      transition: 2 -> g(1) # 0.891\n\
                      transition: 3 -> g(4) # 1.0\n\
                      transition: 4 -> g(5) # 1.0\n\
                      transition: 5 -> g(3) # 0.891";
    let pta: PTA<usize, char> = pta_string.parse().unwrap();
    println!("{}", pta);

    let start_time = Instant::now();
    let mpt = pta.most_probable_tree();
    println!("{}\t{}", mpt.1, mpt.0);
    println!("time: {:?}", start_time.elapsed());

    println!("\nbest_parse");
    let best_parse = pta.best_parse();
    println!("{}\t{}", best_parse.1, best_parse.0);

    // let mut xi = "(s (b) (a))".parse().unwrap();
    // println!(
    //     "{}\t{}",
    //     pta.prefix_probability(
    //         &mut xi,
    //         &mut HashSet::new(),
    //         &mut HashSet::new()
    //     ),
    //     xi
    // );
}
