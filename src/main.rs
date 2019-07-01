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
            Arg::with_name("generate")
                .short("g")
                .long("generate")
                .conflicts_with_all(&["experiments", "best_parse", "verbose"])
                .help("foo"),
        )
        .arg(
            Arg::with_name("experiments")
                .short("e")
                .long("experiments")
                .conflicts_with_all(&["generate", "best_parse", "verbose"])
                .help("foo"),
        )
        .get_matches();

    // generate all test pta (with varying amount of level, multiplicity, number
    // of symbols and average rank)
    if matches.is_present("generate") {
        let mut tries = 0;
        // generate a tree with properties:
        // #level, multiplicity, #symbols, average rank +- 0.25
        for level in 3..6 {
            for multiplicity in 2..4 {
                for vocabulary_len in 2..7 {
                    for average_rank in vec![1.0, 1.5, 2.0, 2.5, 3.0] {
                        for _ in 0..1 {
                            while let Err(e) = experiments::generate(
                                level,
                                multiplicity,
                                vocabulary_len,
                                average_rank,
                                &format!(
                                    "l{}_m{}_v{}_rk{}",
                                    level,
                                    multiplicity,
                                    vocabulary_len,
                                    average_rank.to_string().replace(".", ","),
                                ),
                            ) {
                                println!("{} Trying again.", e);
                                tries += 1;
                                if tries > 20 {
                                    panic!(
                    "Maximum number of tries exceeded. Adjust desired PTA \
                     parameters."
                );
                                }
                            }
                        }
                    }
                }
            }
        }
    } else if matches.is_present("experiments") {

    }
    // calculate and output the best parse/most probable tree
    else {
        // read input file and parse input to pta
        let pta: PTA<String, String> =
            PTA::from_file(&matches.value_of("INPUT").unwrap());
        // print the input pta (root weight vector and transitions)
        if matches.is_present("verbose") {
            println!("{}", pta);
        }

        let start_time = Instant::now();
        if matches.is_present("best_parse") {
            let best_parse = pta.best_parse();
            println!("best parse:\t {}", best_parse.0);
            println!("probability:\t {}", best_parse.1);
        } else {
            let mpt = pta.most_probable_tree();
            println!("mpt:\t\t {}", mpt.0);
            println!("probability:\t {}", mpt.1);
            println!("insertions:\t {}", mpt.2);
        }
        println!("time:\t\t {:?}", start_time.elapsed());
    }
}
