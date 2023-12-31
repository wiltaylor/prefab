use std::{collections::HashMap, fs, path::Path};

use crate::template::TemplateOption;
use anyhow::{Result, anyhow};
use toml::Value;

pub fn load_config(path: impl AsRef<Path>, mut options: HashMap<String, TemplateOption>) -> Result<HashMap<String, TemplateOption>> {
    let file = fs::read_to_string(path)?; 
    let toml = toml::from_str::<HashMap<String, Value>>(&file)?;

    for (name, val) in toml {
        match val {
            Value::String(val) => {
                let entry = 
                    options
                    .get_mut(&name)
                    .ok_or(anyhow!("No entry for {} in manifest!", name))?;

                match entry {
                    TemplateOption::FreeText { value, .. } => {
                        *value = Some(val);
                    }
                    TemplateOption::Regex { value, .. } => {
                        *value = Some(val);
                    }
                    TemplateOption::Choice { value, .. } => {
                        *value = Some(val);
                    }
                    _ => {
                        return Err(anyhow!(
                            "Incorrect type in config for {}! Expected string!",
                            name
                        ));
                    }
                }
            }
            Value::Integer(val) => {
                let entry = options
                    .get_mut(&name)
                    .ok_or(anyhow!("No entry for {} in manifest!", name))?;

                if let TemplateOption::Integer { value, .. } = entry {
                    *value = Some(val);
                } else {
                    return Err(anyhow!(
                        "Incorrect type in config for {}! Expected Integer!",
                        name
                    ));
                }
            }
            Value::Float(val) => {
                let entry = options
                    .get_mut(&name)
                    .ok_or(anyhow!("No entry for {} in manifest!", name))?;

                if let TemplateOption::Float { value, .. } = entry {
                    *value = Some(val);
                } else {
                    return Err(anyhow!(
                        "Incorrect type in config for {}! Expected Integer!",
                        name
                    ));
                }
            }
            Value::Boolean(val) => {
                let entry = options
                    .get_mut(&name)
                    .ok_or(anyhow!("No entry for {} in manifest!", name))?;

                if let TemplateOption::Boolean { value, .. } = entry {
                    *value = Some(val);
                } else {
                    return Err(anyhow!(
                        "Incorrect type in config for {}! Expected Integer!",
                        name
                    ));
                }
            }
            _ => return Err(anyhow!("Unsupported type in toml for {}! {:?}", name, val)),
        }
    }

    Ok(options)

}