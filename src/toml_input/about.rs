use serde::Deserialize;

use crate::input::toml_input::SignatureInput;

#[derive(Debug, Clone, Deserialize)]
pub struct About {
    name: String,
    staff_id: usize,
    signature: Option<SignatureInput>,
}

impl About {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn staff_id(&self) -> usize {
        self.staff_id
    }

    #[must_use]
    pub fn signature(&self) -> Option<&SignatureInput> {
        self.signature.as_ref()
    }
}
