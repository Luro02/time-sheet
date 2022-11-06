use std::path::PathBuf;

use serde::Deserialize;

use crate::input::Signature;
use crate::time::Date;

#[derive(Debug, Clone, Deserialize)]
pub struct SignatureInput {
    path: PathBuf,
    width: Option<f32>,
}

impl From<(Date, SignatureInput)> for Signature {
    fn from((date, signature): (Date, SignatureInput)) -> Self {
        if let Some(width) = signature.width {
            Signature::new_with_width(date, signature.path, width)
        } else {
            Signature::new(date, signature.path)
        }
    }
}
