
use crate::database;
pub(crate) fn list_directories() -> Vec<String> {
    let database = database::Database::new("/home/denzyl/");
    

    database.list_directories()
}

mod test {

    #[test]
    fn test_list_directories() {}
}
