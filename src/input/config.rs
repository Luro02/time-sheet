use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::Context;
use indexmap::IndexMap;
use serde::de::DeserializeOwned;

use crate::input::json_input::{self, Entry, GlobalFile, MonthFile};
use crate::input::toml_input::{self, Absence, DynamicEntry, Holiday};
use crate::input::{Month, Signature};
use crate::latex_string::LatexString;
use crate::time::Date;
use crate::utils;
use crate::utils::PathExt;

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
    month_file: MonthFile,
    global_file: GlobalFile,
    signature: Option<Signature>,
    output: Option<PathBuf>,
    preserve_dir: Option<PathBuf>,
    dynamic_entries: IndexMap<String, DynamicEntry>,
    absences: Vec<(Date, Absence)>,
    holiday: Option<Holiday>,
    repeating: Vec<Entry>,
    latex_mk_path: Option<PathBuf>,
}

fn toml_from_reader<R, T>(reader: R) -> anyhow::Result<T>
where
    R: Read,
    T: DeserializeOwned,
{
    let mut reader = BufReader::new(reader);
    let mut date = String::with_capacity(1024 * 1024);
    reader.read_to_string(&mut date)?;
    Ok(toml::from_str(&date)?)
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
            dynamic_entries: IndexMap::new(),
            absences: Vec::new(),
            holiday: None,
            repeating: Vec::new(),
            latex_mk_path: None,
        }
    }

    pub fn holiday(&mut self, holiday: Option<Holiday>) -> &mut Self {
        self.holiday = holiday;
        self
    }

    pub fn absences(&mut self, absences: Vec<(Date, Absence)>) -> &mut Self {
        self.absences = absences;
        self
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

    pub fn latex_mk_path(&mut self, path: PathBuf) -> &mut Self {
        self.latex_mk_path = Some(path);
        self
    }

    pub fn dynamic_entries(
        &mut self,
        dynamic_entries: IndexMap<String, DynamicEntry>,
    ) -> &mut Self {
        self.dynamic_entries = dynamic_entries;
        self
    }

    pub fn repeating(&mut self, repeating: Vec<Entry>) -> &mut Self {
        self.repeating = repeating;
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

        let mut month = Month::new(
            self.month_file.month(),
            self.month_file.year(),
            self.month_file.transfer(),
            self.month_file.into_entries(),
            self.dynamic_entries,
            Some(expected_working_duration),
            self.absences,
        );

        for entry in self.repeating {
            month.add_entry_if_possible(entry);
        }

        if let Some(holiday) = self.holiday {
            month.schedule_holiday(holiday);
        }

        Config {
            month,
            global_file: self.global_file,
            signature: self.signature,
            output,
            preserve_dir: self.preserve_dir,
            latex_mk_path: self.latex_mk_path,
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

    pub fn try_from_toml(
        month: toml_input::Month,
        global: toml_input::Global,
        department: impl Into<String>,
    ) -> anyhow::Result<ConfigBuilder> {
        let department = department.into();
        let about = global.about();
        let latex_mk_path = global.latex_mk_path();
        let contract = global
            .contract(&department)
            .ok_or_else(|| anyhow::anyhow!("no contract for department `{}`", department))?;
        let dynamic_entries = month
            .dynamic_entries()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<IndexMap<_, _>>();
        let absences = month
            .absences()
            .map(|(k, v)| (k, v.clone()))
            .collect::<Vec<_>>();

        let signature = {
            if let (Some(month_signature), Some(global_signature)) =
                (month.general().signature(), global.about().signature())
            {
                Some(Signature::from((
                    month_signature.date(),
                    global_signature.clone(),
                )))
            } else {
                None
            }
        };

        let holiday = month.holiday().cloned();
        let repeating = global
            .repeating_in_month(month.general().year(), month.general().month())
            .flat_map(|(key, entry)| {
                entry
                    .into_iter()
                    .map(move |e| json_input::Entry::from((key.clone(), e)))
            })
            .collect::<Vec<_>>();

        let month_file = MonthFile::from(month);
        let global_file = GlobalFile::from((about.clone(), department, contract.clone()));

        let mut builder = ConfigBuilder::new(month_file, global_file);

        builder
            .dynamic_entries(dynamic_entries)
            .absences(absences)
            .holiday(holiday)
            .repeating(repeating);

        if let Some(signature) = signature {
            builder.signature(signature);
        }

        if let Some(path) = latex_mk_path {
            builder.latex_mk_path(path.to_path_buf());
        }

        Ok(builder)
    }

    pub fn try_from_toml_files(
        month: impl AsRef<Path>,
        global: impl AsRef<Path>,
        department: impl Into<String>,
    ) -> anyhow::Result<ConfigBuilder> {
        let month: toml_input::Month = toml_from_reader(File::open(month.as_ref())?)
            .with_context(|| format!("failed to parse `{}`", month.as_ref().display()))?;
        let global: toml_input::Global = toml_from_reader(File::open(global.as_ref())?)
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
