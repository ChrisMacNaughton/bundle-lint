# BundleLint

BundleLint is a basic program to lint a bundle or deployed Juju model against a set of rules. These rules are easily writable in a yaml format.

## Usage

The most basic usage is:

    juju-lint $BUNDLE_PATH

If you would like to run `bundle-lint` against a running model, you can run:

    juju export-bundle -m $MODEL_NAME | juju-lint -