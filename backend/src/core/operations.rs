use rusqlite::{params, Connection, Error, Result};

use axum::{http::StatusCode, response::Json as AxumJson};

use crate::core::traits::{Insertable, Mappable};
use crate::core::types::{Article, DbPool, OrderItem};
use serde_json::json;
use std::fmt::Debug;

use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;

pub fn establish_connection(
    pool: &DbPool,
) -> Result<PooledConnection<SqliteConnectionManager>, (StatusCode, AxumJson<serde_json::Value>)> {
    pool.get().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            AxumJson(json!({ "error": "Failed to get connection" })),
        )
    })
}

pub fn update_record<T: Insertable>(conn: &Connection, item: &T) -> Result<()> {
    let table = T::table_name();
    let columns = T::columns();
    let id_column = T::id_column();
    let set_clause = columns
        .iter()
        .enumerate()
        .map(|(i, col)| format!("{} = ?{}", col, i + 2))
        .collect::<Vec<String>>()
        .join(", ");

    let query = format!(
        "UPDATE {} SET {} WHERE {} = ?1",
        table, set_clause, id_column
    );

    let mut params = vec![item.id_value().into()];
    params.extend(item.values());

    conn.execute(&query, rusqlite::params_from_iter(params))?;
    Ok(())
}

pub fn find_record_by_id<T: Mappable + Insertable>(conn: &Connection, id_value: i32) -> Result<T> {
    let table = T::table_name();
    let id_column = T::id_column();
    let columns = T::columns().join(",");

    let query = format!("SELECT {} FROM {} WHERE {} = ?1", columns, table, id_column);

    let mut stmt = conn.prepare(&query)?;
    let mut iter = stmt.query_map([id_value], |row| T::from_row(row, conn))?;

    iter.next()
        .ok_or_else(|| Error::QueryReturnedNoRows)?
        .map_err(|err| err)
}

pub fn insert_record<T: Mappable + Insertable>(conn: &Connection, item: &T) -> Result<()> {
    if T::check_duplicate(conn, item.id_value()) {
        return Err(Error::ToSqlConversionFailure(Box::new(
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Item ID {} is already beeing used.", item.id_value()).to_string(),
            ),
        )));
    }

    let table = T::table_name();
    let columns = T::columns();

    let columns_str = columns.join(", ");
    let placeholders = (1..=columns.len())
        .map(|i| format!("?{}", i))
        .collect::<Vec<String>>()
        .join(", ");

    let query = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table, columns_str, placeholders
    );

    let values = item.values();

    conn.execute(&query, rusqlite::params_from_iter(values))?;

    item.post_insert(conn)?;

    Ok(())
}

pub fn delete_record_by_id<T: Mappable + Insertable + Debug>(
    conn: &Connection,
    id: &Option<i32>,
) -> Result<()> {
    let id_column = T::id_column();
    let table = T::table_name();

    match id {
        Some(id_value) => {
            let query = format!("DELETE FROM {} WHERE {} = ?1", table, id_column);
            conn.execute(&query, params![id_value])?;

            // Post delete used for Order Type, to delete id-entry in article_order table
            T::post_delete(Some(id_value), conn)?;
        }
        None => {
            let query = format!("DELETE FROM {}", table);
            conn.execute(&query, params![])?;

            // Post delete used for Order Type, to delete all-entries in article_order table
            T::post_delete(None, conn)?;
        }
    }
    Ok(())
}

pub fn fetch_all_records<T: Insertable + Mappable + Debug>(conn: &Connection) -> Result<Vec<T>> {
    let table = T::table_name();

    let columns = T::columns().join(",");

    let query = format!("SELECT {} FROM {}", columns, table);

    let mut stmt = conn.prepare(&query)?;

    let iter = stmt.query_map([], |row| T::from_row(row, conn))?;

    let mut item_list = Vec::new();
    for item in iter {
        item_list.push(item?);
    }

    Ok(item_list)
}

pub fn fetch_order_items(conn: &Connection, order_id: i32) -> Result<Vec<OrderItem>> {
    let mut stmt = conn.prepare(
        "
        SELECT a.article_id, a.name, a.price, a.manufacturer, a.stock, a.category, oa.quantity
        FROM article a
        JOIN order_article oa ON a.article_id = oa.article_id
        WHERE oa.order_id = ?
        ",
    )?;

    let article_iter = stmt.query_map([order_id], |row| {
        let article = Article::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
        );
        let quantity: i32 = row.get(6)?;
        let order_item = OrderItem::new(article, quantity);
        Ok(order_item)
    })?;

    article_iter.collect::<Result<Vec<_>, _>>()
}

pub fn initialize_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- Table for articles
        CREATE TABLE IF NOT EXISTS article (
            id             INTEGER PRIMARY KEY,
            article_id     INTEGER NOT NULL,
            name           TEXT NOT NULL,
            price          REAL NOT NULL,
            manufacturer   TEXT NOT NULL,
            stock          INTEGER NOT NULL, 
            category       TEXT
        ); 

        -- Table for customers
        CREATE TABLE IF NOT EXISTS customer (
            id             INTEGER PRIMARY KEY,
            customer_id    INTEGER NOT NULL,
            first_name     TEXT NOT NULL,
            last_name      TEXT NOT NULL,
            street         TEXT NOT NULL, 
            location       TEXT NOT NULL,
            zip_code       INTEGER NOT NULL,
            email          TEXT NOT NULL
        );

        -- Table for orders
        CREATE TABLE IF NOT EXISTS orders ( 
            id             INTEGER PRIMARY KEY,
            order_id       INTEGER NOT NULL,
            customer_id    INTEGER NOT NULL,
            date           TEXT NOT NULL,
            order_type     TEXT NOT NULL,
            status         TEXT NOT NULL, 
            FOREIGN KEY(customer_id) REFERENCES customer(customer_id)
        );

        -- Table for order-article relationships
        CREATE TABLE IF NOT EXISTS order_article (
            id           INTEGER PRIMARY KEY,
            order_id     INTEGER NOT NULL,
            article_id   INTEGER NOT NULL,
            quantity     INTEGER NOT NULL,
            FOREIGN KEY (order_id) REFERENCES orders(order_id),
            FOREIGN KEY (article_id) REFERENCES article(article_id)
        );
        ",
    )?;
    Ok(())
}
