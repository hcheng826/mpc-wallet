#[macro_use]
extern crate rocket;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::serde::{json::Json, Deserialize};
use rocket::{Request, Response};
use std::path::PathBuf;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct SignReq {
    msg: String,
    room_id: String,
}

#[post("/", format = "json", data = "<sign_req>")]
async fn sign(sign_req: Json<SignReq>) -> &'static str {
    let sign_result = match tss_sm_signing::sign(
        sign_req.msg.to_string(),
        PathBuf::from(r"./examples/local-share2.json"),
        vec![1, 2],
        surf::Url::parse("http://localhost:8000").unwrap(),
        // surf::Url::parse("https://4759-60-250-148-100.jp.ngrok.io").unwrap(),
        sign_req.room_id.to_string(),
    )
    .await
    {
        Ok(result) => result,
        Err(error) => format!("error in sign {:?}", error),
    };

    println!("sign_result {:?}", sign_result);

    "Server Good"
}

/// Catches all OPTION requests in order to get the CORS related Fairing triggered.
#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let figment = rocket::Config::figment().merge(("port", 8002));
    let _rocket_instance = rocket::custom(figment)
        .attach(Cors)
        .mount("/", routes![index])
        .mount("/sign", routes![sign, all_options])
        .launch()
        .await?;
    Ok(())
}

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, PATCH, PUT, DELETE, HEAD, OPTIONS, GET",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}
