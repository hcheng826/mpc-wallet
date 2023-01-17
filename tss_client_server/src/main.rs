pub mod db;

#[macro_use]
extern crate rocket;
use dotenv::dotenv;
use reqwest::Client;
use rocket::serde::{json::Json, Deserialize, Serialize};
use std::vec;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct SendTxReq {
    from_address: String,
    tx_data: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct SendTxRes {
    success: bool,
    info: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct TxSenderRes {
    success: bool,
    message_to_sign: String,
    id: usize,
}

lazy_static::lazy_static! {
    static ref PORT: String = std::env::var("PORT").expect("PORT should be set");
    static ref TX_SENDER_URL: String = std::env::var("TX_SENDER_URL").expect("TX_SENDER_URL should be set");
    static ref SM_MANAGER_URL: String = std::env::var("SM_MANAGER_URL").expect("SM_MANAGER_URL should be set");
}

#[post("/send-tx", format = "json", data = "<send_tx_req>")]
async fn send_tx(send_tx_req: Json<SendTxReq>) -> Json<SendTxRes> {
    // talk to tx sender for simulation
    let client = Client::new();
    let mut body = std::collections::HashMap::new();
    body.insert("from_address", &send_tx_req.from_address);
    body.insert("tx_data", &send_tx_req.tx_data);
    let call_tx_sender_result = client
        .post(format!("{}{}", *TX_SENDER_URL, "/request-tx"))
        .json(&body)
        .send()
        .await;

    let res_text_result = match call_tx_sender_result {
        Ok(res) => res.text().await,
        Err(_) => {
            return Json(SendTxRes {
                success: false,
                info: Some("fail to call tx sender".to_string()),
            })
        }
    };

    let res_parsed_result: Result<TxSenderRes, serde_json::Error> = match res_text_result {
        Ok(res_text) => serde_json::from_str(&res_text),
        Err(_) => {
            return Json(SendTxRes {
                success: false,
                info: Some("fail to get tx sender response text".to_string()),
            })
        }
    };

    let tx_sender_res = match res_parsed_result {
        Ok(tx_sender_res) => tx_sender_res,
        Err(_) => {
            return Json(SendTxRes {
                success: false,
                info: Some("fail on parsing tx sender response".to_string()),
            })
        }
    };

    println!("tx_sender_res.success: {}", tx_sender_res.success);
    if tx_sender_res.success == false {
        return Json(SendTxRes {
            success: false,
            info: Some("tx simulation failed".to_string()),
        });
    }

    // talk to SM
    let db_conn = &mut db::establish_connection();
    let local_share = db::get_local_share(db_conn, &send_tx_req.from_address).expect("cannot get local_share from db");

    // TODO: implement timeout for this function
    let _sigature = match tss_sm_client::sign(
        tx_sender_res.message_to_sign,
        local_share,
        vec![1, 2],
        surf::Url::parse(&SM_MANAGER_URL).unwrap(),
        tx_sender_res.id.to_string(),
    )
    .await
    {
        Ok(result) => result,
        Err(error) => format!("error in sign {:?}", error),
    };

    println!("signature: {}", _sigature);
    Json(SendTxRes {
        success: true,
        info: None,
    })
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct NewKeyRes {
    success: bool,
    user_id: String,
    address: Option<String>,
    info: Option<String>,
}

#[post("/new-key")]
async fn new_key() -> Json<NewKeyRes> {
    let db_conn = &mut db::establish_connection();
    let new_key_id = db::insert_new_key(db_conn);

    let client = Client::new();
    let mut body = std::collections::HashMap::new();
    body.insert("userId", new_key_id.to_string());
    let call_tx_sender_result = client
        .post(format!("{}{}", *TX_SENDER_URL, "/new-key"))
        .json(&body)
        .send()
        .await;

    let _res = match call_tx_sender_result {
        Ok(res) => res.text().await,
        Err(_) => {
            return Json(NewKeyRes {
                success: false,
                user_id: new_key_id.to_string(),
                address: None,
                info: Some("fail to call tx sender".to_string()),
            })
        }
    };

    let local_key = tss_sm_client::keygen(
        surf::Url::parse(&SM_MANAGER_URL).unwrap(),
        new_key_id.to_string(),
        2,
        1,
        2,
    )
    .await
    .expect("unwrap local key");

    let pubkey_string =
        serde_json::to_string(&local_key.y_sum_s).expect("error parsing local_key.y_sum_s");
    db::fill_in_key_data(
        db_conn,
        new_key_id,
        &pubkey_string,
        &serde_json::to_string(&local_key).expect("error parsing local_key"),
    );

    println!("key generated. pub key: {}", pubkey_string);
    Json(NewKeyRes {
        success: true,
        user_id: new_key_id.to_string(),
        address: Some(pubkey_string),
        info: None,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let figment = rocket::Config::figment()
        .merge(("port", PORT.parse::<u16>().unwrap()))
        .merge(("address", "0.0.0.0"));
    let _rocket_instance = rocket::custom(figment)
        .mount("/", routes![index, send_tx, new_key])
        .launch()
        .await?;
    Ok(())
}
