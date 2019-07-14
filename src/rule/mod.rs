use std::collections::HashMap;

use failure::Error;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::fetch;
use crate::juju::Bundle;

mod config;
mod relation;

pub use config::{Config, ConfigValue};
pub use relation::Relation;

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
                ConfigValue {
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
            config: Config {
                requires: HashMap::new(),
                forbids,
            },
            relation: Relation::default(),
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
                ConfigValue {
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
            config: Config {
                requires: HashMap::new(),
                forbids,
            },
            relation: Relation::default(),
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
                ConfigValue {
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
            config: Config {
                requires: HashMap::new(),
                forbids,
            },
            relation: Relation::default(),
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
                ConfigValue {
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
            config: Config {
                requires,
                forbids: HashMap::new(),
            },
            relation: Relation::default(),
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
    pub config: Config,
    #[serde(default)]
    pub relation: Relation,
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
                    if let VerificationResult::Fail { reason: f } = self.config.verify(&bundle) {
                        return VerificationResult::Fail { reason: f };
                    }
                }
            }
        }
        VerificationResult::Pass
    }
}
