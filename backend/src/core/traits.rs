use rusqlite::types::ToSqlOutput;
use rusqlite::{Connection, Result, Row};

/// Mapping of database row to a type
pub trait Mappable {
    fn from_row(row: &Row, conn: &Connection) -> Result<Self>
    where
        Self: Sized;
}

/// Allows searching for an record in the database by ID
pub trait Searchable {
    fn search(conn: &Connection, id_value: i32) -> Result<Self>
    where
        Self: Sized;
}

/// Insertion of an items into the database
pub trait Insertable {
    // Checks id duplicates in db
    fn check_duplicate(conn: &Connection, id_value: i32) -> bool {
        let query = format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = ?1)",
            Self::table_name(),
            Self::id_column()
        );

        conn.query_row(&query, [id_value], |row| row.get::<_, i32>(0))
            .unwrap_or(0)
            != 0
    }

    fn table_name() -> &'static str;
    fn columns() -> Vec<&'static str>;
    fn id_column() -> &'static str;
    fn id_value(&self) -> i32;

    // Returns all values for specific type
    fn values(&self) -> Vec<ToSqlOutput<'_>>;

    fn post_insert(&self, _conn: &Connection) -> Result<()> {
        Ok(()) // do nothing by default
    }

    fn post_delete(_id_value: Option<&i32>, _conn: &Connection) -> Result<()> {
        Ok(()) // do nothing by default
    }
}
