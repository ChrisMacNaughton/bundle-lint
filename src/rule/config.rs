use crate::juju::Bundle;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::rule::VerificationResult;

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
    #[serde(default)]
    pub requires: HashMap<String, ConfigValue>,
    #[serde(default)]
    pub forbids: HashMap<String, ConfigValue>,
}

impl Config {
    pub fn verify(&self, bundle: &Bundle) -> VerificationResult {
        if let VerificationResult::Fail { reason: f } = self.verify_required(&bundle) {
            return VerificationResult::Fail { reason: f };
        }
        if let VerificationResult::Fail { reason: f } = self.verify_forbids(&bundle) {
            return VerificationResult::Fail { reason: f };
        }
        VerificationResult::Pass
    }
    fn verify_required(&self, bundle: &Bundle) -> VerificationResult {
        for (application, config) in &self.requires {
            if let Some(other_app) = bundle.application(application) {
                if let Some(value) = other_app.option(&config.name) {
                    if let Some(ref v) = config.value {
                        if v != value {
                            return VerificationResult::Fail {
                                reason: format!(
                                    "{} / {} has an invalid config value ({}), requires {}",
                                    application, config.name, v, value
                                ),
                            };
                        }
                    }
                } else {
                    return VerificationResult::Fail {
                        reason: format!(
                            "{} / {} has a missing config value",
                            application, config.name
                        ),
                    };
                }
            }
        }
        VerificationResult::Pass
    }

    fn verify_forbids(&self, bundle: &Bundle) -> VerificationResult {
        for (application, config) in &self.forbids {
            if let Some(other_app) = bundle.application(application) {
                if let Some(value) = other_app.option(&config.name) {
                    match config.value {
                        Some(ref v) => {
                            if v == value {
                                return VerificationResult::Fail {
                                    reason: format!(
                                        "{} / {} has an invalid config value ({}), forbids {}",
                                        application, config.name, v, value
                                    ),
                                };
                            }
                        }
                        None => {
                            return VerificationResult::Fail {
                                reason: format!(
                                    "{} / {} has an extra config value, forbids {}",
                                    application, config.name, value
                                ),
                            }
                        }
                    }
                }
            }
        }
        VerificationResult::Pass
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ConfigValue {
    pub name: String,
    pub value: Option<String>,
}
