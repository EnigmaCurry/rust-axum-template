//! `set-password` — give a local account a bcrypt password.
//!
//! Needed because `migrations/0002_root_user.sql` creates `root` with no
//! password row at all, so a fresh install has no way to log in under
//! `--auth-method username_password`.
//!
//! The password is never taken as a command-line argument: that would put it in
//! shell history and in `ps` output for every user on the box. It is read from
//! a hidden prompt, or from stdin when piped.

use std::io::{BufRead, IsTerminal, Write};
use std::path::PathBuf;

use crate::{
    config::{SetPasswordConfig, database::build_db_url},
    errors::CliError,
    models::user,
};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

/// bcrypt silently ignores bytes past 72, which would make a long password
/// quietly weaker than it looks. Reject instead.
const MAX_PASSWORD_BYTES: usize = 72;
const MIN_PASSWORD_CHARS: usize = 8;

pub fn set_password<W1: Write, W2: Write>(
    args: SetPasswordConfig,
    root_dir: PathBuf,
    out: &mut W1,
    _err: &mut W2,
) -> Result<(), CliError> {
    let db_url = build_db_url(args.database.url.clone(), &root_dir);
    let username = args.username.clone();

    let password = read_password(&username, args.stdin)?;
    validate_password(&password)?;

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| CliError::RuntimeError(format!("failed to start Tokio runtime: {e}")))?;

    runtime.block_on(apply(&db_url, &username, &password))?;

    writeln!(out, "Set the password for '{username}'.")?;
    Ok(())
}

async fn apply(db_url: &str, username: &str, password: &str) -> Result<(), CliError> {
    use std::str::FromStr;

    // Same options as `serve`, including migrations: this command has to work
    // on a fresh install, before the server has ever run.
    let options = SqliteConnectOptions::from_str(db_url)
        .map_err(|e| CliError::RuntimeError(format!("bad database url '{db_url}': {e}")))?
        .create_if_missing(true)
        .foreign_keys(true);

    let db = SqlitePool::connect_with(options)
        .await
        .map_err(|e| CliError::RuntimeError(format!("could not open {db_url}: {e}")))?;

    sqlx::migrate!()
        .run(&db)
        .await
        .map_err(|e| CliError::RuntimeError(format!("migrations failed: {e}")))?;

    let user = user::select_user_by_username(&db, username)
        .await
        .map_err(|e| CliError::RuntimeError(format!("could not look up '{username}': {e}")))?
        .ok_or_else(|| {
            CliError::InvalidArgs(format!(
                "no user named '{username}'. Existing usernames can be listed with: \
                 {} sql",
                env!("CARGO_BIN_NAME")
            ))
        })?;

    let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| CliError::RuntimeError(format!("failed to hash the password: {e}")))?;

    sqlx::query(
        r#"
        INSERT INTO user_password (user_id, password_hash)
        VALUES (?1, ?2)
        ON CONFLICT(user_id) DO UPDATE SET
            password_hash = excluded.password_hash,
            updated_at    = CURRENT_TIMESTAMP,
            must_reset    = 0
        "#,
    )
    .bind(user.id.0)
    .bind(&hash)
    .execute(&db)
    .await
    .map_err(|e| CliError::RuntimeError(format!("could not store the password: {e}")))?;

    db.close().await;
    Ok(())
}

/// Prompt with echo off, or read one line from stdin when piped.
fn read_password(username: &str, force_stdin: bool) -> Result<String, CliError> {
    if force_stdin || !std::io::stdin().is_terminal() {
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line)?;
        // Only strip the line ending; a password may legitimately end in a space.
        return Ok(line.trim_end_matches(['\n', '\r']).to_string());
    }

    let password = rpassword::prompt_password(format!("New password for '{username}': "))
        .map_err(|e| CliError::RuntimeError(format!("could not read the password: {e}")))?;
    let confirm = rpassword::prompt_password("Confirm password: ")
        .map_err(|e| CliError::RuntimeError(format!("could not read the password: {e}")))?;

    if password != confirm {
        return Err(CliError::InvalidArgs(
            "the passwords did not match".to_string(),
        ));
    }

    Ok(password)
}

fn validate_password(password: &str) -> Result<(), CliError> {
    if password.is_empty() {
        return Err(CliError::InvalidArgs("the password was empty".to_string()));
    }

    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(CliError::InvalidArgs(format!(
            "the password must be at least {MIN_PASSWORD_CHARS} characters"
        )));
    }

    if password.len() > MAX_PASSWORD_BYTES {
        return Err(CliError::InvalidArgs(format!(
            "the password is {} bytes; bcrypt ignores anything past {MAX_PASSWORD_BYTES}, \
             so use a shorter one rather than a silently truncated one",
            password.len()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_short_passwords_are_rejected() {
        assert!(validate_password("").is_err());
        assert!(validate_password("short").is_err());
        assert!(validate_password("exactly8").is_ok());
    }

    #[test]
    fn passwords_past_bcrypts_limit_are_rejected_not_truncated() {
        let at_limit = "a".repeat(MAX_PASSWORD_BYTES);
        assert!(validate_password(&at_limit).is_ok());

        let over = "a".repeat(MAX_PASSWORD_BYTES + 1);
        let err = validate_password(&over).expect_err("should be rejected");
        assert!(format!("{err}").contains("bcrypt ignores"), "{err}");
    }

    #[test]
    fn the_limit_is_counted_in_bytes_not_characters() {
        // 24 three-byte characters: 24 chars, 72 bytes — right at the limit.
        let multibyte = "ꙮ".repeat(24);
        assert_eq!(multibyte.len(), 72);
        assert!(validate_password(&multibyte).is_ok());

        // One more character is 75 bytes, which bcrypt would truncate.
        let over = "ꙮ".repeat(25);
        assert!(validate_password(&over).is_err());
    }

    #[test]
    fn a_hashed_password_verifies() {
        let hash = bcrypt::hash("correct horse", bcrypt::DEFAULT_COST).unwrap();
        assert!(bcrypt::verify("correct horse", &hash).unwrap());
        assert!(!bcrypt::verify("wrong horse", &hash).unwrap());
    }
}
