use crate::server;
use actix::Addr;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::{post, web};

#[post("/api/webhook")]
pub async fn webhook(
    webhook_data: web::Form<server::SendWebhook>,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<HttpResponse, Error> {
    srv.get_ref().do_send(webhook_data.into_inner());
    return Ok(HttpResponse::Ok().content_type("text/html").body("Ok"));
}
