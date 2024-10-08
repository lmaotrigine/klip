use crate::{
    config::{Config, TomlConfig},
    error::{Context, Error, ResultExt},
    state::State,
};
use clap::{Parser, Subcommand};
use platform::env::home_dir;
use std::{num::NonZeroUsize, path::PathBuf};

#[derive(Debug, Parser)]
#[clap(about, author, version = crate::EXPANDED_VERSION)]
#[clap(help_template = r"{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}")]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Command,
    #[clap(short, long)]
    /// path to the configuration file (default=$HOME/.klip.toml)
    pub config: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, Subcommand)]
pub enum Command {
    /// store content
    #[clap(alias = "c")]
    Copy,
    /// retrieve content
    #[clap(alias = "p")]
    Paste,
    /// retrieve and delete content
    #[clap(alias = "m")]
    Move,
    /// start a server
    Serve(ServerArgs),
    /// generate keys
    #[clap(name = "genkeys")]
    Keygen(KeygenArgs),
    /// show version information
    Version,
}

#[derive(Debug, Clone, Copy, Parser)]
#[clap(about, author, version = crate::EXPANDED_VERSION)]
#[clap(help_template = r"{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}")]
pub struct ServerArgs {
    /// the maximum number of simultaneous client connections
    #[clap(long, default_value = "10")]
    pub max_clients: NonZeroUsize,
    /// maximum content length to accept in MiB (0=unlimited)
    #[clap(long, default_value = "0")]
    pub max_len_mb: u64,
    /// connection timeout (in seconds)
    #[clap(short, long, default_value = "10")]
    pub timeout: u64,
    /// data transmission timeout (in seconds)
    #[clap(short, long, default_value = "3600")]
    pub data_timeout: u64,
}

#[derive(Debug, Parser, Clone, Copy)]
#[clap(about, author, version = crate::EXPANDED_VERSION)]
#[clap(help_template = r"{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}")]
pub struct KeygenArgs {
    /// derive the keys from a password (default=random keys)
    #[clap(short, long)]
    password: bool,
}

impl Cli {
    pub async fn run() -> Result<(), Context> {
        let cli = Self::parse();

        if let Command::Keygen(KeygenArgs { password }) = cli.subcommand {
            let config_file = match cli.config {
                Some(config_file) => config_file,
                None => Self::default_config_file()?,
            };
            let key = if password {
                platform::password::get().context("failed to read password interactively")?
            } else {
                String::new()
            };
            crate::keygen::generate_keys(config_file.display(), key.as_bytes());
            return Ok(());
        }
        let config_file = match &cli.config {
            Some(config_file) => config_file.clone(),
            None => Self::default_config_file()?,
        };
        let config = toml::from_str::<toml::value::Table>(
            &std::fs::read_to_string(config_file.canonicalize().context(format!(
                "failed to canonicalize config file path '{}'",
                config_file.display()
            ))?)
            .context(format!(
                "while reading config file at '{}'",
                config_file.display()
            ))?,
        )
        .context("while parsing config file")?;
        let toml_config = TomlConfig::new(config);
        let config = Config::new(&toml_config, &cli)?;
        let ret = match cli.subcommand {
            Command::Version => {
                println!("{}", crate::EXPANDED_VERSION);
                Ok(())
            }
            Command::Copy => crate::client::run(config, true, false).await,
            Command::Move => crate::client::run(config, false, true).await,
            Command::Paste => crate::client::run(config, false, false).await,
            Command::Serve(_) => crate::server::serve(State::new(config)).await,
            Command::Keygen(_) => unreachable!(),
        };
        Ok(ret?)
    }

    fn default_config_file() -> Result<PathBuf, Error> {
        Ok(home_dir()
            .ok_or(Error::NoHome)?
            .canonicalize()?
            .join(".klip.toml"))
    }
}
