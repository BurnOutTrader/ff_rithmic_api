use tokio::time::{sleep, Duration};
use crate::api_client::RithmicApiClient;
use crate::credentials::RithmicCredentials;
use crate::rithmic_proto_objects::rti::request_login::SysInfraType;
use crate::rithmic_proto_objects::rti::RequestHeartbeat;

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
    rithmic_api.connect_and_login(SysInfraType::TickerPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::TickerPlant).await);
    rithmic_api.connect_and_login(SysInfraType::HistoryPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::HistoryPlant).await);
    rithmic_api.connect_and_login(SysInfraType::OrderPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::OrderPlant).await);
    rithmic_api.connect_and_login(SysInfraType::PnlPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::PnlPlant).await);
    rithmic_api.connect_and_login(SysInfraType::RepositoryPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::RepositoryPlant).await);


    /// send a heartbeat request as a test message, 'RequestHeartbeat' Template number 18
    let heart_beat = RequestHeartbeat {
        template_id: 18,
        user_msg: vec![format!("{} Testing heartbeat", app_name)],
        ssboe: None,
        usecs: None,
    };
    let send_message = rithmic_api.send_message_split_streams(&SysInfraType::TickerPlant, &heart_beat).await?;

    // Sleep to simulate some work
    sleep(Duration::from_secs(5)).await;

    // Logout and Shutdown all connections
    rithmic_api.shutdown_all().await?;
    // or Logout and Shutdown a single connection
    //RithmicApiClient::shutdown_split_websocket(&rithmic_api, SysInfraType::TickerPlant).await?;

    Ok(())
}