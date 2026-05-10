//! Strongly-typed identifier newtypes returned by the Alpaca API.
//!
//! Each newtype wraps an opaque server-issued string. Use the
//! constructor or `From` impls to build one from a known id (e.g. a
//! command-line argument or a database lookup); the `Display` and
//! `AsRef<str>` impls let you log or interpolate the inner value
//! without converting back to `String`.

use serde::{Deserialize, Serialize};

macro_rules! id_newtype {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Wrap an existing identifier. Accepts anything convertible
            /// into a `String`, including `&str`.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Borrow the inner string.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consume this id and return the inner `String`.
            #[must_use]
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }

        impl From<&$name> for $name {
            fn from(value: &$name) -> Self {
                value.clone()
            }
        }
    };
}

id_newtype!(AccountId, "Identifier for an Alpaca account.");
id_newtype!(OrderId, "Server-issued order identifier (UUID).");
id_newtype!(
    ClientOrderId,
    "Client-defined order identifier. Up to 128 characters; \
     supplied by callers when submitting orders."
);
id_newtype!(AssetId, "Identifier for a tradable asset.");
id_newtype!(WatchlistId, "Identifier for a watchlist.");
id_newtype!(OptionContractId, "Identifier for an option contract.");
id_newtype!(
    ActivityId,
    "Identifier for an account-activity event. Format is \
     timestamp-based (e.g. `20250507000000000::abc`), not a UUID."
);
id_newtype!(
    ExecutionId,
    "Identifier for a single order execution (fill / partial fill)."
);
