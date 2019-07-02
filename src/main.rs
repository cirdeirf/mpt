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
use glob::glob;
use pta::{experiments, PTA};
use std::path::Path;
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
                .conflicts_with_all(&["experiments", "best_parse"])
                .help("foo"),
        )
        .arg(
            Arg::with_name("experiments")
                .short("e")
                .long("experiments")
                .conflicts_with_all(&["generate"])
                .help("foo"),
        )
        .get_matches();

    // generate all test pta (with varying amount of level, multiplicity, number
    // of symbols and average rank)
    if matches.is_present("generate") {
        // generate a tree with properties:
        // #level, multiplicity, #symbols, average rank +- 0.25
        for level in 3..6 {
            for multiplicity in 2..4 {
                for vocabulary_len in 2..7 {
                    for average_rank in vec![1.0, 1.5, 2.0, 2.5, 3.0] {
                        // TODO more than one pta per configuration
                        for _ in 0..1 {
                            let mut tries = 0;
                            while let Err(e) = experiments::generate(
                                level,
                                multiplicity,
                                vocabulary_len,
                                average_rank,
                                &format!(
                                    "l{}_m{}_v{}_rk{:.1}",
                                    level,
                                    multiplicity,
                                    vocabulary_len,
                                    average_rank
                                )
                                .replace(".", ","),
                            ) {
                                if matches.is_present("verbose") {
                                    println!("{} Trying again.", e);
                                }
                                tries += 1;
                                if tries > 2000 {
                                    panic!(
                    "Maximum number of tries (2000) exceeded. Adjust desired \
                    PTA parameters or try again."
                );
                                }
                            }
                        }
                    }
                }
            }
        }
    } else if matches.is_present("experiments") {
        for entry in
            glob("experiments/pta/*.pta").expect("Failed to read glob pattern.")
        {
            if let Err(_) = entry {
                continue;
            }
            let path = &entry.unwrap();
            println!("{}", &path.display());
            // read input file and parse input to pta
            let (pta, info) = PTA::<String, String>::from_file(path);
            // print the input pta (root weight vector and transitions)
            if matches.is_present("verbose") {
                println!("{}", pta);
            }

            let start_time = Instant::now();
            println!("{}", info);
            if matches.is_present("best_parse") {
                let best_parse = pta.best_parse();
                println!("best parse:\t {}", best_parse.0);
                println!("probability:\t {}", best_parse.1);
            } else {
                match pta.most_probable_tree() {
                    Ok(mpt) => {
                        println!("mpt:\t\t {}", mpt.0);
                        println!("probability:\t {}", mpt.1);
                        println!("insertions:\t {}", mpt.2);
                    }
                    Err(e) => println!("{}", e),
                }
            }
            println!("time:\t\t {:?}\n", start_time.elapsed());
        }
    }
    // calculate and output the best parse/most probable tree
    else {
        // read input file and parse input to pta
        let pta: PTA<String, String> =
            PTA::from_file(Path::new(&matches.value_of("INPUT").unwrap())).0;
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
            match pta.most_probable_tree() {
                Ok(mpt) => {
                    println!("mpt:\t\t {}", mpt.0);
                    println!("probability:\t {}", mpt.1);
                    println!("insertions:\t {}", mpt.2);
                }
                Err(e) => panic!("{}", e),
            }
        }
        println!("time:\t\t {:?}", start_time.elapsed());
    }
}
