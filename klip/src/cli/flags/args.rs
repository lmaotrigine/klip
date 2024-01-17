use std::{ffi::OsString, num::NonZeroUsize, path::PathBuf};

#[derive(Debug, Default)]
pub struct Args {
    mode: Mode,
    positional: Vec<OsString>,
    config: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    HelpSubcommand,
    HelpLong,
    #[default]
    HelpShort,
    VersionShort,
    VersionLong,
    Copy,
    Move,
    Paste,
    Serve(ServeArgs),
    Generate(GenerateMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServeArgs {
    max_clients: NonZeroUsize,
    max_len: u64,
    timeout: u64,
    data_timeout: u64,
}

impl Default for ServeArgs {
    fn default() -> Self {
        Self {
            max_clients: NonZeroUsize::new(10).expect("wtf"),
            max_len: 0,
            timeout: 10,
            data_timeout: 3600,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerateMode {
    Man,
    CompleteBash,
    CompleteZsh,
    CompleteFish,
    CompletePowerShell,
    Keys(GenerateKeysMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GenerateKeysMode {
    #[default]
    Random,
    Deterministic,
}
