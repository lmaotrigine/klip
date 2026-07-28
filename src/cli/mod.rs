use crate::{
    config::{Config, TomlConfig},
    error::{Context, Error, ResultExt},
    state::State,
};
use platform::env::home_dir;
use std::{num::NonZeroUsize, path::PathBuf};

mod commands;

pub fn version(long: bool) -> &'static str {
    static STORAGE: std::sync::OnceLock<[String; 2]> = std::sync::OnceLock::new();
    STORAGE.get_or_init(|| {
        let short = format!("v{}", option_env!("CARGO_PKG_VERSION").unwrap_or("N/A"));
        let protocol_version = crate::client::DEFAULT_CLIENT_VERSION;
        let long = option_env!("KLIP_BUILD_GIT_HASH").map_or_else(
            || format!("{short} (protocol version {protocol_version})"),
            |hash| format!("{short} (rev {hash}) (protocol version {protocol_version})"),
        );
        [short, long]
    })[usize::from(long)]
    .as_str()
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
pub enum Subcommand {
    Copy,
    Paste,
    Move,
    Serve(ServerArgs),
    Keygen(bool),
    Version,
}

impl Subcommand {
    fn from_arg_matches_mut(matches: &mut clap::ArgMatches) -> Result<Self, clap::Error> {
        if let Some((subcommand, mut sub_matches)) = matches.remove_subcommand() {
            match subcommand.as_str() {
                "copy" => Ok(Self::Copy),
                "paste" => Ok(Self::Paste),
                "move" => Ok(Self::Move),
                "serve" => Ok(Self::Serve(ServerArgs::from_arg_matches_mut(
                    &mut sub_matches,
                )?)),
                "genkeys" => Ok(Self::Keygen(
                    sub_matches.remove_one("password").unwrap_or(false),
                )),
                "version" => Ok(Self::Version),
                _ => Err(clap::Error::raw(
                    clap::error::ErrorKind::InvalidSubcommand,
                    format!("the subcommand '{subcommand}' wasn't recognized"),
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

#[derive(Debug)]
pub struct Cli {
    pub subcommand: Subcommand,
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

    pub async fn run() -> Result<(), Context> {
        let cli = Self::parse();
        if let Subcommand::Keygen(password) = cli.subcommand {
            let config_file = match cli.config {
                Some(config_file) => config_file,
                None => Self::default_config_file()?,
            };
            let key = if password {
                rpassword::prompt_password("Password: ")
                    .context("failed to read password interactively")?
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
            Subcommand::Version => {
                println!("klip {}", version(true));
                Ok(())
            }
            Subcommand::Copy => crate::client::run(config, true, false).await,
            Subcommand::Move => crate::client::run(config, false, true).await,
            Subcommand::Paste => crate::client::run(config, false, false).await,
            Subcommand::Serve(_) => crate::server::serve(State::new(config)).await,
            Subcommand::Keygen(_) => unreachable!(),
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
