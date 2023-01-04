#[macro_use]
extern crate rocket;
use reqwest::Client;
use std::path::PathBuf;
use tokio::task;
use uuid::Uuid;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/", format = "plain", data = "<serialized_tx>")]
async fn send_tx(serialized_tx: &str) -> String {
    let room_id = Uuid::new_v4();

    let sign_task = task::spawn(tss_sm_signing::sign(
        serialized_tx.to_string(),
        PathBuf::from(r"./examples/local-share1.json"),
        vec![1, 2],
        surf::Url::parse("http://localhost:8000").unwrap(),
        // surf::Url::parse("https://4759-60-250-148-100.jp.ngrok.io").unwrap(),
        room_id.to_string(),
    ));
    let serialized_tx_clone = serialized_tx.to_string();

    let req_server_sign_task = task::spawn(async move {
        let client = Client::new();
        let mut body = std::collections::HashMap::new();
        body.insert("msg", serialized_tx_clone);
        body.insert("room_id", room_id.to_string());
        let _res = client
            .post("http://localhost:8002/sign")
            // .post("https://d56b-60-250-148-100.jp.ngrok.io/sign")
            .json(&body)
            .send()
            .await;
    });

    let (sign_task_result, req_result) = tokio::join!(sign_task, req_server_sign_task);

    match req_result {
        Ok(()) => (),
        Err(error) => println!("error in server req {:?}", error),
    };

    let signature = match sign_task_result {
        Ok(task_result) => match task_result {
            Ok(signature) => signature,
            Err(err) => format!("error in sign {:?}", err),
        },
        Err(error) => format!("error in sign {:?}", error),
    };

    signature
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let figment = rocket::Config::figment()
        .merge(("port", 8001))
        .merge(("address", "0.0.0.0"));
    let _rocket_instance = rocket::custom(figment)
        .mount("/", routes![index])
        .mount("/send-tx", routes![send_tx])
        .launch()
        .await?;
    Ok(())
}
