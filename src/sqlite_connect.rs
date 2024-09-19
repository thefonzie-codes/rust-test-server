use rusqlite::{Connection, Result};
use rusqlite::NO_PARAMS;

fn main() {
    let connection = Connection::open("mockdb.db").unwrap();

    let query = "
    CREATE TABLE users (name TEXT, age INTEGER);
    INSERT INTO users VALUES ('Alice', 42);
    INSERT INTO users VALUES ('Bob', 69);
";

    connection.execute(query).unwrap();
}
