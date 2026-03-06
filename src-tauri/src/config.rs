use std::fs;
use yaml_rust::YamlLoader;

struct Config {
    database: String,
    port: String,
    ip: String,
    gpu: bool,
    folders: Vec<Folder>,
}

struct Folder {
    path: String,
    name: String,
}
impl Config {
    pub fn init(data_directory: &str) -> Option<Config> {
        let file = fs::read_to_string(format!("{}/siegu.yaml", data_directory));

        if let Ok(config) = file {
            let docs = YamlLoader::load_from_str(&config).unwrap();

            let config = Config::vec_to_config(docs);
            Some(config)
        } else {
            None
        }
    }

    pub fn vec_to_config(docs: Vec<yaml_rust::Yaml>) -> Config {
        let doc = &docs[0];
        let database = doc["database"].as_str().unwrap();
        let port = doc["port"].as_str().unwrap();
        let ip = doc["ip"].as_str().unwrap();
        let gpu = doc["gpu"].as_bool().unwrap();
        let folders = doc["folders"].as_vec().unwrap();
        let mut folder_vec = Vec::new();
        for folder in folders {
            let path = folder["path"].as_str().unwrap();
            let name = folder["name"].as_str().unwrap();
            folder_vec.push(Folder {
                path: path.to_string(),
                name: name.to_string(),
            });
        }
        Config {
            database: database.to_string(),
            port: port.to_string(),
            ip: ip.to_string(),
            gpu,
            folders: folder_vec,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn new() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("siegu.yaml");
        let mut file = std::fs::File::create(&config_path).unwrap();
        writeln!(
            file,
            "database: siegu.db\nport: '8080'\nip: 127.0.0.1\ngpu: false\nfolders:\n  - path: /tmp\n    name: tmp"
        )
        .unwrap();

        let config = Config::init(temp_dir.to_str().unwrap()).unwrap();
        assert_eq!(config.database, "siegu.db");
        assert_eq!(config.port, "8080");

        std::fs::remove_file(config_path).unwrap();
    }
}
