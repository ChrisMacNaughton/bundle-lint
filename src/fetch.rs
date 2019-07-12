use crate::rule::Rule;
use std::env::temp_dir;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use failure::Error;
use log::trace;

pub fn load(path: &str) -> Result<PathBuf, Error>{
    trace!("About to download {}", path);
    let mut local_path = temp_dir();
    local_path.push("juju_lint_base");

    if local_path.exists() {
        fs::remove_dir_all(&local_path)?;
    }
    let mut cmd = Command::new("git");
    cmd.arg("clone");
    cmd.arg("--depth=1");
    cmd.arg(path);
    cmd.arg(&local_path);
    cmd.output()?;
    Ok(local_path)
}

pub fn import(path: &PathBuf) -> Result<Vec<Rule>, Error> {
    let mut rules = vec![];
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension().map(|a| a.to_string_lossy()) {
                if extension == "yaml" {
                    let mut local_rules: Vec<Rule> = serde_yaml::from_str(&fs::read_to_string(path)?)?;
                    rules.append(&mut local_rules);
                }
            }
        }
    }
    Ok(rules)
}