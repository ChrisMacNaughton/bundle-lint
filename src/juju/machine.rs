use serde::{Deserialize, Serialize};

use failure::Error;

#[cfg(test)]
mod tests {
    use super::*;

    const MACHINE: &'static str = "constraints: virt-type=kvm";

    #[test]
    fn it_parses_a_machine() {
        let machine = Machine::parse(&MACHINE).unwrap();
        assert_eq!(machine.constraints.unwrap(), "virt-type=kvm");
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Machine {
    series: Option<String>,
    constraints: Option<String>,
}

impl Machine {
    pub fn parse(input: &str) -> Result<Machine, Error> {
        Ok(serde_yaml::from_str(&input)?)
    }
}
