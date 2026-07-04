use crate::{AccountType, error::Error};
use std::{env, fmt};

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

/// Alpaca API credentials: a key ID and secret key pair.
///
/// Construct directly with [`ApiKey::new`] to supply credentials
/// explicitly, or let a client's `new` constructor load them from the
/// environment. Cloneable and safe to share across threads.
#[derive(Clone)]
pub struct ApiKey {
    key_id: String,
    secret_key: String,
}

impl ApiKey {
    /// Create an `ApiKey` from an explicit key ID and secret key.
    pub fn new(key_id: impl Into<String>, secret_key: impl Into<String>) -> Self {
        ApiKey {
            key_id: key_id.into(),
            secret_key: secret_key.into(),
        }
    }

    /// Load credentials from the environment for the given [`AccountType`].
    pub(crate) fn from_env(account_type: &AccountType) -> Result<ApiKey, Error> {
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
        Ok(ApiKey { key_id, secret_key })
    }

    pub(crate) fn key_id(&self) -> &str {
        &self.key_id
    }

    pub(crate) fn secret_key(&self) -> &str {
        &self.secret_key
    }
}

/// Don't print the secrets to logs on accident
impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiKey")
            .field("key_id", &CENSORED_SECRET)
            .field("secret_key", &CENSORED_SECRET)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    const PAPER_ID: &str = "test_paper_key_id";
    const PAPER_SECRET: &str = "test_paper_secret_key";
    const LIVE_ID: &str = "test_live_key_id";
    const LIVE_SECRET: &str = "test_live_secret_key";

    fn capture_env() -> (String, String, String, String) {
        (
            env::var(PAPER_KEY_ID_ENV).unwrap_or_default(),
            env::var(PAPER_SECRET_KEY_ENV).unwrap_or_default(),
            env::var(LIVE_KEY_ID_ENV).unwrap_or_default(),
            env::var(LIVE_SECRET_KEY_ENV).unwrap_or_default(),
        )
    }

    fn restore_env(keys: (String, String, String, String)) {
        // These tests are explicitly serial
        unsafe {
            env::set_var(PAPER_KEY_ID_ENV, keys.0);
            env::set_var(PAPER_SECRET_KEY_ENV, keys.1);
            env::set_var(LIVE_KEY_ID_ENV, keys.2);
            env::set_var(LIVE_SECRET_KEY_ENV, keys.3);
        }
    }

    fn set_paper_vars() {
        unsafe {
            env::set_var(PAPER_KEY_ID_ENV, PAPER_ID);
            env::set_var(PAPER_SECRET_KEY_ENV, PAPER_SECRET);
        }
    }

    fn set_live_vars() {
        unsafe {
            env::set_var(LIVE_KEY_ID_ENV, LIVE_ID);
            env::set_var(LIVE_SECRET_KEY_ENV, LIVE_SECRET);
        }
    }

    #[test]
    #[serial]
    fn test_env_correct() {
        let env = capture_env();
        set_paper_vars();
        let alpaca_env = ApiKey::from_env(&AccountType::Paper).unwrap();
        assert_eq!(alpaca_env.key_id, PAPER_ID);
        assert_eq!(alpaca_env.secret_key, PAPER_SECRET);
        set_live_vars();
        let alpaca_env = ApiKey::from_env(&AccountType::Live).unwrap();
        assert_eq!(alpaca_env.key_id, LIVE_ID);
        assert_eq!(alpaca_env.secret_key, LIVE_SECRET);
        restore_env(env);
    }

    #[test]
    #[serial]
    fn test_paper_key_not_present() {
        let env = capture_env();
        set_paper_vars();
        unsafe {
            env::remove_var(PAPER_KEY_ID_ENV);
        }
        let res = ApiKey::from_env(&AccountType::Paper);
        assert!(res.is_err());
        restore_env(env);
    }

    #[test]
    #[serial]
    fn test_paper_secret_not_present() {
        let env = capture_env();
        set_paper_vars();
        unsafe {
            env::remove_var(PAPER_SECRET_KEY_ENV);
        }
        let res = ApiKey::from_env(&AccountType::Paper);
        assert!(res.is_err());
        restore_env(env);
    }
    #[test]
    #[serial]
    fn test_live_key_id_not_present() {
        let env = capture_env();
        set_live_vars();
        unsafe {
            env::remove_var(LIVE_KEY_ID_ENV);
        }
        let res = ApiKey::from_env(&AccountType::Live);
        assert!(res.is_err());
        restore_env(env);
    }

    #[test]
    #[serial]
    fn test_live_secret_key_not_present() {
        let env = capture_env();
        set_live_vars();
        unsafe {
            env::remove_var(LIVE_SECRET_KEY_ENV);
        }
        let res = ApiKey::from_env(&AccountType::Live);
        assert!(res.is_err());
        restore_env(env);
    }

    #[test]
    #[serial_test::parallel]
    fn new_stores_supplied_credentials() {
        let key = ApiKey::new("my_key_id", "my_secret");
        assert_eq!(key.key_id(), "my_key_id");
        assert_eq!(key.secret_key(), "my_secret");
    }

    #[test]
    #[serial_test::parallel]
    fn debug_censors_both_fields() {
        let key = ApiKey::new("my_key_id", "my_secret");
        let rendered = format!("{key:?}");
        assert!(!rendered.contains("my_key_id"));
        assert!(!rendered.contains("my_secret"));
        assert!(rendered.contains("********"));
    }
}
