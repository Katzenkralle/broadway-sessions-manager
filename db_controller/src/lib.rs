use diesel::sqlite::SqliteConnection;
use diesel::Connection;
use std::env;
use dotenvy::dotenv;
mod schema;

pub mod user_managment;
pub mod service_managment;

use std::sync::{Arc, Mutex};


pub const SERVICE_PORT_RANGE: (i32, i32) = (6001, 9000);

pub struct DbConn(pub Arc<Mutex<SqliteConnection>>);

impl DbConn {
    pub fn establish_connection() -> Self {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let connection = SqliteConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));
        Self(Arc::new(Mutex::new(connection)))
    }
}