use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use argh::FromArgs;
use lettre::message::header::ContentType;
use lettre::message::{Attachment, SinglePart};
use lettre::Transport;
use log::{error, info};

use time_sheet::generate_time_sheet;
use time_sheet::input::Config;

fn set_env_if_absent<K: AsRef<OsStr>, V: AsRef<OsStr>>(var: K, default: impl FnOnce() -> V) {
    if env::var(var.as_ref()).is_err() {
        env::set_var(var, default());
    }
}

fn main() {
    set_env_if_absent("RUST_APP_LOG", || "trace");
    color_backtrace::install();
    pretty_env_logger::init_custom_env("RUST_APP_LOG");

    if let Err(e) = run() {
        error!("{:?}", e);
        ::std::process::exit(1);
    }
}

/// A time sheet generator for the german university KIT
#[derive(FromArgs)]
struct TopArgs {
    #[argh(subcommand)]
    command: SubCommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum SubCommands {
    Make(MakeArgs),
    Send(SendArgs),
}

/// Makes a time sheet from the given files.
#[derive(FromArgs)]
#[argh(subcommand, name = "make")]
struct MakeArgs {
    /// path to the global file.
    #[argh(option)]
    global: PathBuf,
    /// path to the month file.
    #[argh(option)]
    month: PathBuf,
    /// path to the output folder. default: `<path to month>/pdfs/`
    #[argh(option)]
    output: Option<PathBuf>,
}

/// Makes a time sheet from the given files and sends it to the email.
#[derive(FromArgs)]
#[argh(subcommand, name = "send")]
struct SendArgs {
    /// the title of the email. `{year}` and `{month}` will be replaced with the year/month.
    #[argh(option)]
    subject: String,
    /// path to the global file.
    #[argh(option)]
    global: PathBuf,
    /// path to the month file.
    #[argh(option)]
    month: PathBuf,
    /// path to the output folder. default: `<path to month>/pdfs/`
    #[argh(option)]
    output: Option<PathBuf>,
    /// keeps the pdf file after sending the email.
    #[argh(switch)]
    keep_pdf: bool,
    /// recipient of the email.
    #[argh(positional)]
    recipient: String,
}

fn build_config(global: &Path, month: &Path, output: &Path) -> anyhow::Result<Config> {
    let mut config = Config::try_from_toml_files(month, global)?;

    config.output(output);

    let config = config.build()?;

    info!("finished building config");

    Ok(config)
}

fn resolve_paths(
    global: PathBuf,
    month: PathBuf,
    output: Option<PathBuf>,
) -> anyhow::Result<(PathBuf, PathBuf, PathBuf)> {
    let workspace = dunce::canonicalize(&month)
        .map_err(|e| anyhow::anyhow!(e))?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("month should have a parent directory"))?
        .to_path_buf();

    let output = output.unwrap_or_else(|| workspace.join("pdfs/"));

    Ok((global, month, output))
}

fn attachment_from_file(path: impl AsRef<Path>) -> anyhow::Result<SinglePart> {
    let path = path.as_ref();

    Ok(Attachment::new(
        path.file_name()
            .ok_or_else(|| anyhow::anyhow!("missing file_name in path \"{}\"", path.display()))?
            .to_str()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to convert path to a unicode string: \"{}\"",
                    path.display()
                )
            })?
            .to_string(),
    )
    .body(fs::read(path)?, ContentType::parse("application/pdf")?))
}

fn send(config: &Config, recipient: &str, subject: &str, keep_pdf: bool) -> anyhow::Result<()> {
    let mail = config
        .mail()
        .ok_or_else(|| anyhow::anyhow!("missing mail config in global config"))?;

    // adjust subject:
    let subject = subject
        .replace("{year:04}", &format!("{:04}", config.month().year()))
        .replace(
            "{year:02}",
            &format!("{:02}", config.month().year().as_usize() % 100),
        )
        .replace(
            "{month:02}",
            &format!("{:02}", config.month().month().as_usize()),
        );

    make(config)?;

    let email = mail
        .builder()
        .to(recipient.parse()?)
        .subject(&subject)
        // attach the file to the email:
        .singlepart(attachment_from_file(config.output())?)?;

    info!(
        "sending email to \"{}\" with subject \"{}\"",
        recipient, &subject
    );

    mail.to_transport().send(&email).with_context(|| {
        format!(
            "failed to send email to \"{}\" with subject \"{}\"",
            recipient, subject
        )
    })?;

    info!("sent email successfully");

    if !keep_pdf {
        info!("removing pdf file");
        fs::remove_file(config.output())?;
    }

    Ok(())
}

fn make(config: &Config) -> anyhow::Result<()> {
    generate_time_sheet(config)?;

    Ok(())
}

fn run() -> anyhow::Result<()> {
    let TopArgs { command } = argh::from_env();

    match command {
        SubCommands::Make(args) => {
            let (global, month, output) = resolve_paths(args.global, args.month, args.output)?;
            let config = build_config(&global, &month, &output)?;
            make(&config)
        }
        SubCommands::Send(args) => {
            let (global, month, output) = resolve_paths(args.global, args.month, args.output)?;
            let config = build_config(&global, &month, &output)?;

            info!("recipient: \"{}\"", args.recipient);

            send(&config, &args.recipient, &args.subject, args.keep_pdf)
        }
    }
}
