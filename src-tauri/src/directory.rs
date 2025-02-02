
use crate::database;
pub(crate) fn list_directories() -> Vec<String> {
    let database = database::Database::new("/home/denzyl/");
    

    database.list_directories()
}

mod test {

    #[test]
    fn test_list_directories() {}

    #[test]
    fn test_add_directories(){

    }
}

pub(crate) fn remove_directory(path: String)  {
    let database = database::Database::new("/home/denzyl/");
    database.remove_directory(path);
}

pub(crate) fn add_directory(path: String)  {
    let database = database::Database::new("/home/denzyl/");
    database.add_directory(path);
}
