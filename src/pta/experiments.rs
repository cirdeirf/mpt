use crate::pta::{Transition, PTA};
use log_domain::LogDomain;
use num_traits::One;
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn generate(
    level: usize,
    multiplicity: usize,
    vocabulary: Vec<&str>,
    filename: &str,
) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(
        ((level << 16) + (multiplicity << 8) + vocabulary.len()) as u64,
    );
    let mut transition_map = HashMap::new();
    'outer: for k in 0..level {
        for l in 0..level {
            for m in 0..multiplicity {
                for n in 0..multiplicity {
                    let t = Transition {
                        source_state: k.to_string() + &m.to_string(),
                        symbol: String::from(
                            *vocabulary.choose(&mut rng).unwrap(),
                        ),
                        target_states: vec![l.to_string() + &n.to_string()],
                        probability: LogDomain::new(rng.gen::<f64>()).unwrap(),
                    };
                    transition_map
                        .entry(t.source_state.clone())
                        .or_insert_with(Vec::new)
                        .push(t);
                }
            }
            if k < l {
                continue 'outer;
            }
        }
    }
    let initial_state =
        (level - 1).to_string() + &(multiplicity - 1).to_string();
    let initial_t = Transition {
        source_state: initial_state.clone(),
        symbol: String::from("$"),
        target_states: vec![],
        probability: LogDomain::new(rng.gen::<f64>()).unwrap(),
    };
    transition_map
        .entry(initial_state)
        .or_insert_with(Vec::new)
        .push(initial_t);
    normalise_transition_weights(&mut transition_map);
    let mut root_weight_map = HashMap::new();
    root_weight_map.insert(String::from("00"), LogDomain::one());
    let pta = PTA::new(
        root_weight_map,
        transition_map.values().cloned().flatten().collect(),
    );
    write_to_file(&pta, level, multiplicity, vocabulary.len(), filename);
}

fn normalise_transition_weights(
    transition_map: &mut HashMap<String, Vec<Transition<String, String>>>,
) {
    for transitions in transition_map.values_mut() {
        let weight_sum: LogDomain<f64> =
            transitions.iter().map(|t| t.probability).sum();
        for t in transitions.iter_mut() {
            t.probability /= weight_sum;
        }
    }
}

fn write_to_file(
    pta: &PTA<String, String>,
    level: usize,
    multiplicity: usize,
    vocabulary_len: usize,
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
            "% {} / {} / {} (level / multiplicity / vocabulary size)\n{}",
            level, multiplicity, vocabulary_len, pta
        )
        .as_bytes(),
    ) {
        Ok(_) => println!(
            "Created a new pta with {} levels, multiplicity {} \
             and a vocabulary size of {} at {}",
            level,
            multiplicity,
            vocabulary_len,
            path.display()
        ),
        Err(e) => {
            panic!("couldn't write to {}: {}", path.display(), e.description())
        }
    }
}
