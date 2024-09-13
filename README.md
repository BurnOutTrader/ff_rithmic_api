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

Step 4: When work is done, logout from all connections gracefully:
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
    Ok(())
}
```