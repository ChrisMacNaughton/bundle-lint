# BundleLint

BundleLint is a basic program to lint a bundle or deployed Juju model against a set of rules. These rules are easily writable in a yaml format.

## Usage

The most basic usage is:

    juju-lint model $JUJU_MODEL

Additionally, a `debug` flag (`-d`) can be passed to the base command to add more verbose output:

    juju-lint -d model $JUJU_MODEL

To run juju-lint against a bundle rather than a model, call it like:

    juju-lint bundle $BUNDLE_PATH

