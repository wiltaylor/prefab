use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf}, env::current_dir,
};

use anyhow::{anyhow, Ok, Result};
use serde::Deserialize;
use tera::{Context, Tera, from_value, to_value};

use crate::util::{copy_all, exec};
use regex::Regex as re;

#[derive(Deserialize, Debug, Clone)]
pub enum TemplateOption {
    FreeText {
        prompt: String,
        value: Option<String>,
        #[serde(default)]
        mandatory: bool,
    },
    Boolean {
        prompt: String,
        value: Option<bool>,
        #[serde(default)]
        mandatory: bool,
    },
    Integer {
        prompt: String,
        value: Option<i64>,
        #[serde(default)]
        mandatory: bool,
    },
    Float {
        prompt: String,
        value: Option<f64>,
        #[serde(default)]
        mandatory: bool,
    },
    Regex {
        prompt: String,
        pattern: String,
        value: Option<String>,
        #[serde(default)]
        mandatory: bool,
    },
    Choice {
        prompt: String,
        options: Vec<String>,
        value: Option<String>,
        #[serde(default)]
        mandatory: bool,
    },
}

#[derive(Deserialize, Debug)]
pub struct Manifest {
    pub title: Option<String>,
    #[allow(dead_code)]
    pub author: Option<String>,
    #[allow(dead_code)]
    pub description: Option<String>,
    before_hook: Option<String>,
    after_hook: Option<String>,
    options: HashMap<String, TemplateOption>,
}

#[derive(Debug)]
pub struct Template {
    tera: Tera,
    pub manifest: Manifest,
    pub source_path: PathBuf,
}

impl TemplateOption {
    pub fn get_value(&self) -> Option<String> {
        match self {
            TemplateOption::FreeText { value, .. } =>
                value.clone(),
            TemplateOption::Boolean { value, .. } =>
                value.as_ref().map(|o| format!("{}", o)),
            TemplateOption::Integer { value, .. } =>
                value.as_ref().map(|o| format!("{}", o)),
            TemplateOption::Float { value, .. } =>
                value.as_ref().map(|o| format!("{}", o)),
            TemplateOption::Regex { value, .. } =>
                value.clone(),
            TemplateOption::Choice { value, .. } =>
                value.clone(),
        }
    }

    pub fn set_value(self, text: String)-> TemplateOption {
        match self {
            TemplateOption::FreeText { prompt, value:_, mandatory } => {
                if !text.is_empty() {
                    TemplateOption::FreeText { prompt, value: Some(text), mandatory: false }
                }else{
                    TemplateOption::FreeText { prompt, value: None, mandatory }
                }
            },
            TemplateOption::Boolean { prompt, value:_, mandatory } => {
                let text = text.trim().to_lowercase();
                if text == "true" {
                    return TemplateOption::Boolean { prompt, value: Some(true), mandatory }
                }

                if text.is_empty() {
                    return TemplateOption::Boolean { prompt, value: None, mandatory };
                }

                TemplateOption::Boolean { prompt, value: Some(false), mandatory }
            },
            TemplateOption::Integer { prompt, value, mandatory } => {
                let result: Result<i64, _> = text.parse();

                if text.is_empty() {
                    return TemplateOption::Integer { prompt, value: None, mandatory };
                }

                 match result {
                    Result::Ok(num) => {
                        TemplateOption::Integer { prompt, value: Some(num), mandatory }
                    },
                    Err(_) => {
                        TemplateOption::Integer { prompt, value, mandatory }
                    },
                }                
            },
            TemplateOption::Float { prompt, value, mandatory } => {
                let mut text = text;
                if text.ends_with('.') {
                    text += "0";
                }

                let result: Result<f64, _> = text.parse();

                if text.is_empty() {
                    return TemplateOption::Float { prompt, value: None, mandatory };
                }

                 match result {
                    Result::Ok(num) => {
                        TemplateOption::Float { prompt, value: Some(num), mandatory }
                    },
                    Err(_) => {
                        TemplateOption::Float { prompt, value, mandatory }
                    },
                }  
            },
            TemplateOption::Regex { prompt, pattern, value:_, mandatory } => {
                if !text.is_empty() {
                    TemplateOption::Regex { prompt, pattern, value: Some(text), mandatory }
                }else{
                    TemplateOption::Regex { prompt, pattern, value: None, mandatory }
                }
            },
            TemplateOption::Choice { prompt, options, value:_, mandatory } => {
                if !text.is_empty() {
                    TemplateOption::Choice { prompt, options, value: Some(text), mandatory }
                }else{
                    TemplateOption::Choice { prompt, options, value: None, mandatory }
                }
            },
        }       
    }

    pub fn get_pattern(&self) -> Option<String> {
        match self {
            TemplateOption::Regex { pattern, .. } => Some(pattern.clone()),
            _ => None
        }
    }

    pub fn get_choice_options(&self) -> Option<Vec<String>> {
        match self {
            TemplateOption::Choice { options, .. } => Some(options.clone()),
            _ => None,
        }
    }

    pub fn get_prompt(&self) -> String {
        match self {
            TemplateOption::FreeText { prompt, .. } => prompt.clone(),
            TemplateOption::Boolean { prompt,.. } =>  prompt.clone(),
            TemplateOption::Integer { prompt,.. } =>  prompt.clone(),
            TemplateOption::Float { prompt,.. } =>  prompt.clone(),
            TemplateOption::Regex { prompt,.. } =>  prompt.clone(),
            TemplateOption::Choice { prompt,.. } =>  prompt.clone(),
        }
    }

    pub fn is_mandatory(&self) -> bool {
        match self {
            TemplateOption::FreeText { mandatory, .. } => *mandatory,
            TemplateOption::Boolean { mandatory,.. } => *mandatory,
            TemplateOption::Integer { mandatory,.. } => *mandatory,
            TemplateOption::Float { mandatory,.. } => *mandatory,
            TemplateOption::Regex { mandatory,.. } => *mandatory,
            TemplateOption::Choice { mandatory,.. } => *mandatory,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self{
            TemplateOption::FreeText { value, .. } => value.is_none(),
            TemplateOption::Boolean { value,.. } => value.is_none(),
            TemplateOption::Integer { value,.. } => value.is_none(),
            TemplateOption::Float { value,.. } => value.is_none(),
            TemplateOption::Regex { value,.. } => value.is_none(),
            TemplateOption::Choice { value,.. } => value.is_none(),
        }
    }
}

impl Manifest { 
    fn validate_options(&self) -> Result<()> {

        for (k, v) in &self.options {
            match v {
                TemplateOption::Regex { pattern, value: Some(value), .. } => {

                    let regex = re::new(pattern)?;

                    if !regex.is_match(value) {
                        return Err(anyhow!("Regular expression for {} didn't match!", k));
                    }
                },
                TemplateOption::Choice { options, value: Some(value), .. } => {
                    if !options.contains(value) {
                        return Err(anyhow!("Option not in the choice list for {} was set.", k));
                    }
                },
                _ => ()
            }
        }

        Ok(())
    }
}

fn system(args: &HashMap<String, tera::Value>) -> Result<tera::Value, tera::Error> {

    if let Some(command) = args.get("command"){
        if let Result::Ok(command) = from_value::<String>(command.clone()){
            return if let Result::Ok(result) = exec(command.as_str(), HashMap::<String, String>::new(), current_dir()?) {
                Result::Ok(to_value::<String>(result)?)
            } else {
                Err(tera::Error::msg("Failed to run command!"))
            }   
        }
    }
    
    Err(tera::Error::msg("You need to pass in the command argument!"))
}

impl Template {
    fn render_file_path(&mut self, path: impl AsRef<Path>) -> Result<String> {
        let ctx = self.get_context();
        if let Some(path) = path.as_ref().clone().to_str(){
            Ok(self.tera.render_str(path, &ctx)?)
        }else{
            Err(anyhow!("Failed to process path!"))
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Template> {
        let mut manifest_path = PathBuf::new();
        let mut tera = Tera::default();

        manifest_path.push(&path);
        manifest_path.push("prefab.toml");

        let mut template_folder_path = PathBuf::new();
        template_folder_path.push(&path);
        template_folder_path.push("template");

        if !manifest_path.exists() {
            return Err(anyhow!(
                "Could not find a template manifest at template path {:?}",
                &manifest_path
            ));
        }

        let file = fs::read_to_string(&manifest_path)?;
        let toml = toml::from_str::<Manifest>(&file).unwrap();

        if template_folder_path.exists() {
            //Add file glob so we find all items in template
            template_folder_path.push("**");

            let tera_dir = template_folder_path.as_mut_os_str().to_str().unwrap();
            tera = Tera::new(tera_dir)?;
        }

        tera.register_function("system", system);

        Ok(Template {
            tera,
            manifest: toml,
            source_path: path.as_ref().to_path_buf(),
        })
    }

    fn get_context(&self) -> Context {
        let mut ctx = Context::new();

        for (name, opt) in &self.manifest.options {
            match opt {
                TemplateOption::FreeText { value, .. } => {
                    if let Some(val) = value {
                        ctx.insert(name, val);
                    }
                }
                TemplateOption::Boolean { value, .. } => {
                    if let Some(val) = value {
                        ctx.insert(name, val);
                    }
                }
                TemplateOption::Integer {  value, .. } => {
                    if let Some(val) = value {
                        ctx.insert(name, val);
                    }
                }
                TemplateOption::Float { value, .. } => {
                    if let Some(val) = value {
                        ctx.insert(name, val);
                    }
                }
                TemplateOption::Regex { value, .. } => {
                    if let Some(val) = value {
                        ctx.insert(name, val);
                    }
                }
                TemplateOption::Choice { value, .. } => {
                    if let Some(val) = value {
                        ctx.insert(name, val)
                    }
                }
            }
        }

        ctx
    }

    pub fn apply(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let ctx = self.get_context();
        let names: Vec<String> = self.tera.get_template_names().map(|n| n.to_string()).collect();

        self.manifest.validate_options()?;

        if let Some(hook) = &self.manifest.before_hook {
            let mut env = HashMap::<String, String>::new();
            env.insert("PREFAB_TEMPLATE".to_string(), self.source_path.to_string_lossy().to_string());
            exec(hook, env, &path)?;
        }

        for tmpl in names {
            let text = self.tera.render(&tmpl, &ctx)?;
            let mut file_name = PathBuf::new();

            file_name.push(&path);
            file_name.push(tmpl);

            let rendered_path = self.render_file_path(file_name)?;
            file_name = PathBuf::new();
            file_name.push(rendered_path);

            let mut folder = file_name.clone();
            folder.pop();
            fs::create_dir_all(folder)?;

            println!("Creating {:?}", &file_name);

            fs::write(file_name, text)?;
        }

        let mut static_folder_path = PathBuf::new();
        static_folder_path.push(&self.source_path);
        static_folder_path.push("static");

        if static_folder_path.exists() {
            let ctx = self.get_context();
            copy_all(static_folder_path, &path, &mut self.tera, &ctx)?;
        }

        if let Some(hook) = &self.manifest.after_hook {
            let mut env = HashMap::<String, String>::new();
            env.insert("PREFAB_TEMPLATE".to_string(), self.source_path.to_string_lossy().to_string());
            exec(hook, env, &path)?;
        }

        Ok(())
    }

    pub fn get_options(&self) -> HashMap<String, TemplateOption> {
        let mut result = HashMap::new();
        for (key, value) in self.manifest.options.iter() {
            result.insert(key.clone(), value.clone());
        }

        result
    }

    pub fn set_options(&mut self, options: HashMap<String, TemplateOption>) {
        self.manifest.options = options;
    }
}
