use std::{collections::HashMap, path::PathBuf, str::FromStr};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub logs_folder_path: String,
    pub server_crud_feature: bool
}

impl Default for Config {
    fn default() -> Self {
        return Self::new("logs".into(), true)
    }
}

// We want this to be blocking
impl Config {
    pub fn new(logs_folder_path: String, server_crud_feature: bool) -> Self {
        Self {
            logs_folder_path,
            server_crud_feature
        }
    }
    fn create_config_directories() -> PathBuf {
        let path = Self::get_default_config_dir();
        match std::fs::create_dir_all(&path) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e.to_string())
            }
        };
        return path;
    }
    pub fn get_default_config_dir() -> PathBuf {
        let mut config_path = match std::env::var("STEWARDX_CONFIG") {
            Ok(p) => PathBuf::from_str(p.as_str()).unwrap(),
            Err(_) => Self::default_config_path()
        };
        config_path.pop();
        return config_path;
    }
    fn default_config_path() -> PathBuf {
        let mut config_dir = match home::home_dir() {
            Some(p) => p,
            None => {
                panic!("Please specify a $HOME variable so that I can create configuration files for you.");
            }
        };
        config_dir.push(".config");
        config_dir.push("stewardx");
        config_dir.push("config.json");
        return config_dir;
    }
    pub fn get_config_dir() -> PathBuf {
        match std::env::var("STEWARDX_CONFIG") {
            Ok(o) => {
                let mut p = PathBuf::from_str(o.as_str()).unwrap();
                p.pop();
                p
            },
            Err(_) => {
                return Self::get_default_config_dir()
            }
        }
    }
    pub fn prepare_config() -> Self {
        let config_path = match std::env::var("STEWARDX_CONFIG") {
            Ok(p) => PathBuf::from_str(p.as_str()).unwrap(),
            Err(_) => {
                tracing::info!("Environment variable STEWARDX_CONFIG is not found. I'll try to create the default config directories and file.");
                let mut dir_path = Self::create_config_directories();
                dir_path.push("config.json");
                dir_path
            }
        };
        return match std::fs::read_to_string(&config_path) {
            Ok(o) => serde_json::from_str::<Self>(&o).expect("Malformed config file!"),
            Err(_e) => {
                tracing::info!("Creating config file with defaults.");
                let c = Self::default();
                let content = serde_json::to_string(&c).unwrap();
                std::fs::write(&config_path, &content).expect("Couldn't create default config file. Check permissions please.");
                c
            }
        };
    }
    pub fn get_logs_folder_path(&self) -> PathBuf {
        let logs_folder_path = &self.logs_folder_path;
        if logs_folder_path.starts_with("/") {
            return PathBuf::from_str(&logs_folder_path).unwrap();
        }
        let mut config_dir = Self::get_config_dir();
        config_dir.extend(logs_folder_path.split("/"));
        let index_path = config_dir;
        return index_path;
    }
    pub fn get_features<'a>(&'a self) -> HashMap<&'a str, bool> {
        let mut features = HashMap::new();
        features.insert("server_crud", self.server_crud_feature);
        features
    }
}
