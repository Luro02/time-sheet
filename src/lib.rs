#![feature(never_type, step_trait, trait_alias, associated_type_defaults)]

mod latex_generator;
mod latex_string;
mod tex_render;
mod utils;
mod verifier;

pub mod input;
pub mod time;

use std::fs;

use log::{error, info};

use crate::input::Config;
use crate::latex_generator::LatexGenerator;
use crate::verifier::{DefaultVerifier, Verifier};

pub fn generate_time_sheet(config: &Config) -> anyhow::Result<()> {
    if let Err(errors) = DefaultVerifier.verify(config) {
        for error in errors {
            error!("{}", error);
        }

        return Err(anyhow::anyhow!("verification failed"));
    }

    let total_time = config.month().total_working_time().as_secs() / 60;
    info!("worked: {:02}:{:02}", total_time / 60, total_time % 60);

    info!("generating time sheet from month and global files");

    let generator = LatexGenerator::new(config);

    let output = config.output();
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    generator.generate(output)?;

    Ok(())
}
