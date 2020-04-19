extern crate diesel;
extern crate dotenv;

use diesel::prelude::*;
use diesel::r2d2;
use dotenv::dotenv;

pub fn establish_connection() -> r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = r2d2::ConnectionManager::<SqliteConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed creating pool.")
}
