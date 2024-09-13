use std::io::Cursor;
use std::time::Duration;
use crate::api_client::RithmicApiClient;
use crate::credentials::RithmicCredentials;
use crate::rithmic_proto_objects::rti::request_login::SysInfraType;
use crate::rithmic_proto_objects::rti::{RequestHeartbeat, ResponseHeartbeat};
use tokio::sync::mpsc::Receiver;
use crate::errors::RithmicApiError;
use prost::{Message as ProstMessage};
use prost::encoding::{decode_key, decode_varint, WireType};
use tokio::time::Instant;
use tungstenite::Message;
use crate::map::create_template_decoder_map;

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
  /*  let _history_receiver: Receiver<Message> = rithmic_api.connect_and_login(SysInfraType::HistoryPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::HistoryPlant).await);
    let _order_receiver: Receiver<Message> =rithmic_api.connect_and_login(SysInfraType::OrderPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::OrderPlant).await);
    let _pnl_receiver: Receiver<Message> =rithmic_api.connect_and_login(SysInfraType::PnlPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::PnlPlant).await);
    let _repo_receiver: Receiver<Message> =rithmic_api.connect_and_login(SysInfraType::RepositoryPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::RepositoryPlant).await);*/


    // send a heartbeat request as a test message, 'RequestHeartbeat' Template number 18
    let heart_beat = RequestHeartbeat {
        template_id: 18,
        user_msg: vec![format!("{} Testing heartbeat", app_name)],
        ssboe: None,
        usecs: None,
    };
    let _ = rithmic_api.send_message(&SysInfraType::TickerPlant, &heart_beat).await?;

    let map = create_template_decoder_map();
    receive::<ResponseHeartbeat>(ticker_receiver).await; //i think the key is to return a
    // Logout and Shutdown all connections
    rithmic_api.shutdown_all().await?;
    // or Logout and Shutdown a single connection
    //RithmicApiClient::shutdown_split_websocket(&rithmic_api, SysInfraType::TickerPlant).await?;

    Ok(())
}


pub async fn receive<T: ProstMessage + std::default::Default>(mut receiver: Receiver<Message>)   {
    let end_time = Instant::now() + Duration::from_secs(300); // 5 minutes from now
    while Instant::now() < end_time {
        if let Some( message) = receiver.recv().await {
            println!("{}", message);
            match message {
                Message::Text(text) => {
                    println!("{}", text)
                }
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

                    if let Some(template_id) = extract_template_id(&message_buf) {
                        println!("Extracted template_id: {}", template_id);

                        // Now you can use the template_id to determine which type to decode into
                        match template_id {
                            // Assuming each message type has a unique template_id
                            19 => {
                                if let Ok(msg) = ResponseHeartbeat::decode(&message_buf[..]) {
                                    println!("Decoded as AccountRmsUpdates: {:?}", msg);
                                }
                            },
                            // Add cases for other template_ids and corresponding message types
                            _ => println!("Unknown template_id: {}", template_id),
                        }
                    } else {
                        println!("Failed to extract template_id");
                    }
                }
                Message::Ping(ping) => {
                    println!("{:?}", ping)
                }
                Message::Pong(pong) => {
                    println!("{:?}", pong)
                }
                Message::Close(close) => {
                    println!("{:?}", close)
                }
                Message::Frame(frame) => {
                    println!("{}", frame)
                }
            }
        }
    }
}

fn extract_template_id(bytes: &[u8]) -> Option<i32> {
    let mut cursor = Cursor::new(bytes);

    while let Ok((field_number, wire_type)) = decode_key(&mut cursor) {
        if field_number == 154467 && wire_type == WireType::Varint {
            // We've found the template_id field
            return decode_varint(&mut cursor).ok().map(|v| v as i32);
        } else {
            // Skip this field
            match wire_type {
                WireType::Varint => { let _ = decode_varint(&mut cursor); }
                WireType::SixtyFourBit => { let _ = cursor.set_position(cursor.position() + 8); }
                WireType::LengthDelimited => {
                    if let Ok(len) = decode_varint(&mut cursor) {
                        let _ = cursor.set_position(cursor.position() + len as u64);
                    } else {
                        return None; // Error decoding length
                    }
                }
                WireType::StartGroup | WireType::EndGroup => {} // These are deprecated and shouldn't appear
                WireType::ThirtyTwoBit => { let _ = cursor.set_position(cursor.position() + 4); }
            }
        }
    }

    None // template_id field not found
}