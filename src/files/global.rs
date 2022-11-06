use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::latex_string::LatexString;
use crate::time::WorkingDuration;
use crate::working_area::WorkingArea;
use crate::{toml_input, utils};

#[must_use]
fn global_schema() -> String {
    "https://raw.githubusercontent.com/kit-sdq/TimeSheetGenerator/master/examples/schemas/global.json".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlobalFile {
    #[serde(rename = "$schema", default = "global_schema")]
    schema: String,
    name: String,
    #[serde(rename = "staffId")]
    staff_id: usize,
    department: String,
    #[serde(rename = "workingTime")]
    working_time: WorkingDuration,
    // TODO: round to 2 decimal places
    #[serde(serialize_with = "utils::round_serialize")]
    wage: f32,
    #[serde(rename = "workingArea")]
    working_area: WorkingArea,
    #[serde(skip_serializing)]
    signature: Option<LatexString>,
}

impl From<(toml_input::About, String, toml_input::Contract)> for GlobalFile {
    fn from(
        (about, department, contract): (toml_input::About, String, toml_input::Contract),
    ) -> Self {
        Self {
            schema: global_schema(),
            name: about.name().to_string(),
            staff_id: about.staff_id(),
            department,
            working_time: contract.working_time().clone(),
            wage: contract.wage().unwrap_or(12.00),
            working_area: contract.area().clone(),
            signature: contract
                .bg_content()
                .map(|s| LatexString::from_str(s).unwrap()),
        }
    }
}

impl GlobalFile {
    #[must_use]
    pub fn signature(&self) -> Option<&str> {
        self.signature.as_deref()
    }
}
