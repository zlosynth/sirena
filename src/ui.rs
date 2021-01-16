use std::collections::HashMap;

pub use gazpatcho::config as template;

pub type Data = HashMap<String, gazpatcho::model::Value>;
