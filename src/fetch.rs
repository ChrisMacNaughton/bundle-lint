use crate::rule::Rule;
use std::fs;
use std::path::PathBuf;

use failure::Error;
use git2::Repository;
use log::{debug, trace};
use xdg;

pub fn load(path: &str) -> Result<PathBuf, Error> {
    let p = PathBuf::from(path);
    if p.exists() {
        debug!("Using local Rule path: {}", path);
        return Ok(p);
    }
    trace!("About to download {}", path);
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bundle-lint")?;
    let local_path = xdg_dirs.place_cache_file("juju_lint_base")?;
    debug!(
        "Trying to store rule configuration in {}",
        local_path.display()
    );
    if local_path.exists() {
        fs::remove_dir_all(&local_path)?;
    }
    debug!("Cloning {} to {}", path, local_path.display());
    Repository::clone(path, &local_path)?;
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
                    let rule_yaml = fs::read_to_string(path)?;
                    debug!("Loding rules from:\n{}", rule_yaml);
                    let mut local_rules: Vec<Rule> = serde_yaml::from_str(&rule_yaml)?;
                    rules.append(&mut local_rules);
                }
            }
        }
    }
    Ok(rules)
}
