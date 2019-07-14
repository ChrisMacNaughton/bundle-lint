use crate::juju::Bundle;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::rule::VerificationResult;

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Relation {
    pub requires: Option<String>,
    pub forbids: Option<String>,
}
