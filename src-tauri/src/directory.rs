use database::Database;

use crate::database;
pub(crate) fn list_directories() -> Vec<String> {
    let database = database::Database::new("/home/denzyl/");
    let directories = (&database).list_directories();

    directories
}

mod test {

    #[test]
    fn test_list_directories() {}
}
