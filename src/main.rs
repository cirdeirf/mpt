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
                .default_value(
                    "experiments/pta/manually_constructed/example.pta",
                )
                .help("Set the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Set level of verbosity: \n\
                v: print the pta (root weight mapping and transitions), \n\
                vv: + output current best tree (only for mpt), \n\
                vvv: + print variables after each iteration of the while loop \
                (only mpt)."),
        )
        .arg(
            Arg::with_name("best_parse")
                .short("b")
                .long("best-parse")
                .help(
                    "Calculate the tree with the best run (cf. Figure 3, \
                     Maletti and Satta, 2009)",
                ),
        )
        .arg(
            Arg::with_name("generate")
                .short("g")
                .long("generate")
                .conflicts_with_all(&["experiments", "best_parse"])
                .help("Generate a number of synthetic automata with the \
                parameters (level, multiplicity, alphabet size) as used during \
                testing for the thesis. They are saved in: experiments/pta/."),
        )
        .arg(
            Arg::with_name("experiments")
                .short("e")
                .long("experiments")
                .conflicts_with_all(&["generate"])
                .help("Calculate the most probable tree/best run for all pta \
                in the test set: experiments/pta/test1/."),
        )
        .get_matches();

    // generate all test pta (with varying amount of level, multiplicity, number
    // of symbols and average rank)
    if matches.is_present("generate") {
        // generate a tree with properties:
        // #level, multiplicity, #symbols, average rank +- 0.2
        for level in 2..5 {
            for multiplicity in 2..4 {
                for vocabulary_len in 2..6 {
                    for average_rank in &[1.0, 1.5, 2.0, 2.5] {
                        for i in 0..10 {
                            let mut tries = 0;
                            while let Err(e) = experiments::generate(
                                level,
                                multiplicity,
                                vocabulary_len,
                                *average_rank,
                                &format!(
                                    "l{}_m{}_v{}_rk{:.1}__{}",
                                    level,
                                    multiplicity,
                                    vocabulary_len,
                                    average_rank,
                                    i
                                )
                                .replace(".", ","),
                            ) {
                                if matches.is_present("verbose") {
                                    println!("{} Trying again.", e);
                                }
                                tries += 1;
                                if tries > 10000 {
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
        for entry in glob("experiments/pta/test1/*.pta")
            .expect("Failed to read glob pattern.")
        {
            if entry.is_err() {
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
                match pta.most_probable_tree(matches.occurrences_of("verbose"))
                {
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
            match pta.most_probable_tree(matches.occurrences_of("verbose")) {
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
