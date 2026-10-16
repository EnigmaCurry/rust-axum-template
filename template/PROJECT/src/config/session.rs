use conf::Conf;
use serde::{Deserialize, Serialize};

#[derive(Conf, Debug, Clone, Serialize, Deserialize)]
#[conf(serde)]
pub struct SessionConfig {
    /// Session cleanup interval in seconds.
    /// (default 60, or set SESSION_CHECK_SECONDS).
    #[arg(long = "session-check-seconds", env = "SESSION_CHECK_SECONDS")]
    #[conf(default(60))]
    pub check_seconds: u64,

    /// Session inactivity timeout in seconds.
    /// (default 604800 = 7 days, or set SESSION_EXPIRY_SECONDS).
    #[arg(long = "session-expiry-seconds", env = "SESSION_EXPIRY_SECONDS")]
    #[conf(default(60480))]
    pub expiry_seconds: u64,

    /// Require HTTPS for the session cookie. Defaults to true.
    ///
    /// `Option` rather than a plain flag so the default can be *on*: a bare
    /// `--session-secure` flag could only ever turn it on, and the dangerous
    /// direction is the one that should have to be spelled out.
    ///
    /// Set false only when serving plain HTTP on something other than
    /// localhost, where a browser discards the cookie and logins silently fail
    /// to stick.
    #[arg(long = "session-secure", env = "SESSION_SECURE")]
    pub secure: Option<bool>,
}

impl SessionConfig {
    /// Secure unless explicitly turned off.
    pub fn effective_secure(&self) -> bool {
        self.secure.unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(secure: Option<bool>) -> SessionConfig {
        SessionConfig {
            check_seconds: 60,
            expiry_seconds: 60480,
            secure,
        }
    }

    #[test]
    fn session_cookies_are_secure_unless_turned_off() {
        assert!(config(None).effective_secure(), "the default must be secure");
        assert!(config(Some(true)).effective_secure());
        assert!(!config(Some(false)).effective_secure());
    }
}
