//

use std::env;

/// The environment variable containing the Alpaca paper account key ID.
const PAPER_KEY_ID_ENV: &str = "ALPACA_PAPER_API_KEY_ID";
/// The environment variable containing the Alpaca paper account secret key.
const PAPER_SECRET_KEY_ENV: &str = "ALPACA_PAPER_API_SECRET_KEY";
/// The environment variable containing the Alpaca live account key ID.
const LIVE_KEY_ID_ENV: &str = "ALPACA_LIVE_API_KEY_ID";
/// The environment variable containing the Alpaca live account secret key.
const LIVE_SECRET_KEY_ENV: &str = "ALPACA_LIVE_API_SECRET_KEY";

/// The type of Alpaca account.
#[derive(Debug, Eq, PartialEq)]
pub enum AlpacaAccountType {
    /// Paper trading account.
    Paper,
    /// Live trading account.
    Live,
}

pub struct AlpacaEnv {
    pub account_type: AlpacaAccountType,
    pub key_id: String,
    pub secret_key: String,
}

impl AlpacaEnv {
    pub fn new(account_type: AlpacaAccountType) -> AlpacaEnv {
        let env_keys = match account_type {
            AlpacaAccountType::Paper => (PAPER_KEY_ID_ENV, PAPER_SECRET_KEY_ENV),
            AlpacaAccountType::Live => (LIVE_KEY_ID_ENV, LIVE_SECRET_KEY_ENV),
        };
        let key_id = env::var(env_keys.0)
            .unwrap_or_else(|_| panic!("Missing Alpaca API key ID, please set: {}", env_keys.0));
        let secret_key = env::var(env_keys.1)
            .unwrap_or_else(|_| panic!("Missing Alpaca secret key, please set: {}", env_keys.1));
        AlpacaEnv {
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
    fn set_paper_vars() {
        env::set_var(PAPER_KEY_ID_ENV, "test_paper_key_id");
        env::set_var(PAPER_SECRET_KEY_ENV, "test_paper_secret_key");
    }

    fn set_live_vars() {
        env::set_var(LIVE_KEY_ID_ENV, "test_live_key_id");
        env::set_var(LIVE_SECRET_KEY_ENV, "test_live_secret_key");
    }

    #[test]
    #[serial]
    fn test_env_correct() {
        set_paper_vars();
        let alpaca_env = AlpacaEnv::new(AlpacaAccountType::Paper);
        assert_eq!(alpaca_env.account_type, AlpacaAccountType::Paper);
        assert_eq!(alpaca_env.key_id, "test_paper_key_id");
        assert_eq!(alpaca_env.secret_key, "test_paper_secret_key");
        set_live_vars();
        let alpaca_env = AlpacaEnv::new(AlpacaAccountType::Live);
        assert_eq!(alpaca_env.account_type, AlpacaAccountType::Live);
        assert_eq!(alpaca_env.key_id, "test_live_key_id");
        assert_eq!(alpaca_env.secret_key, "test_live_secret_key");
    }

    #[test]
    #[serial]
    #[should_panic]
    fn test_paper_key_not_present() {
        set_paper_vars();
        env::remove_var(PAPER_KEY_ID_ENV);
        AlpacaEnv::new(AlpacaAccountType::Paper);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn test_paper_secret_not_present() {
        set_paper_vars();
        env::remove_var(PAPER_SECRET_KEY_ENV);
        AlpacaEnv::new(AlpacaAccountType::Paper);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn test_live_key_id_not_present() {
        set_live_vars();
        env::remove_var(LIVE_KEY_ID_ENV);
        AlpacaEnv::new(AlpacaAccountType::Live);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn test_live_secret_key_not_present() {
        set_live_vars();
        env::remove_var(LIVE_SECRET_KEY_ENV);
        AlpacaEnv::new(AlpacaAccountType::Live);
    }
}
