use klip::Cli;

macro_rules! eprintln_log {
    ($($tt:tt)*) => {{
        use ::std::io::Write;
        let mut stderr = ::std::io::stderr().lock();
        if let Err(e) = ::core::write!(stderr, "klip: ") {
            if e.kind() == ::std::io::ErrorKind::BrokenPipe {
                ::std::process::exit(0);
            } else {
                ::std::process::exit(1);
            }
        }
        if let Err(e) = ::core::writeln!(stderr, $($tt)*) {
            if e.kind() == ::std::io::ErrorKind::BrokenPipe {
                ::std::process::exit(0);
            } else {
                ::std::process::exit(1);
            }
        }
    }}
}

struct Logger(());
const LOGGER: &Logger = &Logger(());

impl Logger {
    fn init() -> Result<(), log::SetLoggerError> {
        log::set_logger(LOGGER)
    }
}

impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        match (record.file(), record.line()) {
            (Some(file), Some(line)) => {
                eprintln_log!(
                    "{}|{}|{file}:{line}: {}",
                    record.level(),
                    record.target(),
                    record.args()
                );
            }
            (Some(file), None) => {
                eprintln_log!("{}|{}|{file}: {}", record.level(), record.target(), record.args());
            }
            _ => {
                eprintln_log!("{}|{}: {}", record.level(), record.target(), record.args());
            }
        }
    }

    fn flush(&self) {}
}

// since Rust no longer uses jemalloc by default, klip will, by default, use the
// system allocator. on linux, this would normally be glibc's allocator, which
// is decent. in particular, klip does not have a particularly allocation-heavy
// workload, so there isn't much difference (for our purposes) between glibc's
// allocator and jemalloc.
//
// however, when klip is built with musl, this means klip will use musl's
// allocator, which appears to be substantially worse. musl's goal is not to
// have the fastest version of everything; its goal is to be small and amenable
// to static compilation. even though klip isn't particularly allocation heavy,
// musl's allocator appears to slow down klip quite a bit. therefore, when
// building with musl, we use jemalloc.
//
// we don't unconditionally use jemalloc because it can be nice to use the
// system's default allocator by default. moreover, jemalloc seems to increase
// compilation times by a bit.
//
// we also only do this on 64-bit systems because jemalloc doesn't support i686.
#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

async fn shutdown() {
    let ctrlc = async {
        tokio::signal::ctrl_c().await.expect("failed to install ^C handler");
    };
    #[cfg(unix)]
    let term = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let term = core::future::pending::<()>();
    tokio::select! {
        () = ctrlc => (),
        () = term => (),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(e) = Logger::init() {
        anyhow::bail!("failed to initialise logger: {e}");
    }
    log::set_max_level(log::LevelFilter::Warn);
    tokio::select! {
        r = Cli::run() => r,
        () = shutdown() => {
            log::warn!("violently shutting down");
            std::process::exit(1);
        }
    }
}
