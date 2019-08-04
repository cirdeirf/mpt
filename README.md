# Most Probable Tree Algorithm

This is a Rust-based implementation of the most probable tree (mpt) algorithm
introduced in "The Problem of Computing the Most Probable Tree of a
Probabilistic Tree Automaton" and the best run algorithm presented in "Parsing
Algorithms based on Tree Automata" by Andreas Maletti and Giorgio Satta, 2009.
More information about the mpt algorithm and the classes can be found in the
thesis itself, in the comments of the source code and in the rustdoc
documentation which can be generated and opened by:
```
cargo doc --document-private-items --open
```
This project is licensed unter the terms of the GNU General Public License v3.0.

## Setup

This program requires Rust and Cargo (Rust build tool and package manager).
Both are available from the Rust programming
language website (<https://www.rust-lang.org/>). For even simpler use, we
provide a binary (`mpt`).

## Example

The example pta from the thesis is available in
`experiments/pta/manually_constructed/example.pta`:
```
root: q0 # 0.9
root: q1 # 0.1
transition: q1 -> α() # 0.1
transition: q2 -> α() # 0.5
transition: q2 -> β() # 0.5
transition: q1 -> γ(q1) # 0.5
transition: q1 -> γ(q2) # 0.3
transition: q1 -> σ(q1, q2) # 0.1
transition: q0 -> σ(q1, q2) # 1.0
```
Every root weight/transition not mentioned is assumed to have a probability of
zero. For states and symbols all strings are allowed that do not contain any of
the following characters: `'"'`, `' '`, `'-'`, `'>'`, `'→'`, `','`, `';'`,
`'('`, `')'`, `'['`, `']'`, `'%'`.

The most probable tree for this example pta can be calculated by calling
```
cargo run
```
which should print
```
mpt:		 σ( γ( α ), β )
probability:	 0.09100000000000004
insertions:	 13
time:		 1.226144ms
```
to the standard output.

Similarly, the best run for the example is computed by calling
```
cargo run -- --best-parse
```
which yields
```
best parse:	 σ( γ( β ), β )
probability:	 0.06749999999999999
time:		 227.69µs
```

### Ambiguity

Note that the implementation does not choose symbols σ ∈ Σ in the same order
every time it is called. Therefore it can happen that fewer insertions are
necessary when a new current best complete tree has been found that prevents the
insertion of another tree, e.g., for the example: We extend tree ξ = `σ( γ( x₁),
β )` to a tree ξ' = `σ( γ( σ( x₂, x₃ ) ), β )` with probability `0.02275` and
afterwards, we extend ξ to `σ( γ( α ), β )` with the higher probability of
`0.091`. If we would have extended with α first, we would not have created ξ'.
Additionally, there might be multiple mpt. For the example, the following output
is therefore valid as well:
```
mpt:		 σ( γ( α ), α )
probability:	 0.09100000000000004
insertions:	 12
time:		 1.887662ms
```

## Experiments

The experiments have been conducted on a set of synthetic automata. How these
are constructed is described in Section 5.4.1. A new test set with the
parameters from the thesis (level, multiplicity, alphabet size) can be generated
by invoking:
```
cargo run -- --generate
```

The specific set of synthetic pta we used can be found in
`experiments/pta/test1/`. In order to automatically calculate the most probable
tree for all these automata, one has to call the following command:
```
cargo run --release -- --experiments
```
Analogously, for the best run with the `--best-parse` flag. Note that
``--release`` is not necessary but improves the runtime (the binary is compiled
with this flag).

The experiments were conducted on an Intel® Xeon® Silver 4114 CPU
with 2.20GHz and the ouput is available in `experiments/test1_mpt.log` and
`experiments/test1_best_parse.log`.

## Help

The following command line arguments are available:
```
❯ cargo run -- --help

mpt 0.1
Pius Meinert <yrr+work@pm.me>
Most probable tree and best parse algorithms for probabilistic tree automata (pta).
Implementation for my master's thesis: "The Problem of Computing the Most Probable Tree of a Probabilistic Tree
Automaton". By default, the most probable tree algorithm is executed.

USAGE:
    mpt [FLAGS] <INPUT>

FLAGS:
    -b, --best-parse     Calculate the tree with the best run (cf. Figure 3, Maletti and Satta, 2009)
    -e, --experiments    Calculate the most probable tree/best run for all pta in the test set: experiments/pta/test1/.
    -g, --generate       Generate a number of synthetic automata with the parameters (level, multiplicity, alphabet
                         size) as used during testing for the thesis. They are saved in: experiments/pta/.
    -h, --help           Prints help information
    -V, --version        Prints version information
    -v, --verbose        Set level of verbosity: 
                         v: print the pta (root weight mapping and transitions), 
                         vv: + output current best tree (only for mpt), 
                         vvv: + print variables after each iteration of the while loop (only mpt).

ARGS:
    <INPUT>    Set the input file to use [default: experiments/pta/manually_constructed/example.pta]
```
