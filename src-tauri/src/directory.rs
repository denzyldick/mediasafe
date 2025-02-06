use crate::database;
pub(crate) fn list_directories() -> Vec<String> {
    let database = database::Database::new("/home/denzyl/");

    database.list_directories()
}

mod test {
    use crate::database;

    #[test]
    fn test_list_directories() {}

    #[test]
    fn test_add_directories() {
        let database = database::Database::new("./tests/");
        let path = "test";
        database.add_directory(&path);
        let directory = database.list_directories();

        assert_eq!(path, directory.get(0).unwrap())
    }
}

pub(crate) fn remove_directory(path: String) {
    let database = database::Database::new("/home/denzyl/");
    database.remove_directory(path);
}

pub(crate) fn add_directory(path: String) {
    let database = database::Database::new("/home/denzyl/");
    database.add_directory(&path);
}
