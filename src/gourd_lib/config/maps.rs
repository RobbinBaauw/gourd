use core::fmt;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::swap;
use std::ops::Deref;
use std::ops::DerefMut;

use glob::glob;
use serde::de;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use super::Input;
use super::Program;
use crate::constants::GLOB_ESCAPE;
use crate::constants::INTERNAL_GLOB;
use crate::constants::INTERNAL_POST;

/// A wrapper around [BTreeMap] to allow serde expansion of globs.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, Serialize)]
pub struct InputMap(BTreeMap<String, Input>);

/// A wrapper around [BTreeMap] with programs.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, Serialize)]
pub struct ProgramMap(BTreeMap<String, Program>);

impl<'de> Deserialize<'de> for ProgramMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// The custom map visitor for inputs.
        struct MapVisitor {
            /// Wheter to disallow internal strings.
            disallow: bool,
            /// Phantom marker.
            marker: PhantomData<()>,
        }

        impl<'de> Visitor<'de> for MapVisitor {
            type Value = ProgramMap;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map")
            }

            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = BTreeMap::new();

                while let Some((k, v)) = map.next_entry()? {
                    if self.disallow {
                        disallow_substring(&k, INTERNAL_GLOB)?;
                        disallow_substring(&k, INTERNAL_POST)?;
                    }

                    values.insert(k, v);
                }

                Ok(ProgramMap(values))
            }
        }

        let visitor = MapVisitor {
            disallow: !deserializer.is_human_readable(),
            marker: PhantomData,
        };

        deserializer.deserialize_map(visitor)
    }
}

impl<'de> Deserialize<'de> for InputMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// The custom map visitor for inputs.
        struct MapVisitor {
            /// Wheter to disallow internal strings.
            disallow: bool,
            /// Phantom marker.
            marker: PhantomData<()>,
        }

        impl<'de> Visitor<'de> for MapVisitor {
            type Value = InputMap;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map")
            }

            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = BTreeMap::new();

                while let Some((k, v)) = map.next_entry()? {
                    if self.disallow {
                        disallow_substring(&k, INTERNAL_GLOB)?;
                        disallow_substring(&k, INTERNAL_POST)?;
                    }

                    values.insert(k, v);
                }

                let expanded = expand_globs(values)?;

                Ok(InputMap(expanded))
            }
        }

        let visitor = MapVisitor {
            disallow: !deserializer.is_human_readable(),
            marker: PhantomData,
        };

        deserializer.deserialize_map(visitor)
    }
}

/// Takes the set of all inputs and expands the globbed arguments.
///
/// # Examples
/// ```toml
/// [inputs.test_input]
/// arguments = [ "=glob=/test/**/*.jpg" ]
/// ```
///
/// May get expanded to:
///
/// ```toml
/// [inputs.test_input_glob_0]
/// arguments = [ "/test/a/a.jpg" ]
///
/// [inputs.test_input_glob_1]
/// arguments = [ "/test/a/b.jpg" ]
///
/// [inputs.test_input_glob_2]
/// arguments = [ "/test/b/b.jpg" ]
/// ```
fn expand_globs<T>(inputs: BTreeMap<String, Input>) -> Result<BTreeMap<String, Input>, T>
where
    T: de::Error,
{
    let mut result = BTreeMap::new();

    for (original, input) in inputs {
        let mut globset = HashSet::new();
        globset.insert(input.clone());

        let mut is_glob = false;

        for arg_index in 0..input.arguments.len() {
            let mut next_globset = HashSet::new();

            for input_instance in &globset {
                is_glob |= explode_globset(input_instance, arg_index, &mut next_globset)?;
            }

            swap(&mut globset, &mut next_globset);
        }

        if is_glob {
            for (idx, glob) in globset.iter().enumerate() {
                result.insert(
                    format!("{}{}{}", original, INTERNAL_GLOB, idx),
                    glob.clone(),
                );
            }
        } else {
            result.insert(original, input);
        }
    }

    Ok(result)
}

/// Given a `input` and `arg_index` expand the glob at that
/// argument and put the results in `fill`.
fn explode_globset<T>(input: &Input, arg_index: usize, fill: &mut HashSet<Input>) -> Result<bool, T>
where
    T: de::Error,
{
    let arg = &input.arguments[arg_index];
    let no_escape = arg.strip_prefix(GLOB_ESCAPE);

    if let Some(globbed) = no_escape {
        for path in glob(globbed).map_err(|_| {
            de::Error::custom(format!(
                "could not expand the glob {globbed}, \
                any arguments starting with `{GLOB_ESCAPE}` are considered globs"
            ))
        })? {
            let mut glob_instance = input.clone();

            glob_instance.arguments[arg_index] = path
                .map_err(|_| {
                    de::Error::custom(format!("the glob failed to evaluate at {no_escape:?}"))
                })?
                .to_str()
                .ok_or(de::Error::custom(format!(
                    "failed to stringify {no_escape:?}"
                )))?
                .to_string();

            fill.insert(glob_instance);
        }

        Ok(true)
    } else {
        fill.insert(input.clone());
        Ok(false)
    }
}

/// Make sure that a substring is not part of a string.
fn disallow_substring<T>(name: &String, disallowed: &'static str) -> Result<(), T>
where
    T: de::Error,
{
    if name.contains(disallowed) {
        Err(de::Error::custom(format!(
            "failed to include the input {name}, \
            the input name contained `{disallowed}`, not allowed"
        )))
    } else {
        Ok(())
    }
}

impl Deref for InputMap {
    type Target = BTreeMap<String, Input>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InputMap {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.0
    }
}

impl Deref for ProgramMap {
    type Target = BTreeMap<String, Program>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProgramMap {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.0
    }
}

impl From<BTreeMap<String, Program>> for ProgramMap {
    fn from(value: BTreeMap<String, Program>) -> Self {
        ProgramMap(value)
    }
}

impl From<BTreeMap<String, Input>> for InputMap {
    fn from(value: BTreeMap<String, Input>) -> Self {
        InputMap(value)
    }
}

/// The user facing deserializer.
///
/// This may perform checks outside of just veryfing if the data is parsable.
pub struct UserDeserializer<'de>(toml::Deserializer<'de>);

impl<'de> UserDeserializer<'de> {
    /// Create a new user facing deserializer.
    pub fn new(s: &'de str) -> Self {
        UserDeserializer(toml::Deserializer::new(s))
    }
}

impl<'de> Deserializer<'de> for UserDeserializer<'de> {
    type Error = toml::de::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_any(visitor)
    }

    fn is_human_readable(&self) -> bool {
        false
    }

    // This passes all other deserialization functions to the `toml` deserializer.
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
