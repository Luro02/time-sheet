#![feature(
    never_type,
    step_trait,
    trait_alias,
    associated_type_defaults,
    const_mut_refs
)]

mod latex_generator;
mod latex_string;
mod tex_render;
mod utils;

pub mod input;
pub mod time;

use std::fs;

use log::info;

use crate::input::Config;
use crate::latex_generator::LatexGenerator;

pub fn generate_time_sheet(config: &Config) -> anyhow::Result<()> {
    let total_time = config.month().total_working_time();
    info!("worked: {}", total_time);

    info!("generating time sheet from month and global files");

    let generator = LatexGenerator::new(config);

    let output = config.output();
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    generator.generate(output)?;

    Ok(())
}
