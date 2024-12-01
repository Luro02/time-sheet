#![feature(never_type, step_trait, trait_alias, associated_type_defaults)]

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use lettre::message::header::ContentType;
use lettre::message::{Attachment, SinglePart};
use lettre::Transport;
use log::{error, info};
use seahorse::{App, Command, Context, Flag};

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

mod seahorse_exts {
    use core::fmt;
    use std::path::PathBuf;

    use anyhow::Context as _;
    use log::error;
    use seahorse::{App, Command, Context};

    type TryAction<E> = fn(_: &Context) -> Result<(), E>;

    pub trait ErrorLike: Send + Sync + fmt::Debug + 'static {}

    impl<E: Send + Sync + fmt::Debug + 'static> ErrorLike for E {}

    pub trait TryActionExt {
        #[must_use]
        fn try_action<E>(self, action: TryAction<E>) -> Self
        where
            E: ErrorLike;
    }

    impl TryActionExt for App {
        fn try_action<E>(self, action: TryAction<E>) -> Self
        where
            E: ErrorLike,
        {
            self.action(move |context: &Context| {
                if let Err(e) = action(context) {
                    error!("{:?}", e);
                    ::std::process::exit(1);
                }
            })
        }
    }

    impl TryActionExt for Command {
        fn try_action<E>(self, action: TryAction<E>) -> Self
        where
            E: ErrorLike,
        {
            self.action(move |context: &Context| {
                if let Err(e) = action(context) {
                    error!("{:?}", e);
                    ::std::process::exit(1);
                }
            })
        }
    }

    pub trait ContextExt {
        fn context(&self) -> &Context;

        fn required_string_flag(&self, name: &str) -> Result<String, anyhow::Error> {
            self.context()
                .string_flag(name)
                .with_context(|| anyhow::anyhow!("missing required flag \"{}\"", name))
        }

        fn required_path_flag(&self, name: &str) -> Result<PathBuf, anyhow::Error> {
            self.required_string_flag(name)
                .map(PathBuf::from)
                .with_context(|| anyhow::anyhow!("missing required flag \"{}\"", name))
        }
    }

    impl ContextExt for Context {
        fn context(&self) -> &Context {
            self
        }
    }
}

use seahorse_exts::{ContextExt, TryActionExt};

fn build_config(global: &Path, month: &Path, output: &Path) -> anyhow::Result<Config> {
    let mut config = Config::try_from_toml_files(month, global)?;

    config.output(output);

    let config = config.build()?;

    info!("finished building config");

    Ok(config)
}

fn make_extract_context_flags(context: &Context) -> anyhow::Result<(PathBuf, PathBuf, PathBuf)> {
    let global = context.required_path_flag("global")?;
    let month = context.required_path_flag("month")?;

    let workspace = dunce::canonicalize(&month)
        .map_err(|e| anyhow::anyhow!(e))?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("month should have a parent directory"))?
        .to_path_buf();

    let output = context
        .required_path_flag("output")
        .ok()
        .unwrap_or_else(|| workspace.join("pdfs/"));

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

    make(&config)?;

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
    let args: Vec<String> = env::args().collect();

    let make_command = Command::new("make")
        .usage(format!("{} make [args]", args[0]))
        .description("Makes a time sheet from the given files.")
        .flag(
            Flag::new("global", seahorse::FlagType::String).description("Path to the global file."),
        )
        .flag(Flag::new("month", seahorse::FlagType::String).description("Path to the month file."))
        .flag(
            Flag::new("output", seahorse::FlagType::String).description(
                "[optional] Path to the output folder. Default: `<path to month>/pdfs/`",
            ),
        )
        .try_action(|context: &Context| {
            let (global, month, output) = make_extract_context_flags(context)?;
            let config = build_config(&global, &month, &output)?;
            make(&config)
        });

    let send_command = Command::new("send")
        .usage(format!("{} send [args] recipient@example.com", args[0]))
        .description("Makes a time sheet from the given files and sends it to the email.")
        .flag(
            Flag::new("subject", seahorse::FlagType::String).description("The title of the email. `{year}` and `{month}` will be replaced with the year/month."),
        )
        .flag(
            Flag::new("global", seahorse::FlagType::String).description("Path to the global file."),
        )
        .flag(Flag::new("month", seahorse::FlagType::String).description("Path to the month file."))
        .flag(
            Flag::new("output", seahorse::FlagType::String).description(
                "[optional] Path to the output folder. Default: `<path to month>/pdfs/`",
            ),
        )
        .flag(Flag::new("keep-pdf", seahorse::FlagType::Bool).description("[optional] Keeps the pdf file after sending the email. Default: false"))
        .try_action(|context: &Context| {
            let (global, month, output) = make_extract_context_flags(context)?;
            let config = build_config(&global, &month, &output)?;

            let subject = context.required_string_flag("subject")?;

            if context.args.len() != 1 {
                return Err(anyhow::anyhow!("missing recipient or too many arguments"));
            }

            let keep_pdf = context.bool_flag("keep-pdf");

            let recipient = &context.args[0];
            info!("recipient: \"{}\"", recipient);

            send(&config, recipient, &subject, keep_pdf)
        });

    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage(format!("{} [args]", args[0]))
        .command(make_command)
        .command(send_command);

    app.run(args);

    Ok(())
}
