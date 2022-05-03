/// Step proof:
/// 
/// Application logic proof and verification of the "wrap" verifier
mod step;

/// Wrap proof:
/// 
/// Proof of correct execution of the "step" verifier
mod wrap;

trait Rule {  }

/// An inductive set defined by an enumerable set of production rules
/// 
/// A set produces a single prover/verifier
trait Set {
    /// The production rule
    /// (usually an enum over with a variant for each possible production rule)
    type Rule: Rule;

    /// An iterable type over the production rules
    type Rules: Iterator<Item = Self::Rule>;

    /// Enumer
    fn enum_rules() -> Self::Rules;
}

enum VoteRules {
    Base(),
    Tally(),
}

impl Rule for VoteRules {

}

/// Production rules for votes
struct Votes {
    total_yes: (), // field element
    totaL_no: (),
}

impl Set for Votes {
    //
    type Rule = VoteRules;

    //
    type Rules = std::array::IntoIter<Self::Rule, 2>;
    
    fn enum_rules() -> Self::Rules {
        [VoteRules::Base(), VoteRules::Tally()].into_iter()
    }
}