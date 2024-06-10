use core::fmt;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::env::current_dir;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::swap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use glob::glob;
use regex_lite::Regex;
use serde::de;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use super::Input;
use super::Parameter;
use super::Program;
use crate::bailc;
use crate::constants::GLOB_ESCAPE;
use crate::constants::INTERNAL_GLOB;
use crate::constants::INTERNAL_PREFIX;
use crate::ctx;
use crate::error::Ctx;
use crate::file_system::FileOperations;
use crate::file_system::FileSystemInteractor;

// Q: Why is this done like this? This pattern seems to be harmful.
// A: There was a lot of invesigation into other ways of solving the problem.
// In short: We need DeserializeSeed but this is impossible without seriously
// polluting the codebase.
//
// In long: See: https://github.com/serde-rs/serde/issues/881
// And: https://github.com/Marwes/serde_state/issues/8#issuecomment-904697217
// And the state of: https://github.com/Marwes/serde_state
//
// Thus this solution was chosen.
thread_local! {
  pub(crate) static IS_USER_FACING: RefCell<bool> = const { RefCell::new(false) };
}

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
                let fs = FileSystemInteractor { dry_run: false };
                let mut values = BTreeMap::new();

                while let Some((k, mut v)) = map.next_entry::<String, Program>()? {
                    if IS_USER_FACING.with_borrow(|x| *x) {
                        v.binary = canon_path(&v.binary, fs)?;

                        if let Some(relative) = v.afterscript.clone() {
                            v.afterscript = Some(canon_path(&relative, fs)?);
                        }

                        disallow_substring(&k, INTERNAL_PREFIX)?;
                    }

                    values.insert(k, v);
                }

                Ok(ProgramMap(values))
            }
        }

        let visitor = MapVisitor {
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
                let fs = FileSystemInteractor { dry_run: false };
                let mut values = BTreeMap::new();

                while let Some((k, mut v)) = map.next_entry::<String, Input>()? {
                    if IS_USER_FACING.with_borrow(|x| *x) {
                        if let Some(relative) = v.input.clone() {
                            v.input = Some(canon_path(&relative, fs)?);
                        }

                        disallow_substring(&k, INTERNAL_PREFIX)?;
                    }

                    values.insert(k, v);
                }

                let expanded = expand_globs(values)?;

                Ok(InputMap(expanded))
            }
        }

        let visitor = MapVisitor {
            marker: PhantomData,
        };

        deserializer.deserialize_map(visitor)
    }
}

/// This will take a path and canonicalize it.
fn canon_path<T>(path: &Path, fs: impl FileOperations) -> Result<std::path::PathBuf, T>
where
    T: de::Error,
{
    fs.canonicalize(path).map_err(|_| {
        de::Error::custom(format!(
            "failed to find {:?} relative to {:?}",
            path,
            current_dir().unwrap()
        ))
    })
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
                    format!("{}{}{}{}", original, INTERNAL_PREFIX, INTERNAL_GLOB, idx),
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

/// Takes the set of all inputs and all Parameters and expands parameterd
/// arguments in the inputs with valeus of provided parameters.
///
/// # Examples
///
/// ```toml
/// [parameters.x.sub.a]
/// values = ["1", "2"]
///
/// [parameters.x.sub.b]
/// values = ["15", "60"]
///
/// [parameters.y]
/// values = ["a", "b"]
///
/// [programs.test_program]
/// binary = "test"
///
/// [inputs.test_input]
/// arguments = [ "const", "parameter|{parameter_x_a}", "parameter|{parameter_y}", "parameter|{parameter_x_b}" ]
/// ```
///
/// Will get expanded to:
/// ```toml
/// [inputs.test_input_x-0_y-0]
/// arguments = [ "const", "1", "a", "15" ]
///
/// [inputs.test_input_x-1_y-0]
/// arguments = [ "const", "2", "a", "60" ]
///
/// [inputs.test_input_x-0_y-1]
/// arguments = [ "const", "1", "b", "15" ]
///
/// [inputs.test_input_x-1_y-1]
/// arguments = [ "const", "2", "b", "60" ]
/// ```
pub fn expand_parameters(
    inputs: InputMap,
    parameters: &BTreeMap<String, Parameter>,
) -> anyhow::Result<InputMap> {
    let mut result: BTreeMap<String, Input> = BTreeMap::new();
    let mut parameter_names_encountered: BTreeSet<String> = BTreeSet::new();

    check_subparameter_size_is_equal(parameters)?;

    for (input_name, input) in inputs.iter() {
        let mut map: BTreeMap<String, Vec<(usize, String)>> = BTreeMap::new();
        let mut expandable_parameters: BTreeSet<String> = BTreeSet::new();

        // Find uses of parameters in inputs.
        get_expandable_parameters(
            input,
            &mut map,
            &mut expandable_parameters,
            &mut parameter_names_encountered,
        )?;

        // If none of parameters was used in this input then there's no need to do
        // anything.
        if expandable_parameters.is_empty() {
            result.insert(input_name.clone(), input.clone());
            continue;
        }

        let mut set: BTreeSet<(String, Vec<String>)> = BTreeSet::new();
        set.insert((input_name.clone(), input.arguments.clone()));

        for parameter_name in expandable_parameters {
            let param = parameters.get(&parameter_name).with_context(
                ctx!("Did not find values for parameter specified in input {}",input_name;"{}",parameter_name),
            )?;
            let indexes = &map[&parameter_name];

            if indexes[0].1 == *""
            // Parameters have empty string in indexes
            {
                expand_parameter(&parameter_name, param, &mut set, indexes)?;
            } else {
                expand_subparameter(&parameter_name, param, &mut set, indexes)?;
            }
        }

        for (name, x) in set {
            let mut input_copy = input.clone();
            input_copy.arguments.clone_from(&x);
            result.insert(name, input_copy);
        }
    }

    // Make sure every parameter specified was used at least once.
    check_parameters_were_used(parameters, &parameter_names_encountered)?;

    Ok(InputMap(result))
}

/// Checks if all subparameters of each paramterers specified in `parameters`
/// are equal (Helper function)
fn check_subparameter_size_is_equal(
    parameters: &BTreeMap<String, Parameter>,
) -> anyhow::Result<()> {
    for (parameter_name, parameter) in parameters {
        if let Some(sub_parameters) = &parameter.sub {
            let sub_parameter_size = sub_parameters
                .clone()
                .first_entry()
                .unwrap()
                .get()
                .values
                .len();
            for x in sub_parameters.values() {
                if x.values.len() != sub_parameter_size {
                    bailc!(
                        "Subparameter sizes don't match", ;
                        "{}", parameter_name;
                        "",
                    );
                }
            }
        }
    }
    Ok(())
}

/// Gets names and positions of parameters in the provided `input`. (Helper
/// function)
///
/// Saves paramter names in `expandable_parameters` Set and
/// `parameter_names_encountered` Set
///
/// Saves map of Index to Parameter name in `map`
fn get_expandable_parameters(
    input: &Input,
    map: &mut BTreeMap<String, Vec<(usize, String)>>,
    expandable_parameters: &mut BTreeSet<String>,
    parameter_names_encountered: &mut BTreeSet<String>,
) -> anyhow::Result<()> {
    for (pos, arg) in input.arguments.iter().enumerate() {
        if arg.starts_with("parameter|") {
            let regex_solo = Regex::new(r"^.*\{parameter_([0-9a-zA-Z]*)\}.*$").unwrap();
            let regex_sub =
                Regex::new(r"^.*\{parameter_([0-9a-zA-Z]*)_([0-9a-zA-Z]*)\}.*$").unwrap();

            if let Some(captures) = regex_solo.captures(arg) {
                let param_name = captures[1].to_string();
                expandable_parameters.insert(param_name.clone());

                if let Some(vector) = map.get_mut(&param_name) {
                    vector.push((pos, "".to_string()))
                } else {
                    map.insert(param_name, vec![(pos, "".to_string())]);
                }
            }

            if let Some(captures) = regex_sub.captures(arg) {
                let param_name = captures[1].to_string();
                expandable_parameters.insert(param_name.clone());
                parameter_names_encountered.insert(param_name.clone() + "_" + &captures[2]);

                if let Some(vector) = map.get_mut(&param_name) {
                    vector.push((pos, captures[2].to_string()))
                } else {
                    map.insert(param_name, vec![(pos, captures[2].to_string())]);
                }
            }
        }
    }

    parameter_names_encountered.append(&mut expandable_parameters.clone());

    Ok(())
}

/// Expands provided parameter (Helper function)
fn expand_parameter(
    parameter_name: &String,
    param: &Parameter,
    set: &mut BTreeSet<(String, Vec<String>)>,
    indexes: &Vec<(usize, String)>,
) -> anyhow::Result<()> {
    // If no subparameters are used all strings will be empty
    let param_values = param.values.as_ref().unwrap();

    for (base_name, arguments) in set.clone() {
        for (i, value) in param_values.iter().enumerate() {
            let mut arguments_clone = arguments.clone();
            for index in indexes {
                arguments_clone[index.0] = arguments_clone[index.0]
                    .replace("parameter|", "") // Remove `parameter|` indicator
                    .replace(&format!("{{parameter_{}}}", parameter_name), value);
                // Replace `{parameter_name}` with value of the parameter
            }

            set.insert((
                base_name.clone() + "_" + parameter_name + "-" + &i.to_string(),
                arguments_clone,
            ));
        }

        set.remove(&(base_name, arguments));
    }
    Ok(())
}

/// Expands provided subparameter (Helper function)
fn expand_subparameter(
    parameter_name: &String,
    param: &Parameter,
    set: &mut BTreeSet<(String, Vec<String>)>,
    indexes: &Vec<(usize, String)>,
) -> anyhow::Result<()> {
    let mut sub_parameters = param.clone().sub.unwrap();
    let sub_parameter_size = sub_parameters.first_entry().unwrap().get().values.len();

    for (base_name, arguments) in set.clone() {
        for i in 0..sub_parameter_size {
            let mut arguments_clone = arguments.clone();
            for sub_index in indexes {
                arguments_clone[sub_index.0] = arguments_clone[sub_index.0]
                    .replace("parameter|", "") // Remove `parameter|` indicator
                    .replace(
                        &format!("{{parameter_{}_{}}}", parameter_name, &sub_index.1),
                        &sub_parameters[&sub_index.1].values[i],
                    ); // Replace `{parameter_name_subname}` with
                       // value of the sub parameter
            }
            set.insert((
                base_name.clone() + "_" + parameter_name + "-" + &i.to_string(),
                arguments_clone,
            ));
        }

        set.remove(&(base_name, arguments));
    }

    Ok(())
}

/// Checks that all parameters and subparameters in `parameters` were used in
/// `parameter_names_encountered`.
fn check_parameters_were_used(
    parameters: &BTreeMap<String, Parameter>,
    parameter_names_encountered: &BTreeSet<String>,
) -> anyhow::Result<()> {
    for (parameter_name, parameter) in parameters {
        if !parameter_names_encountered.contains(parameter_name) {
            bailc!(
                "Parameter was not used in any of the inputs", ;
                "{}", parameter_name;
                "",
            );
        }

        if let Some(sub) = &parameter.sub {
            for sub_name in sub.keys() {
                let name = parameter_name.clone() + "_" + sub_name;
                if !parameter_names_encountered.contains(&name) {
                    bailc!(
                        "Subparameter was not used in any of the inputs", ;
                        "{}", name;
                        "",
                    );
                }
            }
        }
    }

    Ok(())
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

impl FromIterator<(String, Program)> for ProgramMap {
    fn from_iter<T: IntoIterator<Item = (String, Program)>>(iter: T) -> Self {
        ProgramMap(iter.into_iter().collect())
    }
}

impl IntoIterator for ProgramMap {
    type Item = (String, Program);
    type IntoIter = std::collections::btree_map::IntoIter<String, Program>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<BTreeMap<String, Input>> for InputMap {
    fn from(value: BTreeMap<String, Input>) -> Self {
        InputMap(value)
    }
}

impl FromIterator<(String, Input)> for InputMap {
    fn from_iter<T: IntoIterator<Item = (String, Input)>>(iter: T) -> Self {
        InputMap(iter.into_iter().collect())
    }
}

impl IntoIterator for InputMap {
    type Item = (String, Input);
    type IntoIter = std::collections::btree_map::IntoIter<String, Input>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
#[path = "tests/maps.rs"]
mod tests;
