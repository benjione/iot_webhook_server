use crate::models::device;
use crate::models::user::User;
use actix_identity::Identity;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::{get, http, post, web};
use diesel;
use diesel::r2d2::ConnectionManager;
use diesel::SqliteConnection;

use crate::models::device::NewDeviceForm;

type Pool = diesel::r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[post("/api/add_device")]
pub async fn add_device(
    device_form: web::Form<NewDeviceForm>,
    id: Identity,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    use self::diesel::prelude::*;
    use crate::schema::users;
    use crate::schema::users::dsl::email;

    let conn: &SqliteConnection = &pool.get().unwrap();

    // get user id
    let user_id = match users::table
        .filter(email.eq(id.identity().unwrap()))
        .limit(1)
        .load::<User>(conn)
    {
        Ok(user) => user[0].id,
        _ => {
            return Ok(HttpResponse::SeeOther()
                .header(http::header::LOCATION, "/login")
                .finish());
        }
    };

    // check if device already exists
    match device::Device::device_exists(user_id, device_form.name.clone(), &pool) {
        true => {
            return Ok(HttpResponse::SeeOther()
                .header(http::header::LOCATION, "/profile")
                .finish());
        }
        false => {}
    }

    // insert device into database
    let new_device = device::NewDevice::new(device_form.into_inner(), user_id);
    new_device.insert_into_database(pool);

    return Ok(HttpResponse::SeeOther()
        .header(http::header::LOCATION, "/profile")
        .finish());
}
