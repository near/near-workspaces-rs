///! Utility module used for deserializing data coming from contracts.

/// Deserializer used in convention with serde to deserialize objects that are
/// from string into a more concrete type. For example, if the contract returns
/// a [`U128`] type, we do not need to directly import `U128` from near_sdk, and
/// instead can add tell serde to deserialize it using this module like so:
/// ```no_run
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct DeserializableStruct {
///     #[serde(with = "workspaces::serde::str")]
///     value: u128,
/// }
/// ```
///
/// [`U128`]: https://docs.rs/near-sdk/latest/near_sdk/json_types/struct.U128.html
pub mod str {
    use serde::{de, Deserialize, Deserializer, Serializer};
    use std::fmt::Display;
    use std::str::FromStr;

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
