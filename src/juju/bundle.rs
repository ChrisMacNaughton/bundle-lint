use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use failure::Error;

use super::{Application, Machine};

#[cfg(test)]
mod tests {
    use super::*;

    const BUNDLE: &'static str = include_str!("../../tests/test_bundle.yaml");

    #[test]
    fn it_parses_a_bundle() {
        let bundle = Bundle::load(BUNDLE).unwrap();
        assert_eq!(bundle.series.unwrap(), "bionic");
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Bundle {
    pub series: Option<String>,
    pub applications: HashMap<String, Application>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub machines: HashMap<String, Machine>,
}

impl Bundle {
    pub fn load(input_yaml: &str) -> Result<Bundle, Error> {
        Ok(serde_yaml::from_str(&input_yaml)?)
    }

    pub fn application(&self, application_name: &str) -> Option<&Application> {
        self.applications.get(application_name)
    }
}
