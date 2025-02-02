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
        let file = fs::read_to_string(format!("{}/mediasafe.yaml", data_directory));

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

mod tests {
    use crate::config::Config;

    

    #[test]
    fn new() {
        let config = Config::init("/home/denzyl/").unwrap();
        assert_eq!(config.database, "mediasafe.db");
        assert_eq!(config.port, "8080");
    }
}
