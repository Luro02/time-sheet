use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::input::json_input::{Entry, GlobalFile};
use crate::input::scheduler::SchedulerOptions;
use crate::input::toml_input::{self, Contract, Mail};
use crate::input::{Month, Signature};
use crate::latex_string::LatexString;
use crate::utils;
use crate::working_duration;

#[derive(Debug, Clone)]
pub struct Config {
    mail: Option<Mail>,
    global_file: GlobalFile,
    signature: Option<Signature>,
    output: PathBuf,
    preserve_dir: Option<PathBuf>,
    month: Month,
    latex_mk_path: Option<PathBuf>,
}

pub struct ConfigBuilder {
    contract: Contract,
    global: toml_input::Global,
    month: toml_input::Month,
    output: Option<PathBuf>,
    preserve_dir: Option<PathBuf>,
}

impl ConfigBuilder {
    fn new(global: toml_input::Global, month: toml_input::Month) -> anyhow::Result<Self> {
        let department = month.general().department();
        let contract = global
            .contract(department)
            .ok_or_else(|| anyhow::anyhow!("no contract for department `{}`", department))?
            .clone();

        Ok(Self {
            output: None,
            preserve_dir: None,
            global,
            month,
            contract,
        })
    }

    pub fn output(&mut self, output: impl Into<PathBuf>) -> &mut Self {
        self.output = Some(output.into());
        self
    }

    pub fn preserve_dir(&mut self, preserve_dir: impl Into<PathBuf>) -> &mut Self {
        self.preserve_dir = Some(preserve_dir.into());
        self
    }

    #[must_use]
    pub fn build(self) -> Config {
        let default_file_name = PathBuf::from(self.global.resolve_output(&self.month));

        let output = {
            if let Some(output) = self.output {
                if output.is_dir() {
                    output.join(default_file_name)
                } else {
                    output
                }
            } else {
                default_file_name
            }
        };

        let mut month = Month::new(
            self.month.general().month(),
            self.month.general().year(),
            self.month.transfer().unwrap_or_default(),
            self.month.entries().map(Entry::from).collect(),
            self.month.dynamic_entries().cloned().collect(),
            Some(self.contract.expected_working_duration()),
            self.month.absences().collect::<Vec<_>>(),
            SchedulerOptions {
                daily_limit: working_duration!(06:00),
                strategy: self.month.general().strategy(),
                ..Default::default()
            },
        );

        for entry in self
            .global
            .repeating_in_month(
                self.month.general().year(),
                self.month.general().month(),
                |date| date.is_workday(),
                self.contract.department(),
            )
            .map(Entry::from)
        {
            month.add_entry_if_possible(entry);
        }

        if let Some(holiday) = self.month.holiday() {
            month.schedule_holiday(holiday);
        }

        Config {
            month,
            mail: self.global.mail().cloned(),
            global_file: GlobalFile::from((
                self.global.about().clone(),
                self.contract.department().to_string(),
                self.contract,
            )),
            signature: {
                if let (Some(month_signature), Some(global_signature)) = (
                    self.month.general().signature(),
                    self.global.about().signature(),
                ) {
                    Some(Signature::from((
                        month_signature.date(),
                        global_signature.clone(),
                    )))
                } else {
                    None
                }
            },
            output,
            preserve_dir: self.preserve_dir,
            latex_mk_path: self.global.latex_mk_path().map(|v| v.to_path_buf()),
        }
    }
}

impl Config {
    pub fn try_from_toml(
        month: toml_input::Month,
        global: toml_input::Global,
    ) -> anyhow::Result<ConfigBuilder> {
        ConfigBuilder::new(global, month)
    }

    pub fn try_from_toml_files(
        month: impl AsRef<Path>,
        global: impl AsRef<Path>,
    ) -> anyhow::Result<ConfigBuilder> {
        let month: toml_input::Month = utils::toml_from_reader(File::open(month.as_ref())?)
            .with_context(|| format!("failed to parse `{}`", month.as_ref().display()))?;
        let global: toml_input::Global = utils::toml_from_reader(File::open(global.as_ref())?)
            .with_context(|| format!("failed to parse `{}`", global.as_ref().display()))?;

        Self::try_from_toml(month, global)
    }

    pub fn output(&self) -> &Path {
        &self.output
    }

    fn global_file(&self) -> &GlobalFile {
        &self.global_file
    }

    pub fn signature(&self) -> Option<&Signature> {
        self.signature.as_ref()
    }

    pub fn preserve_dir(&self) -> Option<&Path> {
        self.preserve_dir.as_deref()
    }

    pub fn bg_content(&self) -> Option<&LatexString> {
        self.global_file().bg_content()
    }

    pub fn month(&self) -> &Month {
        &self.month
    }

    pub fn latex_mk_path(&self) -> Option<&Path> {
        self.latex_mk_path.as_deref()
    }

    pub fn mail(&self) -> Option<&Mail> {
        self.mail.as_ref()
    }

    pub fn write_global_json(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        utils::write(path, serde_json::to_string_pretty(self.global_file())?)?;
        Ok(())
    }

    pub fn write_month_json(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        utils::write(path, self.to_month_json()?)?;
        Ok(())
    }

    pub fn to_month_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self.month())
    }
}
