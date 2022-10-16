#![feature(never_type, step_trait, trait_alias, associated_type_defaults)]

mod config;
mod files;
mod latex_generator;
mod latex_string;
mod signature;
mod tex_render;
mod utils;
mod verifier;
mod working_area;

pub mod time;

pub mod input {
    pub use crate::config::*;
    pub use crate::files::*;
    pub use crate::signature::*;
}

use std::fs;

use log::{error, info};

use crate::config::Config;
use crate::latex_generator::LatexGenerator;
use crate::verifier::{DefaultVerifier, Verifier};

pub fn generate_time_sheet(config: &Config) -> anyhow::Result<()> {
    let month_file = config.month_file();

    if let Err(errors) = DefaultVerifier.verify(month_file) {
        for error in errors {
            error!("{}", error);
        }

        return Err(anyhow::anyhow!("verification failed"));
    }

    let total_time = month_file.total_time().as_secs() / 60;
    info!("worked: {:02}:{:02}", total_time / 60, total_time % 60);

    info!("generating time sheet from month and global files");

    let mut generator = LatexGenerator::new(month_file.clone(), config.global_file().clone());

    if let Some(signature) = config.signature() {
        generator.signature(signature.clone());
    }

    if let Some(preserve_dir) = config.preserve_dir() {
        fs::create_dir_all(preserve_dir)?;
        generator.preserve_dir(preserve_dir);
    }

    let output = config.output();
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    generator.generate(output)?;

    Ok(())
}
