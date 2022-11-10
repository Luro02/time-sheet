use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::input::json_input::{GlobalFile, MonthFile};
use crate::input::toml_input::{self, DynamicEntry};
use crate::input::{Month, Signature};
use crate::latex_string::LatexString;
use crate::utils;
use crate::utils::PathExt;

pub struct Config {
    global_file: GlobalFile,
    signature: Option<Signature>,
    output: PathBuf,
    preserve_dir: Option<PathBuf>,
    month: Month,
}

pub struct ConfigBuilder {
    workspace: Option<PathBuf>,
    month_file: MonthFile,
    global_file: GlobalFile,
    signature: Option<Signature>,
    output: Option<PathBuf>,
    preserve_dir: Option<PathBuf>,
    dynamic_entries: HashMap<String, DynamicEntry>,
}

impl ConfigBuilder {
    fn new(month_file: MonthFile, global_file: GlobalFile) -> Self {
        Self {
            workspace: None,
            month_file,
            global_file,
            signature: None,
            output: None,
            preserve_dir: None,
            dynamic_entries: HashMap::new(),
        }
    }

    pub fn signature(&mut self, signature: Signature) -> &mut Self {
        self.signature = Some(signature);
        self
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

    pub fn dynamic_entries(&mut self, dynamic_entries: HashMap<String, DynamicEntry>) -> &mut Self {
        self.dynamic_entries = dynamic_entries;
        self
    }

    #[must_use]
    pub fn build(self) -> Config {
        let default_file_name = PathBuf::from(format!(
            "pdfs/{:04}-{:02}.pdf",
            self.month_file.year(),
            self.month_file.month()
        ));

        let output = self.output.unwrap_or_else(|| {
            if let Some(workspace) = &self.workspace {
                workspace.join(default_file_name)
            } else {
                default_file_name
            }
        });

        let expected_working_duration = self.global_file.expected_working_duration();

        let month = Month::new(
            self.month_file.month(),
            self.month_file.year(),
            self.month_file.transfer(),
            self.month_file.into_entries(),
            self.dynamic_entries,
            Some(expected_working_duration),
        );

        Config {
            month,
            global_file: self.global_file,
            signature: self.signature,
            output,
            preserve_dir: self.preserve_dir,
        }
    }
}

impl Config {
    pub fn try_from_files(
        month: impl AsRef<Path>,
        global: impl AsRef<Path>,
        department: impl Into<String>,
    ) -> anyhow::Result<ConfigBuilder> {
        let month = month.as_ref();
        let global = global.as_ref();
        let department = department.into();

        if month.has_extension("json") && global.has_extension("json") {
            Self::try_from_json_files(month, global)
        } else if month.has_extension("toml") && global.has_extension("toml") {
            Self::try_from_toml_files(month, global, department)
        } else {
            Err(anyhow::anyhow!(
                "Unknown file extension, month: `{}`, global: `{}` (expected `.json` or `.toml`)",
                month.display(),
                global.display()
            ))
        }
    }

    pub fn try_from_json_files(
        input: impl AsRef<Path>,
        global: impl AsRef<Path>,
    ) -> anyhow::Result<ConfigBuilder> {
        let month_file: MonthFile =
            serde_json::from_reader(BufReader::new(File::open(input.as_ref())?))
                .with_context(|| format!("failed to parse `{}`", input.as_ref().display()))?;
        let global_file: GlobalFile =
            serde_json::from_reader(BufReader::new(File::open(global.as_ref())?))
                .with_context(|| format!("failed to parse `{}`", global.as_ref().display()))?;

        Ok(ConfigBuilder::new(month_file, global_file))
    }

    pub fn try_from_toml_files(
        month: impl AsRef<Path>,
        global: impl AsRef<Path>,
        department: impl Into<String>,
    ) -> anyhow::Result<ConfigBuilder> {
        let month: toml_input::Month =
            serde_json::from_reader(BufReader::new(File::open(month.as_ref())?))
                .with_context(|| format!("failed to parse `{}`", month.as_ref().display()))?;
        let global: toml_input::Global =
            serde_json::from_reader(BufReader::new(File::open(global.as_ref())?))
                .with_context(|| format!("failed to parse `{}`", global.as_ref().display()))?;

        let department = department.into();
        let about = global.about();
        let contract = global
            .contract(&department)
            .ok_or_else(|| anyhow::anyhow!("no contract for department `{}`", department))?;
        let working_duration = Some(contract.working_time().clone());
        let dynamic_entries = month
            .dynamic_entries()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<HashMap<_, _>>();

        let month_file = MonthFile::from((working_duration, month));
        let global_file = GlobalFile::from((about.clone(), department, contract.clone()));

        let mut builder = ConfigBuilder::new(month_file, global_file);

        builder.dynamic_entries(dynamic_entries);

        Ok(builder)
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

    pub fn write_global_json(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        utils::write(path, serde_json::to_string_pretty(self.global_file())?)?;
        Ok(())
    }

    pub fn write_month_json(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        utils::write(path, serde_json::to_string_pretty(self.month())?)?;
        Ok(())
    }
}
