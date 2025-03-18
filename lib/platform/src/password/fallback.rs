use std::{
    fs::File,
    io::{BufRead, BufReader, Result, Write},
};

pub fn read_password() -> Result<String> {
    let tty = File::open("/dev/tty")?;
    let mut reader = BufReader::new(tty);
    let mut password = super::Password::new();
    reader.read_line(&mut password.0)?;
    super::fix_line(password.into_inner())
}

pub fn print_tty(prompt: &str) -> Result<()> {
    write!(std::io::stdout(), "{prompt}")?;
    std::io::stdout().flush()?;
    Ok(())
}
