use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
use tempfile::TempDir;
use thiserror::Error;

use crate::utils;

#[derive(Debug, Error)]
pub enum RenderingError {
    #[error(transparent)]
    RunError(io::Error),
    #[error(transparent)]
    ReadOutputFile(io::Error),
}

pub struct TexRender {
    /// Path to latexmk.
    latex_mk_path: PathBuf,
    /// Whether or not to use XeLaTeX.
    use_xelatex: bool,
    /// Whether or not to allow shell escaping.
    allow_shell_escape: bool,
    /// Temporary directory holding assets to be included.
    working_dir: TempDir,
    preserve_dir: Option<PathBuf>,
}

impl TexRender {
    pub fn from_bytes(source: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        let working_dir = TempDir::new()?;
        utils::write(working_dir.path().join("input.tex"), source.as_ref())?;

        Ok(Self {
            latex_mk_path: "latexmk".into(),
            use_xelatex: true,
            allow_shell_escape: false,
            working_dir,
            preserve_dir: None,
        })
    }

    pub fn add_asset_from_bytes(
        &mut self,
        filepath: impl AsRef<Path>,
        bytes: &[u8],
    ) -> io::Result<()> {
        let workdir_filepath = self.working_dir.path().join(filepath.as_ref());

        utils::create_dir_all(workdir_filepath.parent().expect("filename has no parent?"))?;
        utils::write(workdir_filepath, bytes)
    }

    pub fn preserve_dir(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.preserve_dir = Some(path.into());
        self
    }

    pub fn latex_mk_path(&mut self, latex_mk_path: impl Into<PathBuf>) -> &mut Self {
        self.latex_mk_path = latex_mk_path.into();
        self
    }

    pub fn render(self) -> anyhow::Result<Vec<u8>> {
        let input_file = self.working_dir.path().join("input.tex");
        let output_file = self.working_dir.path().join("input.pdf");

        let mut cmd = Command::new(&self.latex_mk_path);
        cmd.args([
            "-interaction=nonstopmode",
            "-halt-on-error",
            "-file-line-error",
            "-pdf",
            "-cd",
        ]);

        if self.use_xelatex {
            cmd.arg("-xelatex");
        }

        if !self.allow_shell_escape {
            cmd.arg("-no-shell-escape");
        }

        cmd.arg(&input_file);

        cmd.current_dir(self.working_dir.path());

        let output = cmd.output().map_err(RenderingError::RunError)?;

        if !output.status.success() {
            if let Some(path) = self.preserve_dir {
                utils::create_dir_all(&path)?;
                fs_extra::dir::copy(
                    self.working_dir.path(),
                    &path,
                    &fs_extra::dir::CopyOptions {
                        overwrite: true,
                        skip_exist: false,
                        ..Default::default()
                    },
                )
                .with_context(|| {
                    format!(
                        "failed to copy `{}` to `{}`",
                        self.working_dir.path().display(),
                        path.display()
                    )
                })?;
            }
            // latexmk failed,
            return Err(anyhow::anyhow!(
                "latexmk failed with status: {:?}, stdout: {}, stderr: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(utils::read(output_file).map_err(RenderingError::ReadOutputFile)?)
    }
}
