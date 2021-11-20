use serde::{Deserialize, Serialize};
use std::{collections::HashMap, default::Default};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerCommand {
    #[serde(skip_serializing)]
    pub on: String,
    #[serde(default = "Default::default")]
    pub run: Vec<String>,
    #[serde(default = "Default::default")]
    pub query: Vec<String>,
    #[serde(default = "Default::default")]
    pub set: HashMap<String, String>,
}
