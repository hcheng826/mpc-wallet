#[macro_use]
extern crate rocket;
use dotenv::dotenv;
use reqwest::Client;
use rocket::serde::{json::Json, Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::vec;
use uuid::Uuid;

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

// TODO: Will be replaced by DB
fn get_local_share_by_address(address: &str) -> Option<&str> {
    let address_book: HashMap<&str, &str> = HashMap::from([
        (
            "0x159D46720180113e2Ce97af425366778ECCcbA9C",
            "./shares/local-share1-0.json",
        ),
        (
            "0xF910Bd97b8F732Ce06c959DFDcE6De19623060B4",
            "./shares/local-share1-1.json",
        ),
        (
            "0x200dfB01148e580c59B53C6a35de9495cf10cf93",
            "./shares/local-share1-2.json",
        ),
        (
            "0xd1eD919ebF88baFab12FBCe1A6d1e1318a75b05b",
            "./shares/local-share1-3.json",
        ),
    ]);
    return address_book.get(address).copied();
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
    let local_share_path = match get_local_share_by_address(&send_tx_req.from_address) {
        Some(local_share_path) => local_share_path,
        None => {
            return Json(SendTxRes {
                success: false,
                info: Some("cannot find local-share path by address".to_string()),
            })
        }
    };

    // TODO: implement timeout for this function
    let _singature = match tss_sm_client::sign(
        tx_sender_res.message_to_sign,
        PathBuf::from(local_share_path),
        vec![2, 1],
        surf::Url::parse(&SM_MANAGER_URL).unwrap(),
        tx_sender_res.id.to_string(),
    )
    .await
    {
        Ok(result) => result,
        Err(error) => format!("error in sign {:?}", error),
    };

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
    address: Option<Vec<u8>>,
    info: Option<String>,
}

#[post("/new-key")]
async fn new_key() -> Json<NewKeyRes> {
    // TODO: replace by DB
    let user_id = Uuid::new_v4();

    let client = Client::new();
    let mut body = std::collections::HashMap::new();
    body.insert("userId", user_id.to_string());
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
                user_id: user_id.to_string(),
                address: None,
                info: Some("fail to call tx sender".to_string()),
            })
        }
    };

    println!("user_id: {}", user_id);
    let _local_key = tss_sm_client::keygen(
        surf::Url::parse(&SM_MANAGER_URL).unwrap(),
        user_id.to_string(),
        2,
        1,
        2,
    )
    .await
    .expect("unwrap local key");

    // println!("{:?}", local_key.y_sum_s);
    // TODO: write local_key to db or save to file

    Json(NewKeyRes {
        success: true,
        user_id: user_id.to_string(),
        address: Some([1, 2, 3].to_vec()),
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
