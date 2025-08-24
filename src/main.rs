use log::{error, info};
use serde::Deserialize;
use std::thread;
use thiserror::Error;

#[derive(Debug, Error)]
enum ProxyError {
    #[error("configuration load error: {0}")]
    ConfigLoad(#[from] config::ConfigError),
    #[error("ZeroMQ error: {0}")]
    Zmq(#[from] zmq::Error),
    #[error("invalid configuration: {0}")]
    Config(String),
}

#[derive(Debug, Deserialize)]
struct Settings {
    pairs: Vec<Pair>,
}

#[derive(Debug, Deserialize, Clone)]
struct Pair {
    /// Optional label shown in logs
    name: Option<String>,
    /// Bind XSUB here (publishers connect)
    frontend: String,
    /// Bind XPUB here (subscribers connect)
    backend: String,
    /// XSUB receive high-water mark
    #[serde(default = "default_hwm")]
    xsub_rcvhwm: i32,
    /// XPUB send high-water mark
    #[serde(default = "default_hwm")]
    xpub_sndhwm: i32,
}

fn default_hwm() -> i32 {
    100_000
}

fn main() -> Result<(), ProxyError> {
    // Initialize logging: set RUST_LOG=info (or debug, trace, …) to control verbosity.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Load configuration from proxy.{yaml|toml|json} in the current dir.
    // The first existing file among these will be used (they're all optional).
    let cfg = config::Config::builder()
        .add_source(config::File::new("proxy", config::FileFormat::Yaml).required(false))
        .add_source(config::File::new("proxy", config::FileFormat::Toml).required(false))
        .add_source(config::File::new("proxy", config::FileFormat::Json).required(false))
        .build()?;

    let settings: Settings = cfg.try_deserialize()?;
    if settings.pairs.is_empty() {
        return Err(ProxyError::Config("no pairs defined".into()));
    }

    info!("loaded {} pair(s)", settings.pairs.len());

    // One thread per pair; each runs a blocking built-in proxy.
    let mut handles = Vec::with_capacity(settings.pairs.len());
    for (i, pair) in settings.pairs.into_iter().enumerate() {
        let label = pair
            .name
            .clone()
            .unwrap_or_else(|| format!("pair-{}", i + 1));
        info!(
            "[{label}] XSUB bind: {}  |  XPUB bind: {}  |  HWM(rx,tx)=({}, {})",
            pair.frontend, pair.backend, pair.xsub_rcvhwm, pair.xpub_sndhwm
        );

        handles.push(thread::spawn(move || {
            if let Err(e) = run_pair(&label, pair) {
                error!("[{label}] error: {e}");
            }
        }));
    }

    for h in handles {
        let _ = h.join();
    }
    Ok(())
}

fn run_pair(label: &str, pair: Pair) -> Result<(), ProxyError> {
    let ctx = zmq::Context::new();
    let xsub = ctx.socket(zmq::XSUB)?;
    let xpub = ctx.socket(zmq::XPUB)?;

    xsub.set_rcvhwm(pair.xsub_rcvhwm)?;
    xpub.set_sndhwm(pair.xpub_sndhwm)?;

    xsub.bind(&pair.frontend)?;
    xpub.bind(&pair.backend)?;

    info!("[{label}] ready; forwarding with zmq::proxy()");
    // Blocks until interrupted or an error occurs.
    zmq::proxy(&xsub, &xpub)?;

    // Best-effort cleanup if proxy ever returns.
    xsub.set_linger(0).ok();
    xpub.set_linger(0).ok();

    Ok(())
}
