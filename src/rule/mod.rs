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
    fn it_parses_a_rule() {
        let yaml_s = r#"- charm_name: neutron-api
  config:
    -
      config_name: enable-dvr
      config_value: 'True'
      requires:
        neutron-openvswitch:
          name: bridge-mappings
"#;
        let rules: Vec<Rule> = serde_yaml::from_str(yaml_s).unwrap();
        assert_eq!(rules[0].charm_name, "neutron-api");
    }

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
            config: vec![Config {
                config_name: "use-cool-thing".to_string(),
                config_value: "True".to_string(),
                requires: HashMap::new(),
                forbids,
            }],
            relations: vec![Relation::default()],
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
            config: vec![Config {
                config_name: "use-cool-thing".to_string(),
                config_value: "True".to_string(),
                requires: HashMap::new(),
                forbids,
            }],
            relations: vec![Relation::default()],
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
            config: vec![Config {
                config_name: "use-cool-thing".to_string(),
                config_value: "True".to_string(),
                requires: HashMap::new(),
                forbids,
            }],
            relations: vec![Relation::default()],
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
            config: vec![Config {
                config_name: "use-cool-thing".to_string(),
                config_value: "True".to_string(),
                requires,
                forbids: HashMap::new(),
            }],
            relations: vec![Relation::default()],
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
    #[serde(default)]
    pub config: Vec<Config>,
    #[serde(default)]
    pub relations: Vec<Relation>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum VerificationResult {
    Pass,
    Fail { reason: String },
}

impl Rule {
    pub fn verify(&self, bundle: &Bundle) -> VerificationResult {
        if let Some(application) = bundle.application(&self.charm_name) {
            for config in &self.config {
                if let VerificationResult::Fail { reason: f } = config.verify(&application, &bundle)
                {
                    return VerificationResult::Fail { reason: f };
                }
            }
            for relation in &self.relations {
                if let VerificationResult::Fail { reason: f } = relation.verify(&application, &bundle)
                {
                    return VerificationResult::Fail { reason: f };
                }
            }
        }
        VerificationResult::Pass
    }
}
