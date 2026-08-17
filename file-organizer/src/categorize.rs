use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// User-editable rules: category name -> list of extensions (no dots, lowercase).
#[derive(Deserialize)]
struct RulesFile {
    #[serde(flatten)]
    categories: HashMap<String, Vec<String>>,
}

/// Flat lookup table built from the rules: extension -> category.
pub struct Rules {
    map: HashMap<String, String>,
}

impl Rules {
    /// Load from a TOML file if given, otherwise fall back to built-in defaults.
    /// A missing/invalid config file is not fatal — we just warn and use defaults,
    /// since an organizer should never refuse to run over a config typo.
    pub fn load(config_path: Option<&Path>) -> Self {
        let categories = match config_path {
            Some(path) => match std::fs::read_to_string(path) {
                Ok(contents) => match toml::from_str::<RulesFile>(&contents) {
                    Ok(rules) => rules.categories,
                    Err(e) => {
                        eprintln!("Warning: couldn't parse {path:?} ({e}), using defaults");
                        default_categories()
                    }
                },
                Err(e) => {
                    eprintln!("Warning: couldn't read {path:?} ({e}), using defaults");
                    default_categories()
                }
            },
            None => default_categories(),
        };

        let mut map = HashMap::new();
        for (category, extensions) in categories {
            for ext in extensions {
                map.insert(ext.to_lowercase(), category.clone());
            }
        }
        Self { map }
    }

    /// Return the category for a file extension, or "Other" if unmapped.
    pub fn categorize(&self, extension: &str) -> String {
        self.map
            .get(&extension.to_lowercase())
            .cloned()
            .unwrap_or_else(|| "Other".to_string())
    }
}

fn default_categories() -> HashMap<String, Vec<String>> {
    let mut m = HashMap::new();
    m.insert(
        "Images".to_string(),
        vec!["jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "heic", "tiff"]
            .into_iter().map(String::from).collect(),
    );
    m.insert(
        "Documents".to_string(),
        vec!["pdf", "doc", "docx", "txt", "md", "rtf", "odt", "pages"]
            .into_iter().map(String::from).collect(),
    );
    m.insert(
        "Spreadsheets".to_string(),
        vec!["xls", "xlsx", "csv", "ods", "numbers"]
            .into_iter().map(String::from).collect(),
    );
    m.insert(
        "Presentations".to_string(),
        vec!["ppt", "pptx", "key", "odp"]
            .into_iter().map(String::from).collect(),
    );
    m.insert(
        "Video".to_string(),
        vec!["mp4", "mov", "avi", "mkv", "webm", "flv", "wmv"]
            .into_iter().map(String::from).collect(),
    );
    m.insert(
        "Audio".to_string(),
        vec!["mp3", "wav", "flac", "aac", "ogg", "m4a"]
            .into_iter().map(String::from).collect(),
    );
    m.insert(
        "Archives".to_string(),
        vec!["zip", "rar", "7z", "tar", "gz", "bz2", "xz"]
            .into_iter().map(String::from).collect(),
    );
    m.insert(
        "Code".to_string(),
        vec!["rs", "py", "js", "ts", "html", "css", "json", "toml", "yaml", "yml", "sh", "c", "cpp", "go", "java"]
            .into_iter().map(String::from).collect(),
    );
    m.insert(
        "Installers".to_string(),
        vec!["exe", "dmg", "pkg", "deb", "rpm", "msi", "appimage"]
            .into_iter().map(String::from).collect(),
    );
    m
}
