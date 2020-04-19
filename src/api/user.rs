use crate::models::user::{SignIn, SignUp, User};
use actix_identity::Identity;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::{get, http, post, web};
use diesel::r2d2::ConnectionManager;
use diesel::SqliteConnection;

type Pool = diesel::r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[post("/login")]
pub async fn login(
    user_data: web::Form<SignIn>,
    id: Identity,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    // for ownsership purposes
    let email = user_data.email.clone();
    if User::check_user_validity(&user_data, pool) {
        // for some reason in herer user data is still borrowed.
        id.remember(email);
        return Ok(HttpResponse::SeeOther()
            .header(http::header::LOCATION, "/")
            .finish());
    }
    Ok(HttpResponse::SeeOther()
        .header(http::header::LOCATION, "/login")
        .finish())
}

#[post("/signup")]
pub async fn signup(
    new_user: web::Form<SignUp>,
    id: Identity,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    let res = new_user
        .into_inner()
        .check_data_and_insert_to_database(pool);
    match res {
        Ok(user) => {
            id.remember(user.email);
            return Ok(HttpResponse::SeeOther()
                .header(http::header::LOCATION, "/")
                .finish());
        }
        Err(_) => {
            return Ok(HttpResponse::SeeOther()
                .header(http::header::LOCATION, "/login")
                .finish());
        }
    }
}

#[get("/logout")]
pub async fn logout(id: Identity, _pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    id.forget();
    return Ok(HttpResponse::SeeOther()
        .header(http::header::LOCATION, "/")
        .finish());
}
