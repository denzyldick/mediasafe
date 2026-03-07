use super::database;
pub(crate) fn list_directories(config_path: &str) -> Vec<String> {
    let database = database::Database::new(config_path);

    database.list_directories()
}

pub(crate) fn remove_directory(path: String, config_path: &str) {
    let database = database::Database::new(config_path);
    database.remove_directory(path);
}

pub(crate) fn add_directory(path: String, config_path: &str) {
    println!("directory::add_directory called with: {path}");
    let database = database::Database::new(config_path);
    database.add_directory(&path);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_list_directories() {}

    #[test]
    fn test_add_directories() {
        let database = database::Database::new("./tests/");
        let path = "test";
        database.add_directory(path);
        let directory = database.list_directories();

        assert_eq!(path, directory.first().unwrap())
    }
}
