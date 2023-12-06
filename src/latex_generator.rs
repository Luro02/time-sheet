use std::fs;
use std::path::Path;
use std::process::Command;

use log::{debug, info};
use tempfile::TempDir;

use crate::input::Config;
use crate::tex_render::TexRender;
use crate::utils::{self, Resources};

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

pub struct LatexGenerator<'a> {
    config: &'a Config,
}

impl<'a> LatexGenerator<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn generate(self, outpath: impl AsRef<Path>) -> anyhow::Result<()> {
        let temp_dir = {
            if let Some(dir) = self.config.preserve_dir() {
                dir.to_path_buf()
            } else {
                TempDir::new()?.into_path()
            }
        };

        // ensure the temp_dir exists:
        fs::create_dir_all(&temp_dir)?;

        let month_path = temp_dir.join("month.json");
        let global_path = temp_dir.join("global.json");
        let jar =
            Resources::get("TimeSheetGenerator.jar").expect("jar should be embedded in the binary");
        let jar_path = temp_dir.join("TimeSheetGenerator.jar");

        debug!("temp_dir: {}", temp_dir.display());
        utils::write(&jar_path, jar.data)?;
        self.config.write_month_json(&month_path)?;
        self.config.write_global_json(&global_path)?;

        info!("Generating latex file");
        let latex_file = temp_dir.canonicalize()?.join("output.tex");
        debug!("latex_file: {}", latex_file.display());
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

        if let Some(signature) = self.config.bg_content() {
            latex_file_content = latex_file_content.replace(
                "\\SetBgContents{K\\_IPD\\_AZDoku\\_01\\_01-20}",
                &format!("\\SetBgContents{{{}}}", signature),
            );
        }

        if let Some(signature) = self.config.signature() {
            let prefix = "\t%FOOTER\n\t\\par \\bigskip \\bigskip \\medskip\n";
            let new_path = signature.path().file_name().unwrap();
            latex_file_content = latex_file_content.replace(
                prefix,
                &format!(
                    "{}\t\\headentry{{\\hspace*{{\\fill}} {date}, \\includegraphics[width={width:.2}cm]{{{signature}}} }} \\par \\medskip\n",
                    prefix,
                    date = signature.date().formatted("{day}.{month}.{year}"),
                    width = signature.width(),
                    signature = &new_path.to_string_lossy(),
                ),
            );
        }

        let working_duration = self.config.month().real_expected_working_duration();
        latex_file_content = latex_file_content
            .replace(
                "\\centering 40:00",
                &format!("\\centering {}", working_duration),
            )
            .replace("& 40:00", &format!("& {}", working_duration));

        let logo_file = "Latex_Logo.pdf";
        let mut renderer = TexRender::from_bytes(latex_file_content.into_bytes())?;
        renderer.add_asset_from_bytes(
            //
            logo_file,
            Resources::get(logo_file).unwrap().data.as_ref(),
        )?;

        if let Some(path) = self.config.latex_mk_path() {
            renderer.latex_mk_path(path);
        }

        // add the signature image, if it is present
        if let Some(signature) = self.config.signature() {
            let new_path = signature.path().file_name().unwrap();
            renderer.add_asset_from_bytes(
                //
                new_path,
                &fs::read(signature.path())?,
            )?;
        }

        if let Some(dir) = self.config.preserve_dir() {
            renderer.preserve_dir(dir);
        }

        utils::write(outpath, renderer.render()?)?;

        info!("Done");

        Ok(())
    }
}
