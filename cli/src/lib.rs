#![forbid(unsafe_code)]
#![deny(
    dead_code,
    deprecated,
    future_incompatible,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used
)]
#![allow(
    clippy::should_implement_trait,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]

use std::{
    ffi::{OsStr, OsString},
    fmt::Display,
    mem::{replace, take},
    str::FromStr,
};

#[cfg(unix)]
use std::os::unix::ffi::{OsStrExt, OsStringExt};
#[cfg(target_os = "wasi")]
use std::os::wasi::ffi::{OsStrExt, OsStringExt};
#[cfg(windows)]
use std::os::windows::ffi::{OsStrExt, OsStringExt};

type Iter = std::vec::IntoIter<OsString>;

fn make_iter<I: Iterator<Item = OsString>>(i: I) -> Iter {
    i.collect::<Vec<_>>().into_iter()
}

#[derive(Debug, Clone)]
enum State {
    None,
    PendingValue(OsString),
    Shorts(Vec<u8>, usize),
    #[cfg(windows)]
    ShortsU16(Vec<u16>, usize),
    FinishedOpts,
}

#[derive(Debug, Clone)]
enum LastOption {
    None,
    Short(char),
    Long(String),
}

pub enum Error {
    MissingValue(Option<String>),
    UnexpectedOption(String),
    UnexpectedArgument(OsString),
    UnexpectedValue(String, OsString),
    ParsingFailed(String, Box<dyn std::error::Error + Send + Sync + 'static>),
    NonUnicodeValue(OsString),
    Other(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingValue(None) => write!(f, "missing argument"),
            Self::MissingValue(Some(option)) => write!(f, "missing argument for option `{option}`"),
            Self::UnexpectedOption(option) => write!(f, "invalid option `{option}`"),
            Self::UnexpectedArgument(value) => write!(f, "unexpected argument {value:?}"),
            Self::UnexpectedValue(option, value) => {
                write!(f, "unexpected argument for option `{option}`: {value:?}")
            }
            Self::NonUnicodeValue(value) => write!(f, "argument is invalid unicode: {value:?}"),
            Self::ParsingFailed(value, error) => {
                write!(f, "cannot parse argument {value:?}: {error}")
            }
            Self::Other(err) => write!(f, "{err}"),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParsingFailed(_, error) | Self::Other(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Other(value.into())
    }
}

impl<'a> From<&'a str> for Error {
    fn from(value: &'a str) -> Self {
        Self::Other(value.into())
    }
}

impl From<OsString> for Error {
    fn from(value: OsString) -> Self {
        Self::NonUnicodeValue(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arg<'a> {
    Short(char),
    Long(&'a str),
    Value(OsString),
}

impl<'a> Arg<'a> {
    #[must_use]
    pub fn unexpected(self) -> Error {
        match self {
            Self::Short(short) => Error::UnexpectedOption(format!("-{short}")),
            Self::Long(long) => Error::UnexpectedOption(format!("--{long}")),
            Self::Value(value) => Error::UnexpectedArgument(value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Parser {
    source: Iter,
    state: State,
    last_option: LastOption,
    bin_name: Option<String>,
}

impl Parser {
    #[allow(clippy::too_many_lines)]
    pub fn next(&mut self) -> Result<Option<Arg<'_>>, Error> {
        match self.state {
            State::PendingValue(ref mut value) => {
                let value = take(value);
                self.state = State::None;
                return Err(Error::UnexpectedValue(
                    self.format_last_option()
                        .expect("should not be in this state if on last_option."),
                    value,
                ));
            }
            State::Shorts(ref arg, ref mut pos) => match first_codepoint(&arg[*pos..]) {
                Ok(None) => {
                    self.state = State::None;
                }
                Ok(Some('=')) if *pos > 1 => {
                    return Err(Error::UnexpectedValue(
                        self.format_last_option().expect("can't be None here."),
                        self.optional_value().expect("can't be None here."),
                    ));
                }
                Ok(Some(ch)) => {
                    *pos += ch.len_utf8();
                    self.last_option = LastOption::Short(ch);
                    return Ok(Some(Arg::Short(ch)));
                }
                Err(_) => {
                    *pos += 1;
                    self.last_option = LastOption::Short('\u{FFFD}');
                    return Ok(Some(Arg::Short('\u{FFFD}')));
                }
            },
            #[cfg(windows)]
            State::ShortsU16(ref arg, ref mut pos) => match first_utf16_codepoint(&arg[*pos..]) {
                Ok(None) => {
                    self.state = State::None;
                }
                Ok(Some('=')) if *pos > 1 => {
                    return Err(Error::UnexpectedValue(
                        self.format_last_option().expect("can't be None here."),
                        self.optional_value().expect("can't be None here."),
                    ))
                }
                Ok(Some(ch)) => {
                    *pos += ch.len_utf16();
                    self.last_option = LastOption::Short(ch);
                    return Ok(Some(Arg::Short(ch)));
                }
                Err(_) => {
                    *pos += 1;
                    self.last_option = LastOption::Short('\u{FFFD}');
                    return Ok(Some(Arg::Short('\u{FFFD}')));
                }
            },
            State::FinishedOpts => return Ok(self.source.next().map(Arg::Value)),
            State::None => {}
        }
        match self.state {
            State::None => {}
            ref state => panic!("unexpected state {state:?}"),
        }
        let Some(arg) = self.source.next() else {
            return Ok(None);
        };
        if arg == "--" {
            self.state = State::FinishedOpts;
            return self.next();
        }
        #[cfg(any(unix, target_os = "wasi"))]
        {
            let mut arg = arg.into_vec();
            if arg.starts_with(b"--") {
                if let Some(idx) = arg.iter().position(|&b| b == b'=') {
                    self.state = State::PendingValue(OsString::from_vec(arg[idx + 1..].into()));
                    arg.truncate(idx);
                }
                let option = match String::from_utf8(arg) {
                    Ok(text) => text,
                    Err(err) => String::from_utf8_lossy(err.as_bytes()).into_owned(),
                };
                Ok(Some(self.set_long(option)))
            } else if arg.len() > 1 && arg[0] == b'-' {
                self.state = State::Shorts(arg, 1);
                self.next()
            } else {
                Ok(Some(Arg::Value(OsString::from_vec(arg))))
            }
        }
        #[cfg(not(any(unix, target_os = "wasi")))]
        {
            #[cfg(windows)]
            {
                const DASH: u16 = b'-' as u16;
                let mut bytes = arg.encode_wide();
                match (bytes.next(), bytes.next()) {
                    (Some(DASH), Some(_)) => {}
                    _ => return Ok(Some(Arg::Value(arg))),
                }
            }
            let mut arg = match arg.into_string() {
                Ok(arg) => arg,
                Err(arg) => {
                    #[cfg(windows)]
                    {
                        const DASH: u16 = b'-' as u16;
                        const EQ: u16 = b'=' as u16;
                        let mut arg = arg.encode_wide().collect::<Vec<_>>();
                        if arg.starts_with(&[DASH, DASH]) {
                            if let Some(idx) = arg.iter().position(|&u| u == EQ) {
                                self.state =
                                    State::PendingValue(OsString::from_wide(&arg[idx + 1..]));
                                arg.truncate(idx);
                            }
                            let long = self.set_long(String::from_utf16_lossy(&arg));
                            return Ok(Some(long));
                        } else {
                            assert!(arg.len() > 1);
                            assert_eq!(arg[0], DASH);
                            self.state = State::ShortsU16(arg, 1);
                            return self.next();
                        }
                    }
                    #[cfg(not(windows))]
                    {
                        let text = arg.to_string_lossy();
                        if text.starts_with('-') {
                            text.into_owned()
                        } else {
                            return Ok(Some(Arg::Value(arg)));
                        }
                    }
                }
            };
            if arg.starts_with("--") {
                if let Some(idx) = arg.find('=') {
                    self.state = State::PendingValue(arg[idx + 1..].into());
                    arg.truncate(idx);
                }
                Ok(Some(self.set_long(arg)))
            } else if arg.starts_with('-') && arg != "-" {
                self.state = State::Shorts(arg.into(), 1);
                self.next()
            } else {
                Ok(Some(Arg::Value(arg.into())))
            }
        }
    }

    pub fn value(&mut self) -> Result<OsString, Error> {
        if let Some(value) = self.optional_value() {
            return Ok(value);
        }
        if let Some(value) = self.source.next() {
            return Ok(value);
        }
        Err(Error::MissingValue(self.format_last_option()))
    }

    pub fn values(&mut self) -> Result<ValuesIter<'_>, Error> {
        if self.has_pending() || self.next_is_normal() {
            Ok(ValuesIter {
                took_first: false,
                parser: Some(self),
            })
        } else {
            Err(Error::MissingValue(self.format_last_option()))
        }
    }

    fn next_if_normal(&mut self) -> Option<OsString> {
        if self.next_is_normal() {
            self.source.next()
        } else {
            None
        }
    }

    fn next_is_normal(&self) -> bool {
        assert!(!self.has_pending());
        let arg = match self.source.as_slice().first() {
            None => return false,
            Some(arg) => arg,
        };
        if matches!(self.state, State::FinishedOpts) {
            return true;
        }
        if arg == "-" {
            return true;
        }
        #[cfg(any(unix, target_os = "wasi"))]
        let lead_dash = arg.as_bytes().first() == Some(&b'-');
        #[cfg(windows)]
        let lead_dash = arg.encode_wide().next() == Some(b'-' as u16);
        #[cfg(not(any(unix, target_os = "wasi", windows)))]
        let lead_dash = arg.to_string_lossy().as_bytes().first() == Some(&b'-');
        !lead_dash
    }

    pub fn raw_args(&mut self) -> Result<RawArgs<'_>, Error> {
        if let Some(value) = self.optional_value() {
            return Err(Error::UnexpectedValue(
                self.format_last_option().expect("should not be None here."),
                value,
            ));
        }
        Ok(RawArgs(&mut self.source))
    }

    pub fn try_raw_args(&mut self) -> Option<RawArgs<'_>> {
        if self.has_pending() {
            None
        } else {
            Some(RawArgs(&mut self.source))
        }
    }

    fn has_pending(&self) -> bool {
        match self.state {
            State::None | State::FinishedOpts => false,
            State::PendingValue(_) => true,
            State::Shorts(ref arg, pos) => pos < arg.len(),
            #[cfg(windows)]
            State::ShortsU16(ref arg, pos) => pos < arg.len(),
        }
    }

    #[inline(never)]
    fn format_last_option(&self) -> Option<String> {
        match self.last_option {
            LastOption::None => None,
            LastOption::Short(ch) => Some(format!("-{ch}")),
            LastOption::Long(ref long) => Some(long.clone()),
        }
    }

    #[must_use]
    pub fn bin_name(&self) -> Option<&str> {
        Some(self.bin_name.as_ref()?)
    }

    pub fn optional_value(&mut self) -> Option<OsString> {
        Some(self.raw_optional_value()?.0)
    }

    fn raw_optional_value(&mut self) -> Option<(OsString, bool)> {
        match replace(&mut self.state, State::None) {
            State::PendingValue(value) => Some((value, true)),
            State::Shorts(mut arg, mut pos) => {
                if pos >= arg.len() {
                    return None;
                }
                let had_eq = if arg[pos] == b'=' {
                    pos += 1;
                    true
                } else {
                    false
                };
                arg.drain(..pos);
                #[cfg(any(unix, target_os = "wasi"))]
                {
                    Some((OsString::from_vec(arg), had_eq))
                }
                #[cfg(not(any(unix, target_os = "wasi")))]
                {
                    let arg = String::from_utf8(arg)
                        .expect("short option args on exotic platforms must be valid unicode.");
                    Some((arg.into(), had_eq))
                }
            }
            #[cfg(windows)]
            State::ShortsU16(arg, mut pos) => {
                if pos >= arg.len() {
                    return None;
                }
                let mut had_eq = false;
                if arg[pos] == b'=' as u16 {
                    pos += 1;
                    had_eq = true;
                }
                Some((OsString::from_wide(&arg[pos..]), had_eq))
            }
            State::FinishedOpts => {
                self.state = State::FinishedOpts;
                None
            }
            State::None => None,
        }
    }

    fn new(bin_name: Option<OsString>, source: Iter) -> Self {
        Self {
            source,
            state: State::None,
            last_option: LastOption::None,
            bin_name: bin_name.map(|s| match s.into_string() {
                Ok(text) => text,
                Err(text) => text.to_string_lossy().into_owned(),
            }),
        }
    }

    #[must_use]
    pub fn from_env() -> Self {
        let mut source = make_iter(std::env::args_os());
        Self::new(source.next(), source)
    }

    pub fn from_iter<I: IntoIterator<Item = O>, O: Into<OsString>>(args: I) -> Self {
        let mut args = make_iter(args.into_iter().map(Into::into));
        Self::new(args.next(), args)
    }

    pub fn from_args<I: IntoIterator<Item = O>, O: Into<OsString>>(args: I) -> Self {
        Self::new(None, make_iter(args.into_iter().map(Into::into)))
    }

    fn set_long(&mut self, option: String) -> Arg<'_> {
        self.last_option = LastOption::Long(option);
        match self.last_option {
            LastOption::Long(ref option) => Arg::Long(&option[2..]),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct ValuesIter<'a> {
    took_first: bool,
    parser: Option<&'a mut Parser>,
}

impl<'a> Iterator for ValuesIter<'a> {
    type Item = OsString;

    fn next(&mut self) -> Option<Self::Item> {
        let parser = self.parser.as_mut()?;
        if self.took_first {
            parser.next_if_normal()
        } else if let Some((value, had_eq)) = parser.raw_optional_value() {
            if had_eq {
                self.parser = None;
            }
            self.took_first = true;
            Some(value)
        } else {
            let value = parser
                .next_if_normal()
                .expect("at least one value must exist.");
            self.took_first = true;
            Some(value)
        }
    }
}

#[derive(Debug)]
pub struct RawArgs<'a>(&'a mut Iter);

impl<'a> RawArgs<'a> {
    #[must_use]
    pub fn peek(&self) -> Option<&OsStr> {
        Some(self.0.as_slice().first()?.as_os_str())
    }

    pub fn next_if<F: FnOnce(&OsStr) -> bool>(&mut self, f: F) -> Option<OsString> {
        match self.peek() {
            Some(arg) if f(arg) => self.next(),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_slice(&self) -> &[OsString] {
        self.0.as_slice()
    }
}

impl<'a> Iterator for RawArgs<'a> {
    type Item = OsString;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for std::ffi::OsString {}
}

pub trait ParseExt: sealed::Sealed {
    fn parse<T: FromStr<Err = E>, E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>>(
        &self,
    ) -> Result<T, Error>;
    fn parse_with<
        F: FnOnce(&str) -> Result<T, E>,
        T,
        E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    >(
        &self,
        f: F,
    ) -> Result<T, Error>;
    fn string(self) -> Result<String, Error>;
}

impl ParseExt for OsString {
    fn parse<T: FromStr<Err = E>, E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>>(
        &self,
    ) -> Result<T, Error> {
        self.parse_with(FromStr::from_str)
    }

    fn parse_with<
        F: FnOnce(&str) -> Result<T, E>,
        T,
        E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    >(
        &self,
        f: F,
    ) -> Result<T, Error> {
        self.to_str().map_or_else(
            || Err(Error::NonUnicodeValue(self.into())),
            |text| match f(text) {
                Ok(value) => Ok(value),
                Err(err) => Err(Error::ParsingFailed(text.to_owned(), err.into())),
            },
        )
    }

    fn string(self) -> Result<String, Error> {
        match self.into_string() {
            Ok(string) => Ok(string),
            Err(raw) => Err(Error::NonUnicodeValue(raw)),
        }
    }
}

fn first_codepoint(bytes: &[u8]) -> Result<Option<char>, u8> {
    let bytes = bytes.get(..4).unwrap_or(bytes);
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(err) if err.valid_up_to() > 0 => {
            std::str::from_utf8(&bytes[..err.valid_up_to()]).expect("just checked this")
        }
        Err(_) => return Err(bytes[0]),
    };
    Ok(text.chars().next())
}

#[cfg(windows)]
fn first_utf16_codepoint(units: &[u16]) -> Result<Option<char>, u16> {
    match std::char::decode_utf16(units.iter().copied()).next() {
        Some(Ok(ch)) => Ok(Some(ch)),
        Some(Err(_)) => Err(units[0]),
        None => Ok(None),
    }
}

pub mod prelude {
    pub use super::{Arg::*, ParseExt};
}
