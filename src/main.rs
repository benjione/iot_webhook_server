#[macro_use]
extern crate diesel;

use actix::*;
use actix_files as fs;
use actix_identity::Identity;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{error, get, http, web, App, Error, HttpResponse, HttpServer};
use diesel::SqliteConnection;

use tera::Context;
use tera::Tera;

mod api;
mod crypto;
mod db;
mod device;
mod models;
mod schema;
pub mod server;

use std::fs::File;
use std::io::BufReader;

use rustls::internal::pemfile::{certs, rsa_private_keys};
use rustls::{NoClientAuth, ServerConfig};

use models::device::Device;
use models::user;
type Pool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

fn check_logged_in_user(id: &Identity, mut context: Context) -> Context {
    if let Some(id) = id.identity() {
        context.insert("username", &format!("{}", id));
    }
    return context;
}

fn only_allowed_for_logged_in_user(id: &Identity) -> Option<HttpResponse> {
    match id.identity() {
        Some(_) => return None,
        _ => {
            return Some(
                HttpResponse::SeeOther()
                    .header(http::header::LOCATION, "/profile")
                    .finish(),
            )
        }
    }
}

#[get("/")]
async fn welcome(id: Identity, tmpl: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let mut context = Context::new();
    context = check_logged_in_user(&id, context);
    context.insert("title", &"welcome");
    let s = tmpl
        .render("welcome.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[get("/login")]
async fn login(id: Identity, tmpl: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let mut context = Context::new();
    context = check_logged_in_user(&id, context);
    context.insert("title", &"login");
    let s = tmpl
        .render("login.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[get("/signup")]
async fn signup(id: Identity, tmpl: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let mut context = Context::new();
    context = check_logged_in_user(&id, context);
    context.insert("title", &"signup");
    let s = tmpl
        .render("signup.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[get("/profile")]
async fn profile(
    id: Identity,
    tmpl: web::Data<tera::Tera>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    match only_allowed_for_logged_in_user(&id) {
        Some(ret) => return Ok(ret),
        _ => {}
    };
    let mut context = Context::new();
    context = check_logged_in_user(&id, context);
    context.insert("title", &"profile");
    let user = user::User::get_user_from_email(&id.identity().unwrap(), &pool).unwrap();
    let devices = Device::get_devices_for_user(user.id, &pool);
    context.insert("devices", &devices);
    let s = tmpl
        .render("profile.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

async fn error404(id: Identity, tmpl: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let mut context = Context::new();
    context = check_logged_in_user(&id, context);
    context.insert("title", &"404 - Page not found");
    let s = tmpl
        .render("error404.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let pool = db::establish_connection();

    // Start chat server actor
    let server = server::ChatServer::new(pool.clone()).start();

    let server_url = std::env::var("SERVER_URL").expect("SERVER_URL");

    let use_https = std::env::var("USE_HTTPS").expect("USE_HTTPS");


    // Create Http server with websocket support
    let server = HttpServer::new(move || {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        App::new()
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32]).secure(false),
            ))
            .data(tera)
            .data(server.clone())
            .data(pool.clone())
            // websocket
            .service(device::chat_route)
            .service(welcome)
            .service(signup)
            .service(login)
            .service(profile)
            .service(api::user::login)
            .service(api::user::signup)
            .service(api::user::logout)
            .service(api::device::add_device)
            .service(api::webhook::webhook)
            .service(
                // static files
                fs::Files::new("/static", "./static/"),
            )
            .service(
                // static files
                fs::Files::new("/js", "./script/"),
            )
            .default_service(web::to(error404))
    });

    if use_https == "TRUE" {
        let key_path = std::env::var("RSA_KEY").expect("RSA_KEY");
        let certificate_path = std::env::var("CERTIFICATE").expect("CERTIFICATE");

        // load ssl keys
        let mut config = ServerConfig::new(NoClientAuth::new());
        let cert_file = &mut BufReader::new(File::open(certificate_path).unwrap());
        let key_file = &mut BufReader::new(File::open(key_path).unwrap());
        let cert_chain = certs(cert_file).unwrap();
        let mut keys = rsa_private_keys(key_file).unwrap();
        config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

        return server.bind_rustls(server_url, config)?.run().await;
    }

    server.bind(server_url)?.run().await
}
