use std::io::Cursor;
use crate::api_client::RithmicApiClient;
use crate::credentials::RithmicCredentials;
use crate::rithmic_proto_objects::rti::request_login::SysInfraType;
use crate::rithmic_proto_objects::rti::RequestHeartbeat;
use tokio::sync::mpsc::Receiver;
use crate::errors::RithmicApiError;

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
    let mut ticker_receiver: Receiver<Vec<u8>> = rithmic_api.connect_and_login(SysInfraType::TickerPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::TickerPlant).await);
    let _history_receiver: Receiver<Vec<u8>> = rithmic_api.connect_and_login(SysInfraType::HistoryPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::HistoryPlant).await);
    let _order_receiver: Receiver<Vec<u8>> =rithmic_api.connect_and_login(SysInfraType::OrderPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::OrderPlant).await);
    let _pnl_receiver: Receiver<Vec<u8>> =rithmic_api.connect_and_login(SysInfraType::PnlPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::PnlPlant).await);
    let _repo_receiver: Receiver<Vec<u8>> =rithmic_api.connect_and_login(SysInfraType::RepositoryPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::RepositoryPlant).await);


    // send a heartbeat request as a test message, 'RequestHeartbeat' Template number 18
    let heart_beat = RequestHeartbeat {
        template_id: 18,
        user_msg: vec![format!("{} Testing heartbeat", app_name)],
        ssboe: None,
        usecs: None,
    };
    let _ = rithmic_api.send_message(&SysInfraType::TickerPlant, &heart_beat).await?;



    // we can consume the messages like this.
    while let Some(message) = ticker_receiver.recv().await {
        //messages will be forwarded here
        let mut cursor = Cursor::new(message);

        // Read the 4-byte length header
        let mut length_buf = [0u8; 4];
        let _ = tokio::io::AsyncReadExt::read_exact(&mut cursor, &mut length_buf).await.map_err(RithmicApiError::Io);
        let length = u32::from_be_bytes(length_buf) as usize;

        // Read the Protobuf message
        let mut message_buf = vec![0u8; length];
        match tokio::io::AsyncReadExt::read_exact(&mut cursor, &mut message_buf).await.map_err(RithmicApiError::Io) {
            Ok(_) => {}
            Err(e) => eprintln!("Failed to read_extract message: {}", e)
        }

   /*     match RithmicMessage::decode(&message_buf[..]) {
            Ok(decoded_msg) =>{
                // we get our proto message as a concrete type here
                println!("{:?}", decoded_msg)
            }
            Err(e) => eprintln!("failed to decode message: {}", e), // Use the ProtobufDecode variant
        }*/

    }

    // Logout and Shutdown all connections
    rithmic_api.shutdown_all().await?;
    // or Logout and Shutdown a single connection
    //RithmicApiClient::shutdown_split_websocket(&rithmic_api, SysInfraType::TickerPlant).await?;

    Ok(())
}