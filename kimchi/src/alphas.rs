//! This module implements an abstraction to keep track of the powers of alphas.
//! As a recap, alpha is a challenge sent by the verifier in PLONK,
//! and is used to aggregate multiple constraints into a single polynomial.
//! It is important that different constraints use different powers of alpha,
//! as otherwise they can interact and potentially cancel one another.
//! (The proof is in the use of the Schwartz-Zippel lemma.)
//! As such, we want two properties from this:
//!
//! - we should keep track of a mapping between type of constraint and range of powers
//! - when powers of alphas are used, we should ensure that no more no less are used
//!
//! We use powers of alpha in two different places in the codebase:
//!
//! - when creating the index, we do not know alpha at this point so we
//!   simply keep track of what constraints will use what powers
//! - when creating a proof or verifying a proof, at this point we know alpha
//!   so we can use the mapping we created during the creation of the index.
//!
//! For this to work, we use two types:
//!
//! - [Builder], which allows us to map constraints to powers
//! - [Alphas], which you can derive from [Builder] and an `alpha`
//!
//! Both constructions will enforce that you use all the powers of
//! alphas that you register for constraint. This allows us to
//! make sure that we compute the correct amounts, without reusing
//! powers of alphas between constraints.

use ark_ff::Field;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    iter::{Cloned, Take},
    ops::Range,
    slice::Iter,
    thread,
};

use crate::circuits::argument::ArgumentType;

/// This type can be used to create a mapping between powers of alpha and constraint types.
/// See [Alphas::default] to create one,
/// and [Builder::register] to register a new mapping.
/// Once you know the alpha value, you can convert this type to a [Alphas].
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Alphas<F> {
    /// The next power of alpha to use
    /// the end result will be [1, alpha^next_power)    
    next_power: usize,
    /// The mapping between constraint types and powers of alpha
    mapping: HashMap<ArgumentType, Range<usize>>,
    /// The powers of alpha: 1, alpha, alpha^2, etc.
    /// If set to [Some], you can't register new constraints.
    alphas: Option<Vec<F>>,
}

impl<F: Field> Alphas<F> {
    /// Registers a new [ArgumentType],
    /// associating it with a number `powers` of powers of alpha.
    pub fn register(&mut self, ty: ArgumentType, powers: usize) {
        if self.alphas.is_some() {
            panic!("you cannot register new constraints once initialized with a field element");
        }
        let new_power = self.next_power + powers;
        let range = self.next_power..new_power;
        if self.mapping.insert(ty, range.clone()).is_some() {
            panic!("cannot re-register {:?}", ty);
        }
        self.next_power = new_power;
    }

    /// Returns a range of exponents, for a given [ArgumentType], upperbounded by `num`.
    /// Note that this function will panic if you did not register enough powers of alpha.
    pub fn get_exponents(
        &self,
        ty: ArgumentType,
        num: usize,
    ) -> MustConsumeIterator<Take<Range<usize>>, usize> {
        let range = self
            .mapping
            .get(&ty)
            .unwrap_or_else(|| panic!("constraint {:?} was not registered", ty));
        MustConsumeIterator {
            inner: range.clone().take(num),
            debug_info: ty,
        }
    }

    /// Instantiates the ranges with an actual field element `alpha`.
    /// Once you call this function, you cannot register new constraints via [register].
    pub fn instantiate(&mut self, alpha: F) {
        let mut last_power = F::one();
        let mut alphas = Vec::with_capacity(self.next_power);
        alphas.push(F::one());
        for _ in 0..(self.next_power - 1) {
            last_power *= alpha;
            alphas.push(last_power);
        }
        self.alphas = Some(alphas);
    }

    /// This function allows us to retrieve the powers of alpha, upperbounded by `num`
    pub fn get_alphas(
        &self,
        ty: ArgumentType,
        num: usize,
    ) -> MustConsumeIterator<Cloned<Take<Iter<F>>>, F> {
        let range = self
            .mapping
            .get(&ty)
            .unwrap_or_else(|| panic!("constraint {:?} was not registered", ty))
            .clone();
        match &self.alphas {
            None => panic!("you must call instantiate with an actual field element first"),
            Some(alphas) => {
                let alphas_range = alphas[range].iter().take(num).cloned();
                MustConsumeIterator {
                    inner: alphas_range,
                    debug_info: ty,
                }
            }
        }
    }
}

// ------------------------------------------

/// Wrapper around an iterator that warns you if not consumed entirely.
#[derive(Debug)]
pub struct MustConsumeIterator<I, T>
where
    I: Iterator<Item = T>,
    T: std::fmt::Display,
{
    inner: I,
    debug_info: ArgumentType,
}

impl<I, T> Iterator for MustConsumeIterator<I, T>
where
    I: Iterator<Item = T>,
    T: std::fmt::Display,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<I, T> Drop for MustConsumeIterator<I, T>
where
    I: Iterator<Item = T>,
    T: std::fmt::Display,
{
    fn drop(&mut self) {
        if let Some(v) = self.inner.next() {
            if thread::panicking() {
                eprintln!("the registered number of powers of alpha for {:?} is too large, you haven't used alpha^{} (absolute power of alpha)", self.debug_info,
                v);
            } else {
                panic!("the registered number of powers of alpha for {:?} is too large, you haven't used alpha^{} (absolute power of alpha)", self.debug_info,
                v);
            }
        }
    }
}

// ------------------------------------------

#[cfg(test)]
mod tests {
    use mina_curves::pasta::Fp;

    use super::*;

    // testing [Builder]

    #[test]
    fn incorrect_alpha_powers() {
        let mut alphas = Alphas::<Fp>::default();
        alphas.register(ArgumentType::Gate, 3);

        let mut powers = alphas.get_exponents(ArgumentType::Gate, 3);
        assert_eq!(powers.next(), Some(0));
        assert_eq!(powers.next(), Some(1));
        assert_eq!(powers.next(), Some(2));

        alphas.register(ArgumentType::Permutation, 3);
        let mut powers = alphas.get_exponents(ArgumentType::Permutation, 3);

        assert_eq!(powers.next(), Some(3));
        assert_eq!(powers.next(), Some(4));
        assert_eq!(powers.next(), Some(5));
    }

    #[test]
    #[should_panic]
    fn register_after_instantiating() {
        let mut alphas = Alphas::<Fp>::default();
        alphas.instantiate(Fp::from(1));
        alphas.register(ArgumentType::Gate, 3);
    }

    #[test]
    #[should_panic]
    fn didnt_use_all_alpha_powers() {
        let mut alphas = Alphas::<Fp>::default();
        alphas.register(ArgumentType::Permutation, 7);
        let mut powers = alphas.get_exponents(ArgumentType::Permutation, 3);
        powers.next();
    }

    #[test]
    #[should_panic]
    fn registered_alpha_powers_for_some_constraint_twice() {
        let mut alphas = Alphas::<Fp>::default();
        alphas.register(ArgumentType::Gate, 2);
        alphas.register(ArgumentType::Gate, 3);
    }

    #[test]
    fn powers_of_alpha() {
        let mut alphas = Alphas::default();
        alphas.register(ArgumentType::Gate, 4);
        let mut powers = alphas.get_exponents(ArgumentType::Gate, 4);

        assert_eq!(powers.next(), Some(0));
        assert_eq!(powers.next(), Some(1));
        assert_eq!(powers.next(), Some(2));
        assert_eq!(powers.next(), Some(3));

        let alpha = Fp::from(2);
        alphas.instantiate(alpha);

        let mut alphas = alphas.get_alphas(ArgumentType::Gate, 4);
        assert_eq!(alphas.next(), Some(1.into()));
        assert_eq!(alphas.next(), Some(2.into()));
        assert_eq!(alphas.next(), Some(4.into()));
        assert_eq!(alphas.next(), Some(8.into()));
    }
}