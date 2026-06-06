use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub language: String,
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            theme: "dark".to_string(),
        }
    }
}
