//! Crypto domain types shared between the REST market data API and the
//! streaming crypto feed.

use serde::{Deserialize, Serialize};

/// Side that initiated a crypto trade.
///
/// Alpaca encodes this as the single-character `tks` field on both the
/// REST and streaming trade payloads.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[non_exhaustive]
pub enum CryptoTakerSide {
    /// Buyer was the taker.
    #[serde(rename = "B")]
    Buyer,
    /// Seller was the taker.
    #[serde(rename = "S")]
    Seller,
}

#[cfg(test)]
mod tests {
    use super::CryptoTakerSide;

    #[test]
    fn taker_side_round_trips_single_character_codes() {
        for (side, encoded) in [
            (CryptoTakerSide::Buyer, "\"B\""),
            (CryptoTakerSide::Seller, "\"S\""),
        ] {
            assert_eq!(serde_json::to_string(&side).unwrap(), encoded);
            assert_eq!(
                serde_json::from_str::<CryptoTakerSide>(encoded).unwrap(),
                side
            );
        }
    }
}
