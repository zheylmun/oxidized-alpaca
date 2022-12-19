//! The `Env` module loads and stores the required information about the Alpaca Environment

use std::env;

/// The environment variable containing the Alpaca paper account key ID
const PAPER_KEY_ID_ENV: &str = "ALPACA_PAPER_API_KEY_ID";
/// The environment variable containing the Alpaca paper account secret key
const PAPER_SECRET_KEY_ENV: &str = "ALPACA_PAPER_API_SECRET_KEY";
/// The environment variable containing the Alpaca live account key ID
const LIVE_KEY_ID_ENV: &str = "ALPACA_LIVE_API_KEY_ID";
/// The environment variable containing the Alpaca live account secret key
const LIVE_SECRET_KEY_ENV: &str = "ALPACA_LIVE_API_SECRET_KEY";

/// The type of Alpaca account
#[derive(Debug, Eq, PartialEq)]
pub enum AccountType {
    /// Paper trading account
    Paper,
    /// Live trading account
    Live,
}

/// `Env` loads and stores the required information about the Alpaca Environment
pub struct Env {
    /// The type of Alpaca account (paper or live)
    pub account_type: AccountType,
    /// The Alpaca API key ID
    pub key_id: String,
    /// The Alpaca secret key
    pub secret_key: String,
}

impl Env {
    /// Attempt to create a new `Env` instance with the given [`AccountType`]
    ///
    /// # Panics
    /// Panics if the required environment variables are not set
    #[must_use]
    pub fn new(account_type: AccountType) -> Env {
        let env_keys = match account_type {
            AccountType::Paper => (PAPER_KEY_ID_ENV, PAPER_SECRET_KEY_ENV),
            AccountType::Live => (LIVE_KEY_ID_ENV, LIVE_SECRET_KEY_ENV),
        };
        let key_id = env::var(env_keys.0)
            .unwrap_or_else(|_| panic!("Missing Alpaca API key ID, please set: {}", env_keys.0));
        let secret_key = env::var(env_keys.1)
            .unwrap_or_else(|_| panic!("Missing Alpaca secret key, please set: {}", env_keys.1));
        Env {
            account_type,
            key_id,
            secret_key,
        }
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
    fn set_paper_vars() {
        env::set_var(PAPER_KEY_ID_ENV, PAPER_ID);
        env::set_var(PAPER_SECRET_KEY_ENV, PAPER_SECRET);
    }

    fn set_live_vars() {
        env::set_var(LIVE_KEY_ID_ENV, LIVE_ID);
        env::set_var(LIVE_SECRET_KEY_ENV, LIVE_SECRET);
    }

    #[test]
    #[serial]
    fn test_env_correct() {
        set_paper_vars();
        let alpaca_env = Env::new(AccountType::Paper);
        assert_eq!(alpaca_env.account_type, AccountType::Paper);
        assert_eq!(alpaca_env.key_id, PAPER_ID);
        assert_eq!(alpaca_env.secret_key, PAPER_SECRET);
        set_live_vars();
        let alpaca_env = Env::new(AccountType::Live);
        assert_eq!(alpaca_env.account_type, AccountType::Live);
        assert_eq!(alpaca_env.key_id, LIVE_ID);
        assert_eq!(alpaca_env.secret_key, LIVE_SECRET);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn test_paper_key_not_present() {
        set_paper_vars();
        env::remove_var(PAPER_KEY_ID_ENV);
        let env = Env::new(AccountType::Paper);
        assert_eq!(env.account_type, AccountType::Paper);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn test_paper_secret_not_present() {
        set_paper_vars();
        env::remove_var(PAPER_SECRET_KEY_ENV);
        let env = Env::new(AccountType::Paper);
        assert_eq!(env.account_type, AccountType::Paper);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn test_live_key_id_not_present() {
        set_live_vars();
        env::remove_var(LIVE_KEY_ID_ENV);
        let env = Env::new(AccountType::Live);
        assert_eq!(env.account_type, AccountType::Live);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn test_live_secret_key_not_present() {
        set_live_vars();
        env::remove_var(LIVE_SECRET_KEY_ENV);
        let env = Env::new(AccountType::Live);
        assert_eq!(env.account_type, AccountType::Live);
    }
}
