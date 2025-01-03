#![feature(
    never_type,
    step_trait,
    trait_alias,
    associated_type_defaults,
    const_swap
)]

mod latex_generator;
mod latex_string;
mod tex_render;
mod utils;

pub mod input;
pub mod time;

use log::{info, warn};

use crate::input::Config;
use crate::latex_generator::LatexGenerator;

pub fn generate_time_sheet(config: &Config) -> anyhow::Result<()> {
    let total_time = config.month().total_working_time();
    info!("worked: {}", total_time);

    for action in config.month().actions_that_overflow() {
        warn!(
            "action \"{}\" has too much text and will not fit into the table",
            action
        );
    }

    info!("generating time sheet from month and global files");

    let generator = LatexGenerator::new(config);

    let output = config.output();
    if let Some(parent) = output.parent() {
        utils::create_dir_all(parent)?;
    }

    generator.generate(output)?;

    Ok(())
}
