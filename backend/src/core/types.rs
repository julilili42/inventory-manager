use crate::core::operations::{fetch_order_items, find_record_by_id};
use crate::core::traits::{Insertable, Mappable, Searchable};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::types::{Null, ToSqlOutput};
use rusqlite::{params, Connection, Error, Result, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use strum_macros::{Display, EnumString};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct Article {
    pub article_id: i32,
    pub name: String,
    pub price: f64,
    pub manufacturer: String,
    pub stock: i32,
    pub category: Option<String>,
}

impl Article {
    pub fn new(
        article_id: i32,
        name: String,
        price: f64,
        manufacturer: String,
        stock: i32,
        category: Option<String>,
    ) -> Article {
        Article {
            article_id,
            name,
            price,
            manufacturer,
            stock,
            category,
        }
    }
}

impl Mappable for Article {
    fn from_row(row: &Row, _conn: &Connection) -> Result<Self> {
        Ok(Article::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
        ))
    }
}

impl Searchable for Article {
    fn search(conn: &Connection, id: i32) -> Result<Self>
    where
        Self: Sized,
    {
        find_record_by_id(conn, id)
    }
}

impl Insertable for Article {
    fn table_name() -> &'static str {
        "article"
    }
    fn columns() -> Vec<&'static str> {
        vec![
            "article_id",
            "name",
            "price",
            "manufacturer",
            "stock",
            "category",
        ]
    }
    fn id_column() -> &'static str {
        "article_id"
    }
    fn id_value(&self) -> i32 {
        self.article_id
    }

    fn values(&self) -> Vec<ToSqlOutput<'_>> {
        vec![
            self.article_id.into(),
            self.name.clone().into(),
            self.price.into(),
            self.manufacturer.clone().into(),
            self.stock.into(),
            match &self.category {
                Some(s) => s.as_str().into(),
                None => Null.into(),
            },
        ]
    }

    fn post_delete(id_value: Option<&i32>, conn: &Connection) -> Result<()> {
        if let Some(article_id) = id_value {
            conn.execute(
                "DELETE FROM order_article WHERE article_id = ?1",
                params![article_id],
            )?;

            // Delete all orders that no longer have any linked articles after the previous deletion
            conn.execute(
            "DELETE FROM orders WHERE order_id NOT IN (SELECT DISTINCT order_id FROM order_article)",
            params![],
        )?;
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct Customer {
    pub customer_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub street: String,
    pub location: String,
    pub zip_code: i32,
    pub email: String,
}

impl Customer {
    pub fn new(
        customer_id: i32,
        first_name: String,
        last_name: String,
        street: String,
        location: String,
        zip_code: i32,
        email: String,
    ) -> Customer {
        Customer {
            customer_id,
            first_name,
            last_name,
            street,
            location,
            zip_code,
            email,
        }
    }
}

impl Mappable for Customer {
    fn from_row(row: &Row, _conn: &Connection) -> Result<Self> {
        Ok(Customer::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
        ))
    }
}

impl Insertable for Customer {
    fn table_name() -> &'static str {
        "customer"
    }
    fn columns() -> Vec<&'static str> {
        vec![
            "customer_id",
            "first_name",
            "last_name",
            "street",
            "location",
            "zip_code",
            "email",
        ]
    }
    fn id_column() -> &'static str {
        "customer_id"
    }

    fn id_value(&self) -> i32 {
        self.customer_id
    }

    fn values(&self) -> Vec<ToSqlOutput<'_>> {
        vec![
            self.customer_id.into(),
            self.first_name.clone().into(),
            self.last_name.clone().into(),
            self.street.clone().into(),
            self.location.clone().into(),
            self.zip_code.into(),
            self.email.clone().into(),
        ]
    }

    fn post_delete(id_value: Option<&i32>, conn: &Connection) -> Result<()> {
        if let Some(customer_id) = id_value {
            conn.execute(
            "DELETE FROM order_article WHERE order_id IN (SELECT order_id FROM orders WHERE customer_id = ?1)",
            params![customer_id],
        )?;

            conn.execute(
                "DELETE FROM orders WHERE customer_id = ?1",
                params![customer_id],
            )?;
        }

        Ok(())
    }
}

impl Searchable for Customer {
    fn search(conn: &Connection, id: i32) -> Result<Self>
    where
        Self: Sized,
    {
        find_record_by_id(conn, id)
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct OrderItem {
    pub article: Article,
    pub quantity: i32,
}

impl OrderItem {
    pub fn new(article: Article, quantity: i32) -> Self {
        OrderItem { article, quantity }
    }
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, ToSchema)]
pub enum OrderType {
    Return,
    Sale,
}

impl OrderType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Return" => Some(OrderType::Return),
            "Sale" => Some(OrderType::Sale),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, ToSchema)]
pub enum OrderStatus {
    Pending,
    Completed,
    Shipped,
    Delivered,
}

impl OrderStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Pending" => Some(OrderStatus::Pending),
            "Completed" => Some(OrderStatus::Completed),
            "Shipped" => Some(OrderStatus::Shipped),
            "Delivered" => Some(OrderStatus::Delivered),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct Order {
    pub order_id: i32,
    pub customer: Customer,
    pub items: Vec<OrderItem>,
    pub date: String,
    pub order_type: OrderType,
    pub status: OrderStatus,
}

impl Order {
    pub fn new(
        order_id: i32,
        customer: Customer,
        items: Vec<OrderItem>,
        date: String,
        order_type: OrderType,
        status: OrderStatus,
    ) -> Self {
        Order {
            order_id,
            customer,
            items,
            date,
            order_type,
            status,
        }
    }
}

impl Mappable for Order {
    fn from_row(row: &Row, conn: &Connection) -> Result<Self> {
        let order_id = row.get(0)?;
        let fetched_order_items = fetch_order_items(conn, order_id)?;

        let customer_id = row.get(1)?;
        let fetched_customer = find_record_by_id::<Customer>(conn, customer_id)?;

        let date = row.get(2)?;
        let order_type: String = row.get(3)?;
        let status: String = row.get(4)?;

        let order_type = OrderType::from_str(&order_type)
            .ok_or_else(|| Error::InvalidParameterName("Invalid order_type".into()))?;
        let status = OrderStatus::from_str(&status)
            .ok_or_else(|| Error::InvalidParameterName("Invalid status".into()))?;

        Ok(Order::new(
            order_id,
            fetched_customer,
            fetched_order_items,
            date,
            order_type,
            status,
        ))
    }
}

impl Insertable for Order {
    fn table_name() -> &'static str {
        "orders"
    }
    fn columns() -> Vec<&'static str> {
        vec!["order_id", "customer_id", "date", "order_type", "status"]
    }
    fn id_column() -> &'static str {
        "order_id"
    }

    fn id_value(&self) -> i32 {
        self.order_id
    }

    fn values(&self) -> Vec<ToSqlOutput<'_>> {
        vec![
            self.order_id.into(),
            self.customer.customer_id.into(),
            self.date.clone().into(),
            self.order_type.to_string().into(),
            self.status.to_string().into(),
        ]
    }

    fn post_insert(&self, conn: &Connection) -> Result<()> {
        let query = "
            INSERT INTO order_article (order_id, article_id, quantity)
            VALUES (?1, ?2, ?3)
        ";

        let mut stmt = conn.prepare(query)?;

        for order_item in &self.items {
            stmt.execute(params![
                self.order_id,
                order_item.article.article_id,
                order_item.quantity
            ])?;
        }

        Ok(())
    }

    fn post_delete(id_value: Option<&i32>, conn: &Connection) -> Result<()> {
        match id_value {
            Some(id_value) => {
                let query = "DELETE FROM order_article WHERE order_id = ?1";

                conn.execute(query, params![id_value])?;
            }
            None => {
                let query = "DELETE FROM order_article";

                conn.execute(query, params![])?;
            }
        }

        Ok(())
    }
}

impl Searchable for Order {
    fn search(conn: &Connection, id: i32) -> Result<Self>
    where
        Self: Sized,
    {
        find_record_by_id(conn, id)
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ArticleStatistics {
    pub ordered_quantities: HashMap<i32, i32>,
    pub article_revenue: HashMap<i32, f64>,
}

impl ArticleStatistics {
    pub fn new(ordered_quantities: HashMap<i32, i32>, article_revenue: HashMap<i32, f64>) -> Self {
        ArticleStatistics {
            ordered_quantities,
            article_revenue,
        }
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct OrderStatistics {
    pub total_prices: HashMap<i32, f64>,
}

impl OrderStatistics {
    pub fn new(total_prices: HashMap<i32, f64>) -> Self {
        OrderStatistics { total_prices }
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CustomerStatistics {
    pub number_of_orders: HashMap<i32, i32>,
    pub total_revenue: HashMap<i32, f64>,
    pub most_bought_item: HashMap<i32, String>,
}

impl CustomerStatistics {
    pub fn new(
        number_of_orders: HashMap<i32, i32>,
        total_revenue: HashMap<i32, f64>,
        most_bought_item: HashMap<i32, String>,
    ) -> Self {
        CustomerStatistics {
            number_of_orders,
            total_revenue,
            most_bought_item,
        }
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Statistics {
    pub article_statistics: ArticleStatistics,
    pub order_statistics: OrderStatistics,
    pub customer_statistics: CustomerStatistics,
}

impl Statistics {
    pub fn new(
        article_statistics: ArticleStatistics,
        order_statistics: OrderStatistics,
        customer_statistics: CustomerStatistics,
    ) -> Self {
        Statistics {
            article_statistics,
            order_statistics,
            customer_statistics,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum ApiResponse {
    Article(Article),
    Customer(Customer),
    Order(Order),
}

pub type DbPool = Arc<r2d2::Pool<SqliteConnectionManager>>;
