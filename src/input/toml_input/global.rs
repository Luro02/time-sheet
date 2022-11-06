use std::collections::HashMap;

use serde::Deserialize;

use crate::input::toml_input::{About, Contract};

#[derive(Debug, Clone, Deserialize)]
pub struct Global {
    about: About,
    contract: HashMap<String, Contract>,
}

impl Global {
    #[must_use]
    pub fn about(&self) -> &About {
        &self.about
    }

    #[must_use]
    pub fn contract(&self, department: &str) -> Option<&Contract> {
        self.contract.get(department)
    }
}
