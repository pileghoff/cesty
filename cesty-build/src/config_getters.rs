use std::collections::VecDeque;

use crate::CestyBuildError;

pub fn string(
    config: &toml::map::Map<String, toml::Value>,
    test_name: &str,
    key: &'static str,
) -> Result<String, CestyBuildError> {
    let Some(value) = config.get(key) else {
        return Ok(String::new());
    };

    Ok(value
        .as_str()
        .ok_or_else(|| CestyBuildError::ManifestTestParseError {
            section: test_name.to_owned(),
            message: format!("`{key}` must be a string"),
        })?
        .to_string())
}

pub fn string_pairs(
    config: &toml::map::Map<String, toml::Value>,
    test_name: &str,
    key: &'static str,
) -> Result<Vec<(String, String)>, CestyBuildError> {
    let Some(value) = config.get(key) else {
        return Ok(Vec::new());
    };

    let values = value
        .as_table()
        .ok_or_else(|| CestyBuildError::ManifestTestParseError {
            section: test_name.to_owned(),
            message: format!("`{key}` must be an array of strings"),
        })?;

    fn _cleanup(mut s: String) -> String {
        if s.starts_with('"') && s.ends_with('"') {
            s.remove(0);
            s.pop();
        }
        s
    }
    Ok(values
        .iter()
        .map(|value| (_cleanup(value.0.to_string()), _cleanup(value.1.to_string())))
        .collect())
}

pub fn string_array(
    config: &toml::map::Map<String, toml::Value>,
    test_name: &str,
    key: &'static str,
    required: bool,
) -> Result<VecDeque<String>, CestyBuildError> {
    let Some(value) = config.get(key) else {
        if required {
            return Err(CestyBuildError::ManifestTestParseError {
                section: test_name.to_owned(),
                message: format!("missing required `{key}` array"),
            });
        }

        return Ok(VecDeque::new());
    };
    if let Some(value) = value.as_str() {
        return Ok(VecDeque::from(vec![value.to_string()]));
    }

    let values = value
        .as_array()
        .ok_or_else(|| CestyBuildError::ManifestTestParseError {
            section: test_name.to_owned(),
            message: format!("`{key}` must be an array of strings"),
        })?;

    values
        .iter()
        .map(|value| {
            value.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                CestyBuildError::ManifestTestParseError {
                    section: test_name.to_owned(),
                    message: format!("`{key}` must be an array of strings"),
                }
            })
        })
        .collect()
}
