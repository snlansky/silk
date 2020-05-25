use actix_web::{get, post, App, HttpRequest, HttpServer, Responder};
#[macro_use]
extern crate actix_web;

#[get("/ping")]
async fn pong(_req: HttpRequest) -> impl Responder {
    "pong"
}

// Register registers new user in ca server. In registration request attributes, affiliation and
// max enrolments must be set.
// It is responsibility of the SDK user to ensure passwords are with big entropy.
// Identity parameter is certificate for user that makes registration and this user MUST have the role for
// registering new users.
#[post("/api/v1/register")]
async fn register(_req: HttpRequest) -> impl Responder {
    "unimplemented!"
}

// Enroll execute enrollment request for registered user in CA server.
// On success new Identity with ECert and generated csr are returned.
#[post("/api/v1/enroll")]
async fn enroll(_req: HttpRequest) -> impl Responder {
    "unimplemented!"
}

// Revoke revokes ECert in ca server.
// Note that this request will revoke certificate ONLY in Ca server. Peers (for now) do not know
// about this certificate revocation.
// It is responsibility of the SDK user to update peers and set this certificate in every peer revocation list.
#[post("/api/v1/revoke")]
async fn revoke(_req: HttpRequest) -> impl Responder {
    "unimplemented!"
}

// ReEnroll create new certificate from old one. Useful when certificate is about to expire.
// Difference with `Enroll` is that `Enroll` require identity with `Registar` role.
// In re-enrolment the old certificate is used to identify the identity.
#[post("/api/v1/reenroll")]
async fn re_enroll(_req: HttpRequest) -> impl Responder {
    "unimplemented!"
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(pong)
            .service(register)
            .service(enroll)
            .service(revoke)
            .service(re_enroll)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
