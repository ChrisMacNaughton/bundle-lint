use failure::Error;
use log::debug;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::bundle::Bundle;

pub struct Model;

impl Model {
    pub fn export_bundle(model: &str) -> Result<Bundle, Error> {
        let mut cmd = Command::new("juju");
        cmd.arg("export-bundle").arg("--model").arg(model);
        debug!("About to run {:?}", cmd);
        let output = cmd.output()?;
        debug!("Got output from juju: {:?}", output);
        Bundle::load(&String::from_utf8_lossy(&output.stdout))
    }

    pub fn load_bundle(path: PathBuf) -> Result<Bundle, Error> {
        let path = if path == PathBuf::from("-") {
            PathBuf::from("/dev/stdin")
        } else {
            path
        };
        Bundle::load(&fs::read_to_string(path)?)
    }
}
