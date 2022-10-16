use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use log::{debug, info};
use tempfile::TempDir;

use crate::files::{GlobalFile, MonthFile};
use crate::tex_render::TexRender;
use crate::utils;
use crate::utils::Resources;

#[must_use]
fn inject_fix(lines: impl Iterator<Item = impl AsRef<str>>) -> String {
    let mut result = String::new();
    for (number, line) in lines.enumerate() {
        if number == 1 {
            result.push_str("\\usepackage[T1]{fontenc}\n");
        }

        result.push_str(line.as_ref());
        result.push('\n');
    }

    result
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    /// Path to a signature that will then be automatically added.
    path: PathBuf,
    /// The width of the signature in cm, by default `3.8cm`.
    width: f32,
    date: String,
}

impl Signature {
    #[must_use]
    pub fn new(date: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            width: 3.8,
            date: date.into(),
        }
    }

    pub fn date(&self) -> &str {
        &self.date
    }
}

pub struct LatexGenerator {
    month: MonthFile,
    global: GlobalFile,
    preserve_dir: Option<PathBuf>,
    signature: Option<Signature>,
}

impl LatexGenerator {
    pub fn new(month: MonthFile, global: GlobalFile) -> Self {
        Self {
            month,
            global,
            preserve_dir: None,
            signature: None,
        }
    }

    pub fn signature(&mut self, signature: Signature) -> &mut Self {
        self.signature = Some(signature);
        self
    }

    pub fn preserve_dir(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.preserve_dir = Some(path.into());
        self
    }

    pub fn generate(self, outpath: impl AsRef<Path>) -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let month_path = temp_dir.path().join("month.json");
        let global_path = temp_dir.path().join("global.json");
        let jar =
            Resources::get("TimeSheetGenerator.jar").expect("jar should be embedded in the binary");
        let jar_path = temp_dir.path().join("TimeSheetGenerator.jar");

        debug!("temp_dir: {}", temp_dir.path().display());
        utils::write(&jar_path, jar.data)?;
        utils::write(&month_path, serde_json::to_string_pretty(&self.month)?)?;
        utils::write(&global_path, serde_json::to_string_pretty(&self.global)?)?;

        info!("Generating latex file");
        let latex_file = temp_dir.path().join("output.tex");
        let output = Command::new("java")
            .arg("-jar")
            .arg(&jar_path)
            .arg("--file")
            .args([&global_path, &month_path, &latex_file])
            .output()?;

        if !output.status.success() || !output.stdout.is_empty() {
            return Err(anyhow::anyhow!(String::from_utf8(output.stdout)?));
        }

        info!("Done");
        info!("Compiling latex file to pdf");

        // fix the latex file, so it does compile:
        let mut latex_file_content = inject_fix(utils::read_to_string(&latex_file)?.lines());

        if let Some(signature) = self.global.signature() {
            latex_file_content = latex_file_content.replace(
                "\\SetBgContents{K\\_IPD\\_AZDoku\\_01\\_01-20}",
                &format!("\\SetBgContents{{{}}}", signature),
            );
        }

        if let Some(signature) = &self.signature {
            let prefix = "\t%FOOTER\n\t\\par \\bigskip \\bigskip \\medskip\n";
            let new_path = signature.path.file_name().unwrap();
            latex_file_content = latex_file_content.replace(
                prefix,
                &format!(
                    "{}\t\\headentry{{\\hspace*{{\\fill}} {date}, \\includegraphics[width={width}cm]{{{signature}}} }} \\par \\medskip\n",
                    prefix,
                    date = signature.date(),
                    width = signature.width,
                    signature = &new_path.to_string_lossy(),
                ),
            );
        }

        if let Some(working_time) = self.month.working_time() {
            latex_file_content = latex_file_content
                .replace(
                    "\\centering 40:00",
                    &format!("\\centering {}", working_time),
                )
                .replace("& 40:00", &format!("& {}", working_time));
        }

        let logo_file = "Latex_Logo.pdf";
        let mut renderer = TexRender::from_bytes(latex_file_content.into_bytes())?;
        renderer.add_asset_from_bytes(
            //
            logo_file,
            Resources::get(logo_file).unwrap().data.as_ref(),
        )?;

        // add the signature image, if it is present
        if let Some(signature) = &self.signature {
            let new_path = signature.path.file_name().unwrap();
            renderer.add_asset_from_bytes(
                //
                new_path,
                &fs::read(&signature.path)?,
            )?;
        }

        if let Some(dir) = self.preserve_dir {
            renderer.preserve_dir(dir);
        }

        utils::write(outpath, renderer.render()?)?;

        info!("Done");

        Ok(())
    }
}
