//! Reject boolean environment variables that are neither true nor false.
//!
//! The config parser treats any value it does not recognise as `true`, so a
//! typo in a security-relevant setting silently turns the dangerous direction
//! on:
//!
//! ```text
//! $ ${APP_PREFIX}AUTH_TRUSTED_FORWARDED_FOR=flase app config
//! trusted_forwarded_for = true
//! ```
//!
//! Nothing looks wrong afterwards — the app starts, serves, and simply trusts
//! a client-supplied header it should not. Failing open is the wrong direction
//! for a switch like that, so a value we cannot read as a boolean stops
//! startup instead.

use crate::errors::CliError;

/// Env var prefix this binary reads its settings under.
pub const ENV_PREFIX: &str = "${APP_PREFIX}";

/// Settings whose value is a boolean, by env var suffix.
///
/// Keep this in sync with the `bool` and `Option<bool>` fields in this module's
/// siblings. A missing entry is not a crash, just a setting that keeps the old
/// fail-open behaviour.
const BOOLEAN_SETTINGS: &[&str] = &[
    "AUTH_TRUSTED_FORWARDED_FOR",
    "SESSION_SECURE",
    "TLS_SELF_SIGNED_EPHEMERAL",
];

/// Spellings we are willing to act on. Empty is allowed because an unset-but-
/// present variable is how shells spell "leave it alone".
const BOOL_WORDS: &[&str] = &["", "true", "false", "1", "0", "yes", "no", "on", "off"];

/// Check every boolean setting present in the environment.
pub fn validate() -> Result<(), CliError> {
    for suffix in BOOLEAN_SETTINGS {
        let name = format!("{ENV_PREFIX}{suffix}");
        // A non-UTF-8 value never reached the parser either; leave it be.
        if let Ok(raw) = std::env::var(&name) {
            check(&name, &raw)?;
        }
    }
    Ok(())
}

fn check(name: &str, raw: &str) -> Result<(), CliError> {
    if BOOL_WORDS.contains(&raw.trim().to_ascii_lowercase().as_str()) {
        return Ok(());
    }

    Err(CliError::InvalidArgs(format!(
        "{name}='{raw}' is not a boolean. Use one of: true, false, 1, 0, yes, no, on, off. \
         Refusing to guess, because an unrecognised value would otherwise be read as true."
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn booleans_in_any_reasonable_spelling_are_accepted() {
        for ok in [
            "true", "false", "1", "0", "yes", "no", "on", "off", "TRUE", "False", " true ", "",
        ] {
            assert!(check("X", ok).is_ok(), "{ok:?} should be accepted");
        }
    }

    #[test]
    fn anything_else_is_rejected_rather_than_read_as_true() {
        for bad in ["flase", "nonsense", "2", "y", "disabled"] {
            let message = check("X", bad)
                .expect_err("should be rejected")
                .to_string();
            assert!(message.contains("is not a boolean"), "{message}");
            assert!(message.contains(bad), "the error should quote the value: {message}");
        }
    }

    #[test]
    fn every_listed_setting_is_prefixed() {
        // The suffixes are bare on purpose; the prefix is added at check time.
        for suffix in BOOLEAN_SETTINGS {
            assert!(
                !suffix.starts_with(ENV_PREFIX),
                "{suffix} should not carry the prefix itself"
            );
        }
    }
}
