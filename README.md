# ff_rithmic_api
This rithmic api was written for fund-forge, an algorithmic trading platform written in rust.
It is currently incomlete but will eventually contain full functionality for rithmic RProtocol api.

## Login and connect
Step 1: Enter your api details provided by rithmic into the rithmic_credentials.toml \
Step 2: Load credentials and create an instance of a RithmicApiClient:
```rust
#[tokio::main]
async fn main() {
    // Define the file path for credentials
    let file_path = String::from("rithmic_credentials.toml".to_string());
    let credentials = RithmicCredentials::load_credentials_from_file(&file_path).unwrap();
    let rithmic_api = RithmicApiClient::new(credentials);
}
```
Step 3: Connect to the desired rithmic plant:
```rust
#[tokio::main]
async fn main() {
    // Define the file path for credentials
    let file_path = String::from("rithmic_credentials.toml".to_string());
    
    // load credentials
    let credentials = RithmicCredentials::load_credentials_from_file(&file_path).unwrap();
    
    //create api instance
    let rithmic_api = RithmicApiClient::new(credentials);
    
    // connect to plants
    rithmic_api.connect_and_login(SysInfraType::TickerPlant).await?;
    rithmic_api.connect_and_login(SysInfraType::HistoryPlant).await?;
    rithmic_api.connect_and_login(SysInfraType::OrderPlant).await?;
    rithmic_api.connect_and_login(SysInfraType::PnlPlant).await?;
    rithmic_api.connect_and_login(SysInfraType::RepositoryPlant).await?;
}
```

Step 4: Send messages to the desired plant over the `write half` of the plant websocket connection.
```rust
async fn main() {
    // Define the file path for credentials
    let file_path = String::from("rithmic_credentials.toml".to_string());

    // load credentials
    let credentials = RithmicCredentials::load_credentials_from_file(&file_path).unwrap();
    let app_name = credentials.app_name.clone();
    
    // login to the ticker plant
    rithmic_api.connect_and_login(SysInfraType::TickerPlant).await?;
    
    /// send a heartbeat request as a test message, 'RequestHeartbeat' Template number 18
    let heart_beat = RequestHeartbeat {
        template_id: 18,
        user_msg: vec![format!("{} Testing heartbeat", app_name)],
        ssboe: None,
        usecs: None,
    };
    
    let send_message = rithmic_api.send_message_split_streams(&SysInfraType::TickerPlant, &heart_beat).await?;
}
```

Step 5: The connections are maintained in the api instance, when work is done, logout from all connections gracefully.
```rust
#[tokio::main]
async fn main() {
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
    
    // Shutdown all connections
    rithmic_api.shutdown_all().await?;

    // or Logout and Shutdown a single connection
    rithmic_api.shutdown_split_websocket(SysInfraType::TickerPlant).await?;
    
    Ok(())
}
```