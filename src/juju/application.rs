use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use failure::Error;

#[cfg(test)]
mod tests {
    use super::*;

    const APPLICATION: &'static str = r#"charm: cs:ubuntu-12
num_units: 1
to:
- "0""#;
    #[test]
    fn it_parses_an_application() {
        let application = Application::parse(&APPLICATION).unwrap();
        assert_eq!(application.charm, "cs:ubuntu-12");
    }
}

fn zero() -> usize {
    0
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Application {
    charm: String,
    #[serde(default = "zero")]
    num_units: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    to: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    options: BTreeMap<String, String>,
}

impl Application {
    pub fn parse(input: &str) -> Result<Application, Error> {
        Ok(serde_yaml::from_str(&input)?)
    }

    pub fn option(&self, option: &str) -> Option<&String> {
        self.options.get(option)
    }
}
