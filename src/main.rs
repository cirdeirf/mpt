//! This is the implementation for my master's thesis "The Problem of Computing
//! the Most Probable Tree of a Probabilistic Tree Automaton". It includes an
//! algorithm for calculating the most probable tree (mpt) and an implementation
//! of the best parse algorithm presented in "Parsing Algorithms based on Tree
//! Automata" by Andreas Maletti and Giorgio Satta, 2009.
//!
//! This project is licensed unter the terms of the GNU General Public License v3.0.
//!
//! # Example
//!
//! The example pta from the thesis:
//! ```
//! root: q0 # 0.9
//! root: q1 # 0.1
//! transition: q1 -> α() # 0.1
//! transition: q2 -> α() # 0.5
//! transition: q2 -> β() # 0.5
//! transition: q1 -> γ(q1) # 0.5
//! transition: q1 -> γ(q2) # 0.3
//! transition: q1 -> σ(q1, q2) # 0.1
//! transition: q0 -> σ(q1, q2) # 1.0
//! ```
//! Every root weight/transition not mentioned is assumed to have a probability of
//! zero. For states and symbols all strings are allowed that do not contain any of
//! the following characters: `'"'`, `' '`, `'-'`, `'>'`, `'→'`, `','`, `';'`,
//! `'('`, `')'`, `'['`, `']'`, `'%'`.
//!
//! The most probable tree for this example pta can be calculated by calling
//! ```
//! cargo run
//! ```
//! which should print
//! ```
//! mpt:		 σ( γ( α ), β )
//! probability:	 0.09100000000000004
//! insertions:	 13
//! time:		 1.226144ms
//! ```
//! to the standard output.
//!
//! Similarly, the best run for the example is computed by calling
//! ```
//! cargo run -- --best-parse
//! ```
//! which yields
//! ```
//! best parse:	 σ( γ( β ), β )
//! probability:	 0.06749999999999999
//! time:		 227.69µs
//! ```
//!
//! ## Ambiguity
//!
//! Note that the implementation does not choose symbols σ ∈ Σ in the same order
//! every time it is called. Therefore it can happen that fewer insertions are
//! necessary when a new current best complete tree has been found that prevents the
//! insertion of another tree, e.g., for the example: We extend tree ξ = `σ( γ( x₁),
//! β )` to a tree ξ' = `σ( γ( σ( x₂, x₃ ) ), β )` with probability `0.02275` and
//! afterwards, we extend ξ to `σ( γ( α ), β )` with the higher probability of
//! `0.091`. If we would have extended with α first, we would not have created ξ'.
//! Additionally, there might be multiple mpt. For the example, the following output
//! is therefore valid as well:
//! ```
//! mpt:		 σ( γ( α ), α )
//! probability:	 0.09100000000000004
//! insertions:	 12
//! time:		 1.887662ms
//! ```
//!
//! # Experiments
//!
//! The experiments have been conducted on a set of synthetic automata. How these
//! are constructed is described in Section 5.4.1. A new test set with the
//! parameters from the thesis (level, multiplicity, alphabet size) can be generated
//! by invoking:
//! ```
//! cargo run -- --generate
//! ```
//!
//! The specific set of synthetic pta we used can be found in
//! `experiments/pta/test1/`. In order to automatically calculate the most probable
//! tree for all these automata, one has to call the following command:
//! ```
//! cargo run --release -- --experiments
//! ```
//! Analogously, for the best run with the `--best-parse` flag. Note that
//! ``--release`` is not necessary but improves the runtime.
//!
//! # Help
//!
//! The following command line arguments are available:
//! ```
//! cargo run -- --help
//!
//! mpt 0.1
//! Pius Meinert <yrr+work@pm.me>
//! Most probable tree and best parse algorithms for probabilistic tree automata (pta).
//! Implementation for my master's thesis: "The Problem of Computing the Most Probable Tree of a Probabilistic Tree
//! Automaton". By default, the most probable tree algorithm is executed.
//!
//! USAGE:
//!     mpt [FLAGS] INPUT
//!
//! FLAGS:
//!     -b, --best-parse     Calculate the tree with the best run (cf. Figure 3, Maletti and Satta, 2009)
//!     -e, --experiments    Calculate the most probable tree/best run for all pta in the test set: experiments/pta/test1/.
//!     -g, --generate       Generate a number of synthetic automata with the parameters (level, multiplicity, alphabet
//!                          size) as used during testing for the thesis. They are saved in: experiments/pta/.
//!     -h, --help           Prints help information
//!     -V, --version        Prints version information
//!     -v, --verbose        Set level of verbosity:
//!                          v: print the pta (root weight mapping and transitions),
//!                          vv: + output current best tree (only for mpt),
//!                          vvv: + print variables after each iteration of the while loop (only mpt).
//!
//! ARGS:
//!     INPUT    Set the input file to use [default: experiments/pta/manually_constructed/example.pta]
//! ```

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
