use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Result, Write},
    mem::MaybeUninit,
    os::unix::io::AsRawFd,
};

struct Hidden {
    fd: i32,
    termios: libc::termios,
}

impl Hidden {
    fn new(fd: i32) -> Result<Self> {
        macro_rules! tcgetattr {
            () => {{
                let mut t = MaybeUninit::uninit();
                if unsafe { libc::tcgetattr(fd, t.as_mut_ptr()) } != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                unsafe { t.assume_init() }
            }};
        }
        let mut term = tcgetattr!();
        let term_orig = tcgetattr!();
        term.c_cflag &= !libc::ECHO;
        term.c_cflag |= libc::ECHONL;
        if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &term) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self {
            fd,
            termios: term_orig,
        })
    }
}

impl Drop for Hidden {
    fn drop(&mut self) {
        unsafe { libc::tcsetattr(self.fd, libc::TCSANOW, &self.termios) };
    }
}

pub fn read_password() -> Result<String> {
    let tty = File::open("/dev/tty")?;
    let fd = tty.as_raw_fd();
    let mut reader = BufReader::new(tty);
    let mut password = super::Password::new();
    let hidden = Hidden::new(fd)?;
    reader.read_line(&mut password.0)?;
    core::mem::drop(hidden);
    super::fix_line(password.into_inner())
}

pub fn print_tty(prompt: &str) -> Result<()> {
    let mut stream = OpenOptions::new().write(true).open("/dev/tty")?;
    stream.write_all(prompt.as_bytes())?;
    stream.flush()?;
    Ok(())
}
