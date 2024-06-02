use core::fmt;
use std::time::Duration;

use serde::de::Visitor;
use serde::Deserializer;

/// Deserializing duration from a human-readable string.
pub fn deserialize_human_time_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    struct DurationVisitor;

    impl<'de> Visitor<'de> for DurationVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a valid duration string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            humantime::parse_duration(v).map_err(|parse_err| {
                serde::de::Error::custom(format!("This is not a valid duration: {parse_err}"))
            })
        }
    }

    deserializer.deserialize_str(DurationVisitor)
}
