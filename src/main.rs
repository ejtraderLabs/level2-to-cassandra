use serde::{Deserialize, Serialize};
use serde_json;
use zmq::{Context, SocketType};
use std::env;
use tokio;
use scylla::Session;
use scylla::SessionBuilder;
use anyhow::Result;
use std::collections::HashMap;
use std::time::SystemTime;


async fn connect_to_cassandra(
    cassandra_host: &str,
    cassandra_username: &str,
    cassandra_password: &str,
    keyspace: &str,
) -> Result<Session> {
    let session = SessionBuilder::new()
        .known_node(cassandra_host)
        .user(cassandra_username, cassandra_password)
        .build()
        .await?;

    let create_keyspace_query = format!(
        "CREATE KEYSPACE IF NOT EXISTS {} WITH REPLICATION = {{ 'class' : 'SimpleStrategy', 'replication_factor' : 1 }};",
        keyspace
    );
    session.query(create_keyspace_query, &[]).await?;

    session
        .use_keyspace(keyspace, false)
        .await
        .expect("Failed to use keyspace");

    Ok(session)
}


#[derive(Serialize, Deserialize, Debug)]
struct BookData {
    symbol: String,
    price: f64,
    time: i64,
    volume: i32,
    #[serde(rename = "type")]
    order_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TickData {
    symbol: String,
    bid: f64,
    price: f64,
    ask: f64,
    time: i64,
    volume: i32,
    #[serde(rename = "type")]
    trade_type: String,
}

fn simplify_order_type(order_type: &str) -> String {
    order_type.replace("BOOK_TYPE_", "")
}

async fn cassandra_operations(
    session: &Session,
    keyspace: &str,
    topic: &str,
    topic_type: &[u8],
    msg: &[u8],
    cumulative_data: &mut HashMap<String, (i32, i32)>,
    last_processed_date: &mut Option<SystemTime>,
) -> Result<(), Box<dyn std::error::Error>> {
    match topic_type {
        b"BOOK" => {
            let book: Vec<BookData> = serde_json::from_slice(msg)?;

            // Create table if not exists
            let create_table_query = format!(
                "CREATE TABLE IF NOT EXISTS {}.{prefix}_book (
                    symbol text,
                    price double,
                    time timestamp,
                    volume int,
                    type text,
                    PRIMARY KEY (symbol,time,price)
                ) WITH CLUSTERING ORDER BY (time DESC);",
                keyspace,
                prefix = topic
            );
            session.query(create_table_query, &[]).await?;

            // Insert data
            for b in book {
                let insert_query = format!(
                    "INSERT INTO {}.{prefix}_book (symbol, price, time, volume, type) VALUES (?, ?, ?, ?, ?);",
                    keyspace,
                    prefix = topic
                );
                session
                    .query(insert_query, (b.symbol, b.price, b.time, b.volume, simplify_order_type(&b.order_type)))
                    .await?;
            }
        }
        b"TICK" => {
            let tick: TickData = serde_json::from_slice(msg)?;
            
            let tick_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(tick.time as u64);
            let tick_date = tick_time.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() / 86400;
            
            if let Some(last_date) = last_processed_date {
                let last_date = last_date.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() / 86400;
                
                if tick_date > last_date {
                    cumulative_data.insert(tick.symbol.clone(), (0, 0));
                }
            }

            *last_processed_date = Some(tick_time);
            
            let (cumbuy, cumsell) = cumulative_data.entry(tick.symbol.clone()).or_insert((0, 0));

            match tick.trade_type.as_str() {
                "B" => *cumbuy += tick.volume,
                "S" => *cumsell += tick.volume,
                _ => (),
            }

            let cumdelta = *cumbuy - *cumsell;

            // Create table if not exists
            let create_table_query = format!(
                "CREATE TABLE IF NOT EXISTS {}.{prefix}_tick (
                    symbol text,
                    bid double,
                    price double,
                    ask double,
                    time timestamp,
                    volume int,
                    type text,
                    cumbuy int,
                    cumsell int,
                    cumdelta int,
                    PRIMARY KEY (symbol,time,price)
                ) WITH CLUSTERING ORDER BY (time DESC);",
                keyspace,
                prefix = topic
            );
            
            
            session.query(create_table_query, &[]).await?;

            // Insert data
            let insert_query = format!(
                "INSERT INTO {}.{prefix}_tick (symbol, bid, price, ask, time, volume, type, cumbuy, cumsell, cumdelta) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
                keyspace,
                prefix = topic
            );
            session
                .query(
                    insert_query,
                    (tick.symbol, tick.bid, tick.price, tick.ask, tick.time, tick.volume, tick.trade_type.clone(), *cumbuy, *cumsell, cumdelta),
                )
                .await?;
        }
        _ => (),
    }

    Ok(())
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
let cassandra_host = env::var("CASSANDRA_HOST")?;
let cassandra_username = env::var("CASSANDRA_USERNAME")?;
let cassandra_password = env::var("CASSANDRA_PASSWORD")?;
let api_address = format!("tcp://{}", env::var("API_ADDRESS").unwrap());
let secret_key = env::var("SECRET_KEY")?;
let public_key = env::var("PUBLIC_KEY")?;
let server_key = env::var("SERVER_KEY")?;
let keyspace = env::var("KEYSPACE")?;
let topic = env::var("TOPIC")?; 
let ctx = Context::new();
let socket = ctx.socket(SocketType::SUB).expect("Failed to create subscriber socket");


socket
    .set_curve_publickey(public_key.as_bytes())
    .expect("Failed to set public key");
socket
    .set_curve_secretkey(secret_key.as_bytes())
    .expect("Failed to set secret key");
socket
    .set_curve_serverkey(server_key.as_bytes())
    .expect("Failed to set server key");

socket
    .connect(&api_address)
    .expect("Failed to connect to publisher");

socket
    .set_subscribe(topic.as_bytes())
    .expect("Failed to subscribe to topic");
let mut cumulative_data: HashMap<String, (i32, i32)> = HashMap::new();
let mut last_processed_date: Option<SystemTime> = None;

    
    let session = connect_to_cassandra(
        &cassandra_host,
        &cassandra_username,
        &cassandra_password,
        &keyspace,
    )
    .await?;

loop {
    let msg = socket.recv_multipart(0).unwrap();
    let topic = std::str::from_utf8(&msg[0]).expect("Failed to convert topic to string");
    let topic_type = &msg[1];

    match cassandra_operations(&session, &keyspace, topic, &topic_type, &msg[2], &mut cumulative_data, &mut last_processed_date).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Erro ao executar as operações do Cassandra: {}", e);
        }
    }
}
}