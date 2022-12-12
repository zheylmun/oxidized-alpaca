//

use std::env;

/// The environment variable containing the Alpaca paper account key ID.
const PAPER_KEY_ID_ENV: &str = "ALPACA_PAPER_API_KEY_ID";
/// The environment variable containing the Alpaca paper account secret key.
const PAPER_SECRET_KEY_ENV: &str = "ALPACA_PAPER_SECRET_KEY";
/// The environment variable containing the Alpaca live account key ID.
const LIVE_KEY_ID_ENV: &str = "ALPACA_LIVE_API_KEY_ID";
/// The environment variable containing the Alpaca live account secret key.
const LIVE_SECRET_KEY_ENV: &str = "ALPACA_LIVE_SECRET_KEY";

pub enum AlpacaAccountType {
    Paper,
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

    #[test]
    fn test_env_correct() {
        AlpacaEnv::new(AlpacaAccountType::Paper);
    }

    #[test]
    #[should_panic]
    fn test_paper_key_not_present() {
        env::remove_var(PAPER_KEY_ID_ENV);
        AlpacaEnv::new(AlpacaAccountType::Paper);
    }

    #[test]
    #[should_panic]
    fn test_paper_secret_not_present() {
        env::remove_var(PAPER_SECRET_KEY_ENV);
        AlpacaEnv::new(AlpacaAccountType::Paper);
    }

    #[test]
    #[should_panic]
    fn test_live_key_id_not_present() {
        env::remove_var(LIVE_KEY_ID_ENV);
        AlpacaEnv::new(AlpacaAccountType::Live);
    }

    #[test]
    #[should_panic]
    fn test_live_secret_key_not_present() {
        env::remove_var(LIVE_SECRET_KEY_ENV);
        AlpacaEnv::new(AlpacaAccountType::Live);
    }
}
