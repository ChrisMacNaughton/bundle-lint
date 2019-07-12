use std::collections::HashMap;

use failure::Error;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::fetch;
use crate::juju::Bundle;

#[cfg(test)]
mod tests {
    use super::*;

    const BUNDLE: &'static str = r#"
applications:
  test-thing:
    charm: cs:test-thing
    options:
      use-cool-thing: True
  test-thing-2:
    charm: cs:test-thing-2
    options:
      conflicts-with-cool-thing: True
  test-thing-3:
    charm: cs:test-thing-3
    options:
      required-by-cool-thing: True
"#;

    #[test]
    fn it_validates_a_basic_forbids_rule() {
        let bundle = Bundle::load(BUNDLE).unwrap();
        let forbids = {
            let mut h = HashMap::new();
            h.insert(
                "test-thing-2".to_string(),
                Config {
                    name: "conflicts-with-cool-thing".to_string(),
                    value: None,
                },
            );
            h
        };
        let rule = Rule {
            charm_name: "test-thing".to_string(),
            config_name: "use-cool-thing".to_string(),
            config_value: "True".to_string(),
            requires: HashMap::new(),
            forbids: forbids,
        };
        let verification = rule.verify(&bundle);

        assert_eq!(verification, VerificationResult::Fail{reason: "test-thing-2 / conflicts-with-cool-thing has an extra config value, forbids True".into()});
    }

    #[test]
    fn it_validates_a_specific_forbids_rule() {
        let bundle = Bundle::load(BUNDLE).unwrap();
        let forbids = {
            let mut h = HashMap::new();
            h.insert(
                "test-thing-2".to_string(),
                Config {
                    name: "conflicts-with-cool-thing".to_string(),
                    value: Some("True".into()),
                },
            );
            h
        };
        let rule = Rule {
            charm_name: "test-thing".to_string(),
            config_name: "use-cool-thing".to_string(),
            config_value: "True".to_string(),
            requires: HashMap::new(),
            forbids: forbids,
        };
        let verification = rule.verify(&bundle);

        assert_eq!(verification, VerificationResult::Fail{reason: "test-thing-2 / conflicts-with-cool-thing has an invalid config value (True), forbids True".into()});
    }

    #[test]
    fn it_validates_a_passing_specific_forbids_rule() {
        let bundle = Bundle::load(BUNDLE).unwrap();
        let forbids = {
            let mut h = HashMap::new();
            h.insert(
                "test-thing-2".to_string(),
                Config {
                    name: "conflicts-with-cool-thing".to_string(),
                    value: Some("False".into()),
                },
            );
            h
        };
        let rule = Rule {
            charm_name: "test-thing".to_string(),
            config_name: "use-cool-thing".to_string(),
            config_value: "True".to_string(),
            requires: HashMap::new(),
            forbids: forbids,
        };
        let verification = rule.verify(&bundle);

        assert_eq!(verification, VerificationResult::Pass);
    }

    #[test]
    fn it_validates_a_basic_requires_rule() {
        let bundle = Bundle::load(BUNDLE).unwrap();
        let requires = {
            let mut h = HashMap::new();
            h.insert(
                "test-thing-3".to_string(),
                Config {
                    name: "required-by-cool-thing".to_string(),
                    value: None,
                },
            );
            h
        };
        let rule = Rule {
            charm_name: "test-thing".to_string(),
            config_name: "use-cool-thing".to_string(),
            config_value: "True".to_string(),
            requires: requires,
            forbids: HashMap::new(),
        };
        let verification = rule.verify(&bundle);
        assert_eq!(verification, VerificationResult::Pass);
    }
}

pub fn import(config_path: &str) -> Result<Vec<Rule>, Error> {
    let new_config_path = config_path.replace("gh:", "https://github.com/");
    debug!("Loading config from {}", new_config_path);
    fetch::import(&fetch::load(&new_config_path)?)
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Rule {
    pub charm_name: String,
    pub config_name: String,
    pub config_value: String,
    #[serde(default)]
    pub requires: HashMap<String, Config>,
    #[serde(default)]
    pub forbids: HashMap<String, Config>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
    pub name: String,
    pub value: Option<String>,
}

impl Config {
    // pub fn matches(&self, config_name: &String, config_value: &String) -> bool {
    //     false
    // }
}

#[derive(Debug, Eq, PartialEq)]
pub enum VerificationResult {
    Pass,
    Fail { reason: String },
}

impl Rule {
    pub fn verify(&self, bundle: &Bundle) -> VerificationResult {
        if let Some(application) = bundle.application(&self.charm_name) {
            if let Some(value) = application.option(&self.config_name) {
                if *value == self.config_value {
                    match self.verify_requires(bundle) {
                        VerificationResult::Pass => {}
                        VerificationResult::Fail { reason: f } => {
                            return VerificationResult::Fail { reason: f }
                        }
                    }
                    match self.verify_forbids(bundle) {
                        VerificationResult::Pass => {}
                        VerificationResult::Fail { reason: f } => {
                            return VerificationResult::Fail { reason: f }
                        }
                    }
                }
            }
        }
        VerificationResult::Pass
    }

    fn verify_requires(&self, bundle: &Bundle) -> VerificationResult {
        for (application, config) in &self.requires {
            if let Some(other_app) = bundle.application(application) {
                if let Some(value) = other_app.option(&config.name) {
                    match config.value {
                        Some(ref v) => {
                            if v != value {
                                return VerificationResult::Fail {
                                    reason: format!(
                                        "{} / {} has an invalid config value ({}), requires {}",
                                        application, config.name, v, value
                                    ),
                                };
                            }
                        }
                        None => {}
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
