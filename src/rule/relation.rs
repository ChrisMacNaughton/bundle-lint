use crate::juju::{Application, Bundle};
use failure::Error;
use serde::{Deserialize, Serialize};

use crate::rule::VerificationResult;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_a_relation() {
        let rel_yaml = r#"---
config:
  name: enable-dvr
  value: True
requires:
- - 'neutron-api:neutron-plugin-api'
  - 'neutron-openvswitch:neutron-plugin-api'"#;
        let relation = Relation::parse(&rel_yaml).unwrap();
        assert_eq!(relation.requires[0][0], "neutron-api:neutron-plugin-api");
    }
}

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Relation {
    pub config: Option<ConfigDetail>,
    #[serde(default)]
    pub requires: Vec<[String; 2]>,
    #[serde(default)]
    pub forbids: Vec<[String; 2]>,
}

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ConfigDetail {
    pub name: String,
    pub value: String,
}

impl Relation {
    pub fn parse(input: &str) -> Result<Relation, Error> {
        Ok(serde_yaml::from_str(&input)?)
    }
    pub fn verify(&self, application: &Application, bundle: &Bundle) -> VerificationResult {
        if let Some(config) = &self.config {
            if let Some(value) = application.option(&config.name) {
                if *value != config.value {
                    return VerificationResult::Pass;
                }
            }
        }
        if let VerificationResult::Fail { reason: f } = self.verify_required(&bundle) {
            return VerificationResult::Fail { reason: f };
        }
        if let VerificationResult::Fail { reason: f } = self.verify_forbids(&bundle) {
            return VerificationResult::Fail { reason: f };
        }
        VerificationResult::Pass
    }

    fn verify_required(&self, bundle: &Bundle) -> VerificationResult {
        for relation in &self.requires {
            if !bundle
                .relations
                .iter()
                .any(|b_relation| b_relation.iter().all(|k| relation.contains(k)))
            {
                return VerificationResult::Fail {
                    reason: "Missing a required relation".into(),
                };
            }
        }
        VerificationResult::Pass
    }

    fn verify_forbids(&self, bundle: &Bundle) -> VerificationResult {
        for relation in &self.forbids {
            if bundle
                .relations
                .iter()
                .any(|b_relation| b_relation.iter().all(|k| relation.contains(k)))
            {
                return VerificationResult::Fail {
                    reason: "Missing a required relation".into(),
                };
            }
        }
        VerificationResult::Pass
    }
}
