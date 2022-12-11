use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::Context;
use indexmap::IndexMap;

use crate::input::json_input::{Entry, GlobalFile};
use crate::input::toml_input;
use crate::input::{Month, Signature};
use crate::latex_string::LatexString;
use crate::utils;

pub struct Config {
    global_file: GlobalFile,
    signature: Option<Signature>,
    output: PathBuf,
    preserve_dir: Option<PathBuf>,
    month: Month,
    latex_mk_path: Option<PathBuf>,
}

pub struct ConfigBuilder {
    workspace: Option<PathBuf>,
    global: toml_input::Global,
    month: toml_input::Month,
    output: Option<PathBuf>,
    preserve_dir: Option<PathBuf>,
    department: String,
}

impl ConfigBuilder {
    fn new(
        global: toml_input::Global,
        month: toml_input::Month,
        department: String,
    ) -> anyhow::Result<Self> {
        global
            .contract(&department)
            .ok_or_else(|| anyhow::anyhow!("no contract for department `{}`", department))?;

        Ok(Self {
            workspace: None,
            output: None,
            preserve_dir: None,
            global,
            month,
            department,
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

    pub fn workspace(&mut self, workspace: impl Into<PathBuf>) -> &mut Self {
        self.workspace = Some(workspace.into());
        self
    }

    #[must_use]
    pub fn build(self) -> Config {
        let default_file_name = PathBuf::from(format!(
            "pdfs/{:04}-{:02}.pdf",
            self.month.general().year(),
            self.month.general().month()
        ));

        let output = self.output.unwrap_or_else(|| {
            if let Some(workspace) = &self.workspace {
                workspace.join(default_file_name)
            } else {
                default_file_name
            }
        });

        let contract = self.global.contract(&self.department).unwrap();

        let mut month = Month::new(
            self.month.general().month(),
            self.month.general().year(),
            self.month.transfer().unwrap_or_default(),
            self.month
                .entries()
                .map(|(key, entry)| Entry::from((key.clone(), entry.clone())))
                .collect(),
            self.month
                .dynamic_entries()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<IndexMap<_, _>>(),
            Some(contract.expected_working_duration()),
            self.month
                .absences()
                .map(|(k, v)| (k, v.clone()))
                .collect::<Vec<_>>(),
        );

        for entry in self
            .global
            .repeating_in_month(self.month.general().year(), self.month.general().month())
            .flat_map(|(key, entry)| {
                entry
                    .into_iter()
                    .map(move |e| Entry::from((key.clone(), e)))
            })
        {
            month.add_entry_if_possible(entry);
        }

        if let Some(holiday) = self.month.holiday() {
            month.schedule_holiday(holiday);
        }

        Config {
            month,
            global_file: GlobalFile::from((
                self.global.about().clone(),
                self.department,
                contract.clone(),
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
        department: impl Into<String>,
    ) -> anyhow::Result<ConfigBuilder> {
        ConfigBuilder::new(global, month, department.into())
    }

    pub fn try_from_toml_files(
        month: impl AsRef<Path>,
        global: impl AsRef<Path>,
        department: impl Into<String>,
    ) -> anyhow::Result<ConfigBuilder> {
        let month: toml_input::Month = utils::toml_from_reader(File::open(month.as_ref())?)
            .with_context(|| format!("failed to parse `{}`", month.as_ref().display()))?;
        let global: toml_input::Global = utils::toml_from_reader(File::open(global.as_ref())?)
            .with_context(|| format!("failed to parse `{}`", global.as_ref().display()))?;

        Self::try_from_toml(month, global, department)
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
