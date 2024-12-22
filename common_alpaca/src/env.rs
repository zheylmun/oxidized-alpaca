//! The `Env` module loads and stores required information about the Alpaca Environment

use std::{env, fmt};

use crate::{error::Error, AccountType};

/// The environment variable containing the Alpaca paper account key ID
const PAPER_KEY_ID_ENV: &str = "ALPACA_PAPER_API_KEY_ID";
/// The environment variable containing the Alpaca paper account secret key
const PAPER_SECRET_KEY_ENV: &str = "ALPACA_PAPER_API_SECRET_KEY";
/// The environment variable containing the Alpaca live account key ID
const LIVE_KEY_ID_ENV: &str = "ALPACA_LIVE_API_KEY_ID";
/// The environment variable containing the Alpaca live account secret key
const LIVE_SECRET_KEY_ENV: &str = "ALPACA_LIVE_API_SECRET_KEY";
/// Debug value for sensitive information
const CENSORED_SECRET: &str = "********";

/// `Env` loads and stores the required information about the Alpaca Environment
#[derive(Clone)]
pub struct Env {
    /// The Alpaca API key ID
    key_id: String,
    /// The Alpaca secret key
    secret_key: String,
}

impl Env {
    /// Attempt to create a new `Env` instance with the given [`AccountType`]
    pub fn new(account_type: &AccountType) -> Result<Env, Error> {
        let env_keys = match account_type {
            AccountType::Paper => (PAPER_KEY_ID_ENV, PAPER_SECRET_KEY_ENV),
            AccountType::Live => (LIVE_KEY_ID_ENV, LIVE_SECRET_KEY_ENV),
        };
        let key_id = env::var(env_keys.0).map_err(|e| Error::MissingEnvironmentVariable {
            variable_name: env_keys.0.to_string(),
            source: e,
        })?;
        let secret_key = env::var(env_keys.1).map_err(|e| Error::MissingEnvironmentVariable {
            variable_name: env_keys.1.to_string(),
            source: e,
        })?;
        Ok(Env { key_id, secret_key })
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    pub fn secret_key(&self) -> &str {
        &self.secret_key
    }
}

/// Don't print the secrets to logs on accident
impl fmt::Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Env")
            .field("key_id", &CENSORED_SECRET)
            .field("secret_key", &CENSORED_SECRET)
            .finish()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    const PAPER_ID: &str = "test_paper_key_id";
    const PAPER_SECRET: &str = "test_paper_secret_key";
    const LIVE_ID: &str = "test_live_key_id";
    const LIVE_SECRET: &str = "test_live_secret_key";
    fn set_paper_vars() {
        env::set_var(PAPER_KEY_ID_ENV, PAPER_ID);
        env::set_var(PAPER_SECRET_KEY_ENV, PAPER_SECRET);
    }

    fn set_live_vars() {
        env::set_var(LIVE_KEY_ID_ENV, LIVE_ID);
        env::set_var(LIVE_SECRET_KEY_ENV, LIVE_SECRET);
    }
    #[test]
    fn test_env_correct() {
        set_paper_vars();
        let alpaca_env = Env::new(&AccountType::Paper).unwrap();
        assert_eq!(alpaca_env.key_id, PAPER_ID);
        assert_eq!(alpaca_env.secret_key, PAPER_SECRET);
        set_live_vars();
        let alpaca_env = Env::new(&AccountType::Live).unwrap();
        assert_eq!(alpaca_env.key_id, LIVE_ID);
        assert_eq!(alpaca_env.secret_key, LIVE_SECRET);
    }

    #[test]
    fn test_paper_key_not_present() {
        set_paper_vars();
        env::remove_var(PAPER_KEY_ID_ENV);
        let res = Env::new(&AccountType::Paper);
        assert!(res.is_err());
    }

    #[test]
    fn test_paper_secret_not_present() {
        set_paper_vars();
        env::remove_var(PAPER_SECRET_KEY_ENV);
        let res = Env::new(&AccountType::Paper);
        assert!(res.is_err());
    }

    #[test]
    fn test_live_key_id_not_present() {
        set_live_vars();
        env::remove_var(LIVE_KEY_ID_ENV);
        let res = Env::new(&AccountType::Live);
        assert!(res.is_err());
    }

    #[test]
    fn test_live_secret_key_not_present() {
        set_live_vars();
        env::remove_var(LIVE_SECRET_KEY_ENV);
        let res = Env::new(&AccountType::Live);
        assert!(res.is_err());
    }
}
