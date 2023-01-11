use dotenv::dotenv;
use futures::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// TODO: Will be replaced by DB
fn get_local_share_by_address(address: &str) -> Option<&str> {
    let address_book: HashMap<&str, &str> = HashMap::from([
        (
            "0x159D46720180113e2Ce97af425366778ECCcbA9C",
            "./shares/local-share2-0.json",
        ),
        (
            "0xF910Bd97b8F732Ce06c959DFDcE6De19623060B4",
            "./shares/local-share2-1.json",
        ),
        (
            "0x200dfB01148e580c59B53C6a35de9495cf10cf93",
            "./shares/local-share2-2.json",
        ),
        (
            "0xd1eD919ebF88baFab12FBCe1A6d1e1318a75b05b",
            "./shares/local-share2-3.json",
        ),
    ]);
    return address_book.get(address).copied();
}

lazy_static! {
    static ref RABBITMQ_HOST: String =
        std::env::var("RABBITMQ_HOST").expect("RABBITMQ_HOST should be set");
    static ref RABBITMQ_PORT: String =
        std::env::var("RABBITMQ_PORT").expect("RABBITMQ_PORT should be set");
    static ref RABBITMQ_SIGN_SIGNAL_QUEUE_NAME: String =
        std::env::var("RABBITMQ_SIGN_SIGNAL_QUEUE_NAME")
            .expect("RABBITMQ_SIGN_SIGNAL_QUEUE_NAME should be set");
    static ref SM_MANAGER_URL: String =
        std::env::var("SM_MANAGER_URL").expect("SM_MANAGER_URL should be set");
    static ref TX_SENDER_URL: String =
        std::env::var("TX_SENDER_URL").expect("TX_SENDER_URL should be set");
    static ref RABBITMQ_KEYGEN_SIGNAL_QUEUE_NAME: String =
        std::env::var("RABBITMQ_KEYGEN_SIGNAL_QUEUE_NAME")
            .expect("RABBITMQ_KEYGEN_SIGNAL_QUEUE_NAME should be set");
}

#[derive(Serialize, Deserialize, Debug)]
struct SignSignal {
    from_address: String,
    id: usize,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct RabbitMQDelivery {
    data: String,
    requestId: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct SignRes {
    success: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let conn = Connection::connect(
        &format!("amqp://{}:{}", *RABBITMQ_HOST, *RABBITMQ_PORT),
        ConnectionProperties::default(),
    )
    .await?;

    let channel = conn.create_channel().await?;
    let mut consume_options = BasicConsumeOptions::default();
    consume_options.no_ack = false;
    let mut sign_tx_consumer = channel
        .basic_consume(
            &*RABBITMQ_SIGN_SIGNAL_QUEUE_NAME,
            "sign-signal-consumer",
            consume_options,
            FieldTable::default(),
        )
        .await?;

    let mut keygen_consumer = channel
        .basic_consume(
            &*RABBITMQ_KEYGEN_SIGNAL_QUEUE_NAME,
            "keygen-signal-consumer",
            consume_options,
            FieldTable::default(),
        )
        .await?;

    let sign_consume_task = tokio::task::spawn(async move {
        while let Some(delivery) = sign_tx_consumer.next().await {
            tokio::task::spawn(async {
                let delivery = delivery.expect("error in consuming message");
                let delivery_str = std::str::from_utf8(&delivery.data)
                    .expect("cannot get data field from RabbitMQ message");
                let data = serde_json::from_str::<RabbitMQDelivery>(delivery_str)
                    .expect("error on parsing RabbitMQ message")
                    .data;
                let sign_data = serde_json::from_str::<SignSignal>(&data)
                    .expect("error on parsing sign signal");

                let local_share_path = get_local_share_by_address(&sign_data.from_address).unwrap();

                let sign_result = match tss_sm_client::sign(
                    sign_data.message.to_string(),
                    PathBuf::from(local_share_path),
                    vec![2, 1],
                    surf::Url::parse(&*SM_MANAGER_URL).unwrap(),
                    sign_data.id.to_string(),
                )
                .await
                {
                    Ok(result) => result,
                    Err(error) => format!("error in sign {:?}", error),
                };

                // send req to tx sender
                let client = reqwest::Client::new();
                let mut body_json = HashMap::new();
                body_json.insert("id", sign_data.id.to_string());
                body_json.insert("signature", sign_result);

                let _res = client
                    .post(format!("{}/submit-tx", *TX_SENDER_URL))
                    .json(&body_json)
                    .send()
                    .await
                    .expect("error on calling tx sender");
                delivery.ack(BasicAckOptions::default()).await.expect("ack");
            });
        }
    });

    let keygen_consume_task = tokio::task::spawn(async move {
        while let Some(delivery) = keygen_consumer.next().await {
            tokio::task::spawn(async {
                let delivery = delivery.expect("error in consuming message");
                let delivery_str = std::str::from_utf8(&delivery.data)
                    .expect("cannot get data field from RabbitMQ message");
                let data = serde_json::from_str::<RabbitMQDelivery>(delivery_str)
                    .expect("error on parsing RabbitMQ message")
                    .data;
                println!("data: {}", data);
                // let user_id =
                //     serde_json::from_str::<String>(&data).expect("error on parsing user_id");

                // let local_share_path = get_local_share_by_address(&sign_data.from_address).unwrap();

                let _sign_result = match tss_sm_client::keygen(
                    surf::Url::parse(&*SM_MANAGER_URL).unwrap(),
                    data,
                    1,
                    1,
                    2,
                )
                .await
                {
                    Ok(result) => serde_json::to_string(&result).unwrap(),
                    Err(error) => format!("error in keygen {:?}", error),
                };

                // println!("{:?}", sign_result);
                // TODO: write sign_result to db or save to file
                delivery.ack(BasicAckOptions::default()).await.expect("ack");
            });
        }
    });

    let (_task_result, _req_result) = tokio::join!(sign_consume_task, keygen_consume_task);

    Ok(())
}
