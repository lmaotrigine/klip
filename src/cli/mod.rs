use crate::{error::Error, util};
use anyhow::Context;
use std::{
    env::home_dir,
    io::{self, IsTerminal},
    num::NonZeroUsize,
    path::PathBuf,
};

mod commands;
mod config;
use config::Config;

pub fn version(long: bool) -> String {
    let short = format!("v{}", option_env!("CARGO_PKG_VERSION").unwrap_or("N/A"));
    if !long {
        return short;
    }
    let hash = option_env!("KLIP_BUILD_GIT_HASH")
        .map_or_else(String::new, |hash| format!(" (rev {hash})"));
    format!("{short}{hash} (protocol version {})", crate::client::DEFAULT_CLIENT_VERSION)
}

macro_rules! assert_some {
    ($matches:expr, $id:literal) => {
        $matches.remove_one($id).ok_or_else(|| {
            clap::Error::raw(
                clap::error::ErrorKind::MissingRequiredArgument,
                concat!("the following required argument was not provided: ", $id),
            )
        })
    };
}

#[derive(Debug, Clone, Copy)]
pub struct ServerArgs {
    pub max_clients: NonZeroUsize,
    pub max_len_mb: u64,
    pub timeout: u64,
    pub data_timeout: u64,
}

impl ServerArgs {
    fn from_arg_matches_mut(matches: &mut clap::ArgMatches) -> Result<Self, clap::Error> {
        Ok(Self {
            max_clients: assert_some!(matches, "max_clients")?,
            max_len_mb: assert_some!(matches, "max_len_mb")?,
            timeout: assert_some!(matches, "timeout")?,
            data_timeout: assert_some!(matches, "data_timeout")?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimeoutArgs {
    pub timeout: u64,
    pub data_timeout: u64,
}

impl TimeoutArgs {
    fn from_arg_matches_mut(matches: &mut clap::ArgMatches) -> Result<Self, clap::Error> {
        Ok(Self {
            timeout: assert_some!(matches, "timeout")?,
            data_timeout: assert_some!(matches, "data_timeout")?,
        })
    }
}

#[derive(Debug)]
pub enum Subcommand {
    Copy(TimeoutArgs),
    Paste(TimeoutArgs),
    Move(TimeoutArgs),
    Push(TimeoutArgs, String),
    Pull(TimeoutArgs, String),
    Serve(ServerArgs),
    Keygen(bool),
    Version,
}

impl Subcommand {
    fn from_arg_matches_mut(matches: &mut clap::ArgMatches) -> Result<Self, clap::Error> {
        if let Some((subcommand, mut sub_matches)) = matches.remove_subcommand() {
            match subcommand.as_str() {
                "copy" => Ok(Self::Copy(TimeoutArgs::from_arg_matches_mut(&mut sub_matches)?)),
                "paste" => Ok(Self::Paste(TimeoutArgs::from_arg_matches_mut(&mut sub_matches)?)),
                "move" => Ok(Self::Move(TimeoutArgs::from_arg_matches_mut(&mut sub_matches)?)),
                "push" => Ok(Self::Push(
                    TimeoutArgs::from_arg_matches_mut(&mut sub_matches)?,
                    sub_matches.remove_one("cid").unwrap_or_default(),
                )),
                "pull" => Ok(Self::Pull(
                    TimeoutArgs::from_arg_matches_mut(&mut sub_matches)?,
                    sub_matches.remove_one("cid").unwrap_or_default(),
                )),
                "serve" => Ok(Self::Serve(ServerArgs::from_arg_matches_mut(&mut sub_matches)?)),
                "genkeys" => Ok(Self::Keygen(sub_matches.remove_one("password").unwrap_or(false))),
                "version" => Ok(Self::Version),
                _ => Err(clap::Error::raw(
                    clap::error::ErrorKind::InvalidSubcommand,
                    format!("the subcommand '{subcommand}' wasn't recognised"),
                )),
            }
        } else {
            Err(clap::Error::raw(
                clap::error::ErrorKind::MissingSubcommand,
                "a subcommand is required but one was not provided",
            ))
        }
    }
}

pub enum Mode<'a> {
    Copy,
    Paste,
    Move,
    Push { cid: &'a str },
    Pull { cid: &'a str },
}

impl Mode<'_> {
    async fn run(self, config: Config<'_>) -> Result<(), Error> {
        let mut client = config.into_client()?;
        match self {
            Self::Copy => {
                client.copy(tokio::io::stdin()).await?;
                if io::stderr().is_terminal() {
                    eprintln!("Sent");
                }
                Ok(())
            }
            Self::Paste => client.paste(tokio::io::stdout(), false).await,
            Self::Move => client.paste(tokio::io::stdout(), true).await,
            Self::Push { cid } => {
                client.push_stream(tokio::io::stdin(), cid).await?;
                if io::stderr().is_terminal() {
                    eprintln!("Stream sent");
                }
                Ok(())
            }
            Self::Pull { cid } => client.pull_stream(tokio::io::stdout(), cid).await,
        }
    }
}

/// Represents the command line interface for klip. This struct is only useful
/// in the context of the klip binary.
#[derive(Debug)]
pub struct Cli {
    pub(crate) subcommand: Subcommand,
    config: Option<PathBuf>,
}

impl Cli {
    fn try_parse() -> Result<Self, clap::Error> {
        let mut matches = commands::app().get_matches();
        let subcommand = Subcommand::from_arg_matches_mut(&mut matches)?;
        let config = matches.remove_one("config");
        Ok(Self { subcommand, config })
    }

    fn parse() -> Self {
        Self::try_parse().unwrap_or_else(|e| e.exit())
    }

    /// Parse command line arguments and run the appropriate subcommand.
    ///
    /// # Errors
    ///
    /// Errors can occur while reading the config file, parsing it, or
    /// performing the requested operation. In any case, the error is returned
    /// wrapped in [`anyhow::Error`].
    pub async fn run() -> anyhow::Result<()> {
        let cli = Self::parse();
        if let Subcommand::Keygen(password) = cli.subcommand {
            let config_file = match cli.config {
                Some(config_file) => config_file,
                None => Self::default_config_file()?,
            };
            let key = if password {
                util::prompt("Password: ").context("failed to read password interactively")?
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
        let table = toml::from_str(
            &std::fs::read_to_string(config_file.canonicalize().with_context(|| {
                format!("failed to canonicalise config file path '{}'", config_file.display())
            })?)
            .with_context(|| format!("while reading config file at '{}'", config_file.display()))?,
        )
        .context("while parsing config file")?;
        let config = Config::new(table, &cli);
        Ok(match &cli.subcommand {
            Subcommand::Version => {
                println!("klip {}", version(true));
                Ok(())
            }
            Subcommand::Copy(_) => Mode::Copy.run(config).await,
            Subcommand::Move(_) => Mode::Move.run(config).await,
            Subcommand::Push(_, cid) => Mode::Push { cid }.run(config).await,
            Subcommand::Pull(_, cid) => Mode::Pull { cid }.run(config).await,
            Subcommand::Paste(_) => Mode::Paste.run(config).await,
            Subcommand::Serve(_) => config.into_server()?.serve().await,
            Subcommand::Keygen(_) => unreachable!(),
        }?)
    }

    fn default_config_file() -> Result<PathBuf, Error> {
        Ok(home_dir().ok_or(Error::NoHome)?.canonicalize()?.join(".klip.toml"))
    }
}
