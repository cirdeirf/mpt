use integeriser::{HashIntegeriser, Integeriser};
use log_domain::LogDomain;
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Transition<Q, T> {
    pub source_state: Q,
    pub symbol: T,
    pub target_states: Vec<Q>,
    pub probability: LogDomain<f64>,
}

impl<Q, T> Integerisable2 for Transition<Q, T>
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
            probability: self.probability.clone(),
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
            probability: v.probability.clone(),
        }
    }
}

pub trait Integerisable1
where
    Self::I: Integeriser,
{
    type AInt;
    /// type of the integerised self
    type I;
    /// type of the integeriser

    fn integerise(&self, integeriser: &mut Self::I) -> Self::AInt;

    fn un_integerise(_: &Self::AInt, integeriser: &Self::I) -> Self;
}

pub trait Integerisable2
where
    Self::I1: Integeriser,
    Self::I2: Integeriser,
{
    type AInt;
    /// type of the integerised self
    type I1;
    /// type of the first integeriser
    type I2;
    /// type of the second integeriser

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
