use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;

use serde::de;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use crate::config::maps::IS_USER_FACING;
use crate::constants::URL_ESCAPE;

/// A wrapper around [PathBuf] to allow serde expansion of globs.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, Serialize)]
pub struct FetchedPath(pub PathBuf);

impl<'de> Deserialize<'de> for FetchedPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// The custom map visitor for inputs.
        struct FetchedVisitor {
            /// Phantom marker.
            marker: PhantomData<()>,
        }

        impl<'de> Visitor<'de> for FetchedVisitor {
            type Value = FetchedPath;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid path, in the form of a string")
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                // if let Some(actual) = v.strip_prefix(URL_ESCAPE) {
                //   let url = actual.split('>').next().ok_or(de::Error::custom("invalid url
                // syntax, expected '[url] > [file]'"))?;
                //
                //
                //
                // } else {
                //   Ok(v.parse())
                //
                // }
                //
                Ok(FetchedPath(PathBuf::from_str(v)
                    .map_err(|x| de::Error::custom(format!("could not include the path: {x}")))?))
            }
        }

        let visitor = FetchedVisitor {
            marker: PhantomData,
        };

        if IS_USER_FACING.with_borrow(|x| *x) {
            deserializer.deserialize_str(visitor)
        } else {
            Ok(FetchedPath(PathBuf::deserialize(deserializer)?))
        }
    }
}

impl Deref for FetchedPath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
