use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::time::Duration;
use crate::api_client::RithmicApiClient;
use crate::credentials::RithmicCredentials;
use crate::rithmic_proto_objects::rti::request_login::SysInfraType;
use crate::rithmic_proto_objects::rti::{RequestHeartbeat, ResponseHeartbeat};
use tokio::sync::mpsc::Receiver;
use regex::Regex;
use crate::errors::RithmicApiError;
use walkdir::WalkDir;
use prost::{Message as RithmicMessage};
use tokio::time::Instant;
use tungstenite::Message;

// Decoder function type alias
type DecoderFn = fn(Vec<u8>) -> Box<dyn RithmicMessage>;

// Define your decoding functions for each message type

#[tokio::test]
async fn test_rithmic_connection() -> Result<(), Box<dyn std::error::Error>> {
    // Define the file path for credentials
    let file_path = String::from("rithmic_credentials.toml".to_string());

    // Define credentials
    let credentials = RithmicCredentials::load_credentials_from_file(&file_path).unwrap();
    let app_name = credentials.app_name.clone();
    // Save credentials to file
    //credentials.save_credentials_to_file(&file_path)?;

    // Create a new RithmicApiClient instance
    let rithmic_api = RithmicApiClient::new(credentials);

    // Test connections
    let mut ticker_receiver: Receiver<Message> = rithmic_api.connect_and_login(SysInfraType::TickerPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::TickerPlant).await);
    let _history_receiver: Receiver<Message> = rithmic_api.connect_and_login(SysInfraType::HistoryPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::HistoryPlant).await);
    let _order_receiver: Receiver<Message> =rithmic_api.connect_and_login(SysInfraType::OrderPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::OrderPlant).await);
    let _pnl_receiver: Receiver<Message> =rithmic_api.connect_and_login(SysInfraType::PnlPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::PnlPlant).await);
    let _repo_receiver: Receiver<Message> =rithmic_api.connect_and_login(SysInfraType::RepositoryPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::RepositoryPlant).await);


    // send a heartbeat request as a test message, 'RequestHeartbeat' Template number 18
    let heart_beat = RequestHeartbeat {
        template_id: 18,
        user_msg: vec![format!("{} Testing heartbeat", app_name)],
        ssboe: None,
        usecs: None,
    };
    let _ = rithmic_api.send_message(&SysInfraType::TickerPlant, &heart_beat).await?;



    let end_time = Instant::now() + Duration::from_secs(300); // 5 minutes from now
    while Instant::now() < end_time {
        if let Some( message) = ticker_receiver.recv().await {
            println!("{}", message);
            match message {
                Message::Text(_) => {}
                Message::Binary(bytes) => {
                    //messages will be forwarded here
                    let mut cursor = Cursor::new(bytes);
                    // Read the 4-byte length header
                    let mut length_buf = [0u8; 4];
                    let _ = tokio::io::AsyncReadExt::read_exact(&mut cursor, &mut length_buf).await.map_err(RithmicApiError::Io);
                    let length = u32::from_be_bytes(length_buf) as usize;
                    println!("Length: {}", length);

                    // Read the Protobuf message
                    let mut message_buf = vec![0u8; length];
                    match tokio::io::AsyncReadExt::read_exact(&mut cursor, &mut message_buf).await.map_err(RithmicApiError::Io) {
                        Ok(_) => {}
                        Err(e) => eprintln!("Failed to read_extract message: {}", e)
                    }

                    // Create a cursor to wrap the remaining data in the buffer.
                    let mut cursor = Cursor::new(&message_buf);
                    match ResponseHeartbeat::decode(&mut cursor) {
                        Ok(decoded_msg) => {
                            println!("{:?}", decoded_msg);
                        }
                        Err(e) => {
                            eprintln!("Failed to decode message: {}", e);
                        }
                    }
                }
                Message::Ping(_) => {}
                Message::Pong(_) => {}
                Message::Close(_) => {}
                Message::Frame(_) => {}
            }
        }
    }

    // Logout and Shutdown all connections
    rithmic_api.shutdown_all().await?;
    // or Logout and Shutdown a single connection
    //RithmicApiClient::shutdown_split_websocket(&rithmic_api, SysInfraType::TickerPlant).await?;

    Ok(())
}


/*
println!("attemot to decode");
                let template = u16::from_be_bytes([buffer[0], buffer[1]]);
                println!("template id: {}", template);

*/