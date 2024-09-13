use std::io::Cursor;
use std::thread::sleep;
use std::time::Duration;
use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use crate::api_client::RithmicApiClient;
use crate::credentials::RithmicCredentials;
use crate::rithmic_proto_objects::rti::request_login::SysInfraType;
use crate::rithmic_proto_objects::rti::{RequestHeartbeat, ResponseHeartbeat};
use crate::errors::RithmicApiError;
use prost::{Message as ProstMessage};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

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
    let ticker_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> = rithmic_api.connect_and_login(SysInfraType::TickerPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::TickerPlant).await);

    let _history_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> = rithmic_api.connect_and_login(SysInfraType::HistoryPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::HistoryPlant).await);

    let _order_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api.connect_and_login(SysInfraType::OrderPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::OrderPlant).await);

    let _pnl_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api.connect_and_login(SysInfraType::PnlPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::PnlPlant).await);

    let _repo_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api.connect_and_login(SysInfraType::RepositoryPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::RepositoryPlant).await);


    // send a heartbeat request as a test message, 'RequestHeartbeat' Template number 18
    let heart_beat = RequestHeartbeat {
        template_id: 18,
        user_msg: vec![format!("{} Testing heartbeat", app_name)],
        ssboe: None,
        usecs: None,
    };

    // We can send messages with only a reference to the client, so we can wrap our client in Arc or share it between threads and still utilise all associated functions.
    match rithmic_api.send_message(&SysInfraType::TickerPlant, &heart_beat).await {
        Ok(_) => println!("Heart beat sent"),
        Err(e) => eprintln!("Heartbeat send failed: {}", e)
    }

    handle_received_responses(&rithmic_api, ticker_receiver, SysInfraType::TickerPlant).await?;
    let _ = rithmic_api.send_message(&SysInfraType::TickerPlant, &heart_beat).await?;
    sleep(Duration::from_secs(200));
    // Logout and Shutdown all connections
    rithmic_api.shutdown_all().await?;

    // or Logout and Shutdown a single connection
    //RithmicApiClient::shutdown_split_websocket(&rithmic_api, SysInfraType::TickerPlant).await?;

    Ok(())
}

/// Due to the generic type T we cannot call this function directly on main.
/// we use extract_template_id() to get the template id using the field_number 154467 without casting to any concrete type, then we map to the concrete type and handle that message.
pub async fn handle_received_responses(
    client: &RithmicApiClient,
    mut reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    _plant: SysInfraType,
) -> Result<(), RithmicApiError> {
    //tokio::task::spawn(async move {
        while let Some(message) = reader.next().await {
            println!("{:?}", message);
            match message {
                Ok(message) => {
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

                            if let Some(template_id) = client.extract_template_id(&message_buf) {
                                println!("Extracted template_id: {}", template_id);
                                // Now you can use the template_id to determine which type to decode into
                                match template_id {
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
                Err(e) => {
                    eprintln!("failed to receive message: {}", e)
                }
            }
        }
    //});
    Ok(())
}

