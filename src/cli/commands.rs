use std::{num::NonZeroUsize, path::PathBuf};

const HELP_TEMPLATE: &str = "{name} {version}\n{author-with-newline}\n{about-with-newline}\n{usage-heading} {usage}\n\n{all-args}";

fn command(id: &'static str) -> clap::Command {
    clap::Command::new(id)
        .author("Isis Ebsen <isis@5ht2.me>")
        .version(super::version(false))
        .long_version(super::version(true))
        .help_template(HELP_TEMPLATE)
}

fn subcommands() -> [clap::Command; 6] {
    [
        command("copy")
            .about("Store content")
            .alias("c")
            .disable_version_flag(true),
        command("paste")
            .about("Retrieve content")
            .alias("p")
            .disable_version_flag(true),
        command("move")
            .about("Retrieve and delete content")
            .alias("m")
            .disable_version_flag(true),
        command("serve")
            .about("Start a server")
            .disable_version_flag(true)
            .args([
                clap::Arg::new("max_clients")
                    .help("Maximum number of simultaneous client connections")
                    .long("max-clients")
                    .value_name("NUM")
                    .default_value("10")
                    .value_parser(clap::value_parser!(NonZeroUsize)),
                clap::Arg::new("max_len_mb")
                    .help("Maximum content length to accept in MiB (0=unlimited)")
                    .long("max-len-mb")
                    .value_name("NUM")
                    .default_value("0")
                    .value_parser(clap::value_parser!(u64)),
                clap::Arg::new("timeout")
                    .help("Connection timeout (in seconds)")
                    .long("timeout")
                    .short('t')
                    .value_name("TIMEOUT")
                    .default_value("10")
                    .value_parser(clap::value_parser!(u64)),
                clap::Arg::new("data_timeout")
                    .help("Data transmission timeout (in seconds)")
                    .long("data-timeout")
                    .short('d')
                    .value_name("TIMEOUT")
                    .default_value("3600")
                    .value_parser(clap::value_parser!(u64)),
            ]),
        command("genkeys")
            .about("Generate keys")
            .disable_version_flag(true)
            .arg(
                clap::Arg::new("password")
                    .help("Derive the keys from a password (default=random keys)")
                    .action(clap::ArgAction::SetTrue)
                    .long("password")
                    .short('p'),
            ),
        command("version")
            .about("Print version")
            .disable_version_flag(true),
    ]
}

pub fn app() -> clap::Command {
    command("klip")
        .about("Copy/paste anything over the network")
        .subcommands(subcommands())
        .arg(
            clap::Arg::new("config")
                .help("Path to the configuration file (default=$HOME/.klip.toml)")
                .long("config")
                .short('c')
                .value_name("FILE")
                .required(false)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .subcommand_required(true)
        .arg_required_else_help(true)
}
