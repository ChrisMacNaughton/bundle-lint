#[macro_use]
extern crate failure;

pub(crate) mod fetch;
pub mod juju;
mod rule;

pub use rule::{Rule, VerificationResult};
pub use rule::import as import_rules;

// This is a new error type that you've created. It represents the ways a
// toolchain could be invalid.
//
// The custom derive for Fail derives an impl of both Fail and Display.
// We don't do any other magic like creating new types.
#[derive(Debug, Fail)]
pub enum JujuLintError {
    #[fail(display = "Bundle failed lint")]
    LintFailure,
}
