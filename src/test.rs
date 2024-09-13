use tokio::time::{sleep, Duration};
use crate::api_client::RithmicApiClient;
use crate::credentials::RithmicCredentials;
use crate::rithmic_proto_objects::rti::request_login::SysInfraType;

#[tokio::test]
async fn test_rithmic_connection() -> Result<(), Box<dyn std::error::Error>> {
    // Define the file path for credentials
    let file_path = String::from("rithmic_credentials.toml".to_string());

    // Define credentials
    let credentials = RithmicCredentials::load_credentials_from_file(&file_path).unwrap();

    // Save credentials to file
    //credentials.save_credentials_to_file(&file_path)?;

    // Create a new RithmicApiClient instance
    let rithmic_api = RithmicApiClient::new(credentials);

    // Test connections
    rithmic_api.connect_and_login(SysInfraType::TickerPlant).await?;
    rithmic_api.connect_and_login(SysInfraType::HistoryPlant).await?;
    rithmic_api.connect_and_login(SysInfraType::OrderPlant).await?;
    rithmic_api.connect_and_login(SysInfraType::PnlPlant).await?;
    rithmic_api.connect_and_login(SysInfraType::RepositoryPlant).await?;

    // Sleep to simulate some work
    sleep(Duration::from_secs(5)).await;

    // Logout and Shutdown all connections
    RithmicApiClient::shutdown_all(&rithmic_api).await?;
    // or Logout and Shutdown a single connection
    //RithmicApiClient::shutdown_split_websocket(&rithmic_api, SysInfraType::TickerPlant).await?;

    Ok(())
}