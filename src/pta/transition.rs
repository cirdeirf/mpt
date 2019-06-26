use integeriser::{HashIntegeriser, Integeriser};
use log_domain::LogDomain;
use std::hash::Hash;

/// A transition for a probabilistic tree automaton.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Transition<Q, T> {
    pub source_state: Q,
    pub symbol: T,
    pub target_states: Vec<Q>,
    pub probability: LogDomain<f64>,
}

/// Trait for mapping states and symbols to integers and vice versa for easier
/// processing ([rust-integeriser](https://github.com/tud-fop/rust-integeriser).
/// This implementation is adapted from the Integerisable implementation for
/// transitions in [rustomata](https://github.com/tud-fop/rustomata)
/// (rustomata/src/recognisable/transition.rs).
impl<Q, T> Integerisable for Transition<Q, T>
where
    Q: Eq + Clone + Hash,
    T: Eq + Clone + Hash,
{
    type AInt = Transition<usize, usize>;
    type I1 = HashIntegeriser<Q>;
    type I2 = HashIntegeriser<T>;

    fn integerise(
        &self,
        integeriser1: &mut Self::I1,
        integeriser2: &mut Self::I2,
    ) -> Self::AInt {
        Transition {
            source_state: integeriser1.integerise(self.source_state.clone()),
            symbol: integeriser2.integerise(self.symbol.clone()),
            target_states: self
                .target_states
                .iter()
                .map(|q| integeriser1.integerise(q.clone()))
                .collect(),
            probability: self.probability,
        }
    }

    fn un_integerise(
        v: &Self::AInt,
        integeriser1: &Self::I1,
        integeriser2: &Self::I2,
    ) -> Self {
        Transition {
            source_state: integeriser1
                .find_value(v.source_state)
                .unwrap()
                .clone(),
            symbol: integeriser2.find_value(v.symbol).unwrap().clone(),
            target_states: v
                .target_states
                .iter()
                .map(|q| integeriser1.find_value(*q).unwrap().clone())
                .collect(),
            probability: v.probability,
        }
    }
}

pub trait Integerisable
where
    Self::I1: Integeriser,
    Self::I2: Integeriser,
{
    /// type of the integerised self
    type AInt;
    /// type of the first integeriser
    type I1;
    /// type of the second integeriser
    type I2;

    fn integerise(
        &self,
        integeriser1: &mut Self::I1,
        integeriser2: &mut Self::I2,
    ) -> Self::AInt;

    fn un_integerise(
        _: &Self::AInt,
        integeriser1: &Self::I1,
        integeriser2: &Self::I2,
    ) -> Self;
}
