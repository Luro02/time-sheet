use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::input::{GlobalFile, MonthFile, Signature};

pub struct Config {
    month_file: MonthFile,
    global_file: GlobalFile,
    signature: Option<Signature>,
    output: PathBuf,
    preserve_dir: Option<PathBuf>,
}

pub struct ConfigBuilder {
    workspace: Option<PathBuf>,
    month_file: MonthFile,
    global_file: GlobalFile,
    signature: Option<Signature>,
    output: Option<PathBuf>,
    preserve_dir: Option<PathBuf>,
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

        Config {
            month_file: self.month_file,
            global_file: self.global_file,
            signature: self.signature,
            output,
            preserve_dir: self.preserve_dir,
        }
    }
}

impl Config {
    pub fn try_from_files(
        input: impl AsRef<Path>,
        global: impl AsRef<Path>,
    ) -> anyhow::Result<ConfigBuilder> {
        let month_file: MonthFile = serde_json::from_reader(BufReader::new(File::open(input)?))?;
        let global_file: GlobalFile = serde_json::from_reader(BufReader::new(File::open(global)?))?;

        Ok(ConfigBuilder::new(month_file, global_file))
    }

    pub fn output(&self) -> &Path {
        &self.output
    }

    pub fn month_file(&self) -> &MonthFile {
        &self.month_file
    }

    pub fn global_file(&self) -> &GlobalFile {
        &self.global_file
    }

    pub fn signature(&self) -> Option<&Signature> {
        self.signature.as_ref()
    }

    pub fn preserve_dir(&self) -> Option<&Path> {
        self.preserve_dir.as_deref()
    }
}
