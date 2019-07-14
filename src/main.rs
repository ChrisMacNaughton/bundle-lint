extern crate bundle_lint;

use failure::Error;
use std::path::PathBuf;

use log::{debug, info, Level};

use structopt::StructOpt;

use bundle_lint::juju;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "juju-lint",
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
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "bundle")]
    /// A bundle that will be deployed
    Bundle(Bundle),
    #[structopt(name = "model")]
    /// A running Juju model
    Model(Model),
}
#[derive(StructOpt, Debug)]
struct Bundle {
    /// Bundle to lint
    #[structopt(name = "bundle")]
    bundle_path: PathBuf,
    // /// Bundles to overlay on the primary bundle, applied in order
    // #[structopt(long = "overlay", multiple = true)]
    // overlays: Vec<PathBuf>,
}
#[derive(StructOpt, Debug)]
struct Model {
    #[structopt(help = "Name of the model to lint")]
    name: String,
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
    let bundle = match options.command {
        Command::Bundle(bundle) => juju::Model::load_bundle(&bundle.bundle_path)?,
        Command::Model(model) => juju::Model::export_bundle(&model.name)?,
    };
    info!("Loaded bundle: {:#?}", bundle);
    let rules = bundle_lint::import_rules(&options.config_repo)?;
    info!("Loaded rules: {:?}", rules);
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
