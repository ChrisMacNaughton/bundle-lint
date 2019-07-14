use crate::juju::Bundle;
use serde::{Deserialize, Serialize};

use crate::rule::VerificationResult;

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Relation {
    pub requires: Option<String>,
    pub forbids: Option<String>,
}

impl Relation {
    pub fn verify(&self, bundle: &Bundle) -> VerificationResult {
        if let VerificationResult::Fail { reason: f } = self.verify_required(&bundle) {
            return VerificationResult::Fail { reason: f };
        }
        if let VerificationResult::Fail { reason: f } = self.verify_forbids(&bundle) {
            return VerificationResult::Fail { reason: f };
        }
        VerificationResult::Pass
    }

    fn verify_required(&self, _bundle: &Bundle) -> VerificationResult {
        VerificationResult::Pass
    }

    fn verify_forbids(&self, _bundle: &Bundle) -> VerificationResult {
        VerificationResult::Pass
    }
}
