mod pta;

use clap::{App, Arg};
use pta::PTA;
use std::fs;
use std::time::Instant;

fn main() {
    let matches = App::new("mpt")
        .version("0.1")
        .author("Pius Meinert <yrr+work@pm.me>")
        .about(
            "Most probable tree and best parse algorithms for probabilistic \
             tree automata (pta).\n\
             Implementation for my master's thesis: “The Problem of Computing \
             the Most Probable Tree of a Probabilistic Tree Automaton”.",
        )
        .arg(
            Arg::with_name("INPUT")
                .default_value("experiments/default.pta")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .help("Print the PTA (root weight vector and transitions)"),
        )
        .arg(
            Arg::with_name("best_parse")
                .short("b")
                .long("best-parse")
                .help("foo"),
        )
        .get_matches();

    let input = fs::read_to_string(matches.value_of("INPUT").unwrap())
        .expect("Could not read input file");
    let pta: PTA<String, String> = input.parse().unwrap();

    if matches.is_present("verbose") {
        println!("{}", pta);
    }

    let start_time = Instant::now();

    if matches.is_present("best_parse") {
        let best_parse = pta.best_parse();
        println!("{}\t{}", best_parse.1, best_parse.0);
    } else {
        let mpt = pta.most_probable_tree();
        println!("{}\t{}", mpt.1, mpt.0);
    }

    println!("time: {:?}", start_time.elapsed());
}
