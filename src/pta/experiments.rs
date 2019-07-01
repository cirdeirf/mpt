use crate::pta::{Transition, PTA};
use log_domain::LogDomain;
use num_traits::One;
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Generates and saves a pta for testing.
/// The pta is constructed as is described in Section X (TODO) of my master's
/// thesis. Since the symbols are chosen randomly for each transition, it may
/// happen that not enough symbols are included (to adhere to the desired amount
/// of symbols) or that the desired average rank (+- 0.25) is not met. In these
/// cases, an error is returned.
pub fn generate(
    // number of levels, each run includes at least one node from each level
    level: usize,
    // number of nodes per level
    multiplicity: usize,
    // size of the ranked alphabet
    vocabulary_len: usize,
    // average rank, i.e., the average amount of children a transition has
    // (excluding the 'initial' transition X -> $() )
    average_rk: f64,
    filename: &str,
) -> Result<(), &'static str> {
    // generate a ranked alphabet with the specified amount of symbols and an
    // average rank for each symbol that is close average_rk
    let sigma = create_ranked_alphabet(vocabulary_len, average_rk);
    // only symbols without rank for randomly choosing a symbol
    let sigma_keys: Vec<&char> = sigma.keys().collect();
    // random number generator
    let mut rng = thread_rng();
    let mut transition_map = HashMap::new();

    // generate all transitoins
    'outer: for k in 0..level {
        for l in 0..level {
            for m in 0..multiplicity {
                for n in 0..multiplicity {
                    // choose a random symbol
                    let symbol = sigma_keys.choose(&mut rng).unwrap();
                    // create a transition with 'km' as a source state and
                    // rk(symbol) * 'ln' as target states
                    let t: Transition<String, char> = Transition {
                        source_state: k.to_string() + &m.to_string(),
                        symbol: **symbol,
                        target_states: vec![
                            l.to_string() + &n.to_string();
                            *sigma.get(symbol).unwrap()
                        ],
                        probability: LogDomain::new(rng.gen::<f64>()).unwrap(),
                    };
                    // move transition to mapping: states -> transtion
                    // for easier normalisation (properness)
                    transition_map
                        .entry(t.source_state.clone())
                        .or_insert_with(Vec::new)
                        .push(t);
                }
            }
            // transition to each states of level k to all states of level k+1
            if k < l {
                continue 'outer;
            }
        }
    }

    // initial transition with source state 'level+multiplicity', special
    // character '$' and no children, e.g. 32 -> $()
    let initial_state =
        (level - 1).to_string() + &(multiplicity - 1).to_string();
    let initial_t = Transition {
        source_state: initial_state.clone(),
        symbol: '$',
        target_states: vec![],
        probability: LogDomain::new(rng.gen::<f64>()).unwrap(),
    };
    // insert intial transition into transition mapping
    transition_map
        .entry(initial_state)
        .or_insert_with(Vec::new)
        .push(initial_t);

    // make transition mapping proper
    normalise_transition_weights(&mut transition_map);

    // create root weight mapping with single root state '00'
    let mut root_weight_map = HashMap::new();
    root_weight_map.insert(String::from("00"), LogDomain::one());

    let transitions: Vec<Transition<String, char>> =
        transition_map.values().cloned().flatten().collect();
    // calculate the average amount of target states for each transition
    let average_rank = transitions
        .iter()
        .map(|t| t.target_states.len())
        .sum::<usize>() as f64
        / (transitions.len() - 1) as f64;

    // since symbols are chosen randomly it may happen that not enough are
    // contained in the set of transitions
    if transitions
        .iter()
        .map(|t| t.symbol)
        .collect::<HashSet<char>>()
        .len()
        != sigma.len() + 1
    {
        Err("Vocabulary size constraint not adhered to.")
    }
    // same for the average rank, it may happen that the chosen symbols in the
    // transitions do not add up to the desired average rank/number of children
    // (since the disired average rank is often unreachable, it suffices if it
    // is close enough)
    else if (average_rk - average_rank).abs() > 0.25 {
        Err("Average rank constraint not met.")
    }
    // create the new pta and write it to file
    else {
        let pta = PTA::new(root_weight_map, transitions);
        write_to_file(
            &pta,
            level,
            multiplicity,
            sigma.len(),
            average_rank,
            filename,
        );
        Ok(())
    }
}

/// Generates a ranked alphabet over single character from the alphabet of a
/// specified size and with an average rank close the one specified.
fn create_ranked_alphabet(
    vocabulary_len: usize,
    average_rank: f64,
) -> HashMap<char, usize> {
    let mut vocabulary: Vec<char> =
        "abcdefghijklmnopqrstuvwxyz".chars().collect();
    vocabulary.truncate(vocabulary_len);

    let mut sigma: HashMap<char, usize> = HashMap::new();
    for c in vocabulary[..vocabulary.len() - 1].iter() {
        sigma.insert(*c, average_rank.floor() as usize);
    }
    sigma.insert(*vocabulary.last().unwrap(), average_rank.ceil() as usize);

    sigma
}

/// Normalise given transitions, i.e., ensure that the transition mapping is
/// proper (the weights of all transition with the same source state add up to
/// one).
fn normalise_transition_weights(
    transition_map: &mut HashMap<String, Vec<Transition<String, char>>>,
) {
    for transitions in transition_map.values_mut() {
        let weight_sum: LogDomain<f64> =
            transitions.iter().map(|t| t.probability).sum();
        for t in transitions.iter_mut() {
            t.probability /= weight_sum;
        }
    }
}

/// Writes a pta with specified properties to a file in `experiments/pta/`.
fn write_to_file(
    pta: &PTA<String, char>,
    level: usize,
    multiplicity: usize,
    vocabulary_len: usize,
    average_rank: f64,
    filename: &str,
) {
    let path_string = format!("experiments/pta/{}.pta", filename);
    let path = Path::new(&path_string);
    let mut file = match File::create(&path) {
        Ok(file) => file,
        Err(e) => {
            panic!("couldn't create {}: {}", path.display(), e.description())
        }
    };
    match file.write_all(
        format!(
            "% {} / {} / {} / {:.2} \
             (level / multiplicity / vocabulary size / average rank)\n{}",
            level, multiplicity, vocabulary_len, average_rank, pta
        )
        .as_bytes(),
    ) {
        Ok(_) => println!(
            "Created a new pta with {} levels, multiplicity {}, \
             a vocabulary size of {} and an average rank of {:.2} at {}\n",
            level,
            multiplicity,
            vocabulary_len,
            average_rank,
            path.display()
        ),
        Err(e) => {
            panic!("Could not write to {}: {}", path.display(), e.description())
        }
    }
}
