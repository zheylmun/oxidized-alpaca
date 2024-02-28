pub(crate) mod rest_client;
pub(crate) mod streaming_client;
pub(crate) mod vexpand;
use serde::{Deserialize, Deserializer};

pub use rest_client::RestClient;
pub use streaming_client::StreamingClient;

pub(crate) fn null_def_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt = Option::<Vec<T>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}
