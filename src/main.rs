//! This is the implementation for my master's thesis "The Problem of Computing
//! the Most Probable Tree of a Probabilistic Tree Automaton". It includes an
//! algorithm for calculating the most probable tree (mpt) and an implementation
//! of the best parse algorithm presented in "Parsing Algorithms based on Tree
//! Automata" by Andreas Maletti and Giorgio Satta, 2009.
//!
//! # Setup
//!
//! # Example
//!
//! # Experiments

mod pta;

use clap::{App, Arg};
use pta::experiments;
use pta::PTA;
use std::fs;
use std::time::Instant;

fn main() {
    // set command line arguments/options
    let matches = App::new("mpt")
        .version("0.1")
        .author("Pius Meinert <yrr+work@pm.me>")
        .about(
            "Most probable tree and best parse algorithms for probabilistic \
             tree automata (pta).\n\
             Implementation for my master's thesis: \"The Problem of Computing \
             the Most Probable Tree of a Probabilistic Tree Automaton\". By \
             default, the most probable tree algorithm is executed.",
        )
        .arg(
            Arg::with_name("INPUT")
                .default_value("experiments/pta/default.pta")
                .help("Set the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Print the PTA (root weight vector and transitions)"),
        )
        .arg(
            Arg::with_name("best_parse")
                .short("b")
                .long("best-parse")
                .help(
                    "Calculate the tree with the best parse (Figure 3, \
                     Maletti and Satta, 2009)",
                ),
        )
        .arg(
            Arg::with_name("experiments")
                .short("e")
                .long("experiments")
                .conflicts_with_all(&["verbose", "best_parse"])
                .help("foo"),
        )
        .get_matches();

    // read input file and parse input to pta
    let input = fs::read_to_string(matches.value_of("INPUT").unwrap())
        .expect("Could not read input file");
    let pta: PTA<String, String> = input.parse().unwrap();

    // print the input pta (root weight vector and transitions)
    if matches.is_present("verbose") {
        println!("{}", pta);
    }

    let start_time = Instant::now();

    // calculate and output the best parse/most probable tree
    if matches.is_present("best_parse") {
        let best_parse = pta.best_parse();
        println!("best parse:\t {}", best_parse.0);
        println!("probability:\t {}", best_parse.1);
    } else if !matches.is_present("experiments") {
        let mpt = pta.most_probable_tree();
        println!("mpt:\t\t {}", mpt.0);
        println!("probability:\t {}", mpt.1);
    }

    if matches.is_present("experiments") {
        experiments::generate(3, 2, vec!["a", "b", "c", "d", "e", "f"], "test");
    } else {
        println!("time:\t\t {:?}", start_time.elapsed());
    }
}
