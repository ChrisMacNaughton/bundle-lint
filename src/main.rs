extern crate bundle_lint;

use failure::Error;
use std::path::PathBuf;
use std::process;

use log::{debug, Level};

use structopt::StructOpt;

use bundle_lint::juju;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "bundle-lint",
    about = "A program to lint Juju models and bundles."
)]
struct Opt {
    /// Activate debug mode
    #[structopt(short = "d", long = "debug")]
    debug: bool,
    /// The path to the confiruation repository.
    ///
    /// This cna be a local file path or a path to a repository on a hosted
    /// git server. It is also possible to use gh:namespace/repo.git as a
    /// shorthand to a github repository.
    #[structopt(
        name = "config_repo",
        short = "c",
        long = "config_repo",
        default_value = "gh:ChrisMacNaughton/bundlelint-rules"
    )]
    config_repo: String,
    /// Bundle to lint
    #[structopt(name = "bundle")]
    bundle_path: PathBuf,
}

fn main() -> Result<(), Error> {
    let options = Opt::from_args();
    let level = if options.debug {
        Level::Trace
    } else {
        Level::Warn
    };
    simple_logger::init_with_level(level).expect("Couldn't initialize logger");
    debug!("Running with {:?}", options);
    let bundle = match juju::Model::load_bundle(options.bundle_path.clone()) {
        Ok(b) => b,
        Err(e) => {
            println!(
                "Failed to load the bundle ({}): {}",
                options.bundle_path.display(),
                e
            );
            process::exit(1);
        }
    };
    debug!("Loaded bundle: {:#?}", bundle);
    let rules = match bundle_lint::import_rules(&options.config_repo) {
        Ok(b) => b,
        Err(e) => {
            println!(
                "Failed to load the configuration at {}: {}",
                options.config_repo, e
            );
            process::exit(1);
        }
    };
    debug!("Loaded rules: {:#?}", rules);
    let mut passing = true;
    for rule in rules {
        match rule.verify(&bundle) {
            bundle_lint::VerificationResult::Pass => {}
            bundle_lint::VerificationResult::Fail { reason } => {
                println!("{} rule failed: {}", rule.charm_name, reason);
                passing = false;
            }
        }
    }
    if passing {
        println!("Passed all configured lints");
        Ok(())
    } else {
        Err(bundle_lint::JujuLintError::LintFailure.into())
    }
}
