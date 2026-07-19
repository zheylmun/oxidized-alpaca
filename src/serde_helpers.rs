//! Shared serde helpers used by both REST and streaming types.

#[cfg(feature = "restful")]
use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};

pub(crate) fn string_as_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse()
        .map_err(serde::de::Error::custom)
}

pub(crate) fn string_as_optional_decimal<'de, D>(
    deserializer: D,
) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => s.parse().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

pub(crate) fn null_def_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt = Option::<Vec<T>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[cfg(feature = "restful")]
pub(crate) fn optional_decimal_as_string<S>(
    value: &Option<Decimal>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(d) => serializer.serialize_str(&d.to_string()),
        None => serializer.serialize_none(),
    }
}

#[cfg(feature = "restful")]
pub(crate) fn decimal_as_string<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

#[cfg(feature = "restful")]
pub(crate) fn string_as_optional_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => s.parse().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

/// Deserialize an optional integer that some Alpaca crypto feeds encode as
/// a JSON string instead of a number (the `i` trade-id field).
#[cfg(feature = "restful")]
pub(crate) fn string_or_int_as_optional_i64<'de, D>(
    deserializer: D,
) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        Int(i64),
        Str(String),
    }

    match Option::<StringOrInt>::deserialize(deserializer)? {
        Some(StringOrInt::Int(i)) => Ok(Some(i)),
        Some(StringOrInt::Str(s)) => s.parse().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

#[cfg(all(test, feature = "restful"))]
mod tests {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Wrapper {
        #[serde(default, deserialize_with = "super::string_or_int_as_optional_i64")]
        id: Option<i64>,
    }

    #[test]
    fn accepts_int_string_null_and_missing() {
        for (json, expected) in [
            (r#"{"id": 42}"#, Some(42)),
            (r#"{"id": "42"}"#, Some(42)),
            (r#"{"id": -7}"#, Some(-7)),
            (r#"{"id": null}"#, None),
            (r#"{}"#, None),
        ] {
            let parsed: Wrapper = serde_json::from_str(json).unwrap();
            assert_eq!(parsed.id, expected, "{json}");
        }
    }

    /// A non-numeric string is a genuine protocol violation, so it must
    /// surface as a deserialization error rather than silently becoming
    /// `None`.
    #[test]
    fn rejects_non_numeric_string() {
        let err = serde_json::from_str::<Wrapper>(r#"{"id": "not-a-number"}"#).unwrap_err();
        assert!(
            err.to_string().contains("invalid digit"),
            "unexpected error: {err}"
        );
    }
}

#[cfg(feature = "restful")]
pub(crate) fn unix_seconds_vec_as_datetimes<'de, D>(
    deserializer: D,
) -> Result<Vec<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Vec::<i64>::deserialize(deserializer)?;
    raw.into_iter()
        .map(|secs| {
            Utc.timestamp_opt(secs, 0)
                .single()
                .ok_or_else(|| serde::de::Error::custom(format!("invalid unix timestamp: {secs}")))
        })
        .collect()
}
