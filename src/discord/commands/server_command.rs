use std::{collections::HashMap, default::Default};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerCommand {
    #[serde(skip_serializing)]
    on: String,
    #[serde(default = "Default::default")]
    run: Vec<String>,
    #[serde(default = "Default::default")]
    query: Vec<String>,
    #[serde(default = "Default::default")]
    set: HashMap<String, String>,
}
