use crate::config::Pair;
use log::info;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("ZeroMQ error: {0}")]
    Zmq(#[from] zmq::Error),
}

pub fn run_pair(label: &str, pair: Pair) -> Result<(), ProxyError> {
    let ctx = zmq::Context::new();
    let xsub = ctx.socket(zmq::XSUB)?;
    let xpub = ctx.socket(zmq::XPUB)?;

    xsub.set_rcvhwm(pair.xsub_rcvhwm)?;
    xpub.set_sndhwm(pair.xpub_sndhwm)?;

    xsub.bind(&pair.frontend)?;
    xpub.bind(&pair.backend)?;

    info!("[{label}] ready; forwarding with zmq::proxy()");
    zmq::proxy(&xsub, &xpub)?;

    xsub.set_linger(0).ok();
    xpub.set_linger(0).ok();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_pair_invalid_endpoint() {
        let pair = Pair {
            name: None,
            frontend: "bad".to_string(),
            backend: "bad".to_string(),
            xsub_rcvhwm: 1,
            xpub_sndhwm: 1,
        };
        let err = run_pair("test", pair).unwrap_err();
        assert!(matches!(err, ProxyError::Zmq(_)));
    }
}
