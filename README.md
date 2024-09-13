# ff_rithmic_api
This rithmic api was written for [Fund Forge](https://github.com/BurnOutTrader/fund-forge), an algorithmic trading platform written in rust. (fund-forge available once live testing is underway).

The api is currently incomplete but will eventually contain full functionality for rithmic RProtocol api. 

## Work in Progress
Need to determine a way to dynamically decode messages without knowing the concrete types.

Note: If the Proto version is ever updated we will need to uncomment the build.rs code and rerun the build.
## Login and connect
Step 1: Enter your api details provided by rithmic into the rithmic_credentials.toml, if the toml does not exist, then you can create new credentials and save them to a file \
Step 2: Load credentials and create an instance of a RithmicApiClient:
```rust
#[tokio::main]
async fn main() {
    // On first run create the credentials
    let credentials = RithmicCredentials {
        user: "".to_string(),
        system_name: "".to_string(),
        password: "".to_string(),
        app_name: "".to_string(),
        app_version: "1.0".to_string(),
        aggregated_quotes: false,
        template_version: "5.27".to_string(),
        pem: String::from("rithmic_ssl_cert_auth_params.pem"),
        base_url: "wss://rituz00100.rithmic.com:443".to_string()
    };
    
    // Save credentials to file "rithmic_credentials.toml" is in the .gitignore
    credentials.save_credentials_to_file("rithmic_credentials.toml").unwrap();

    // Define the file path for credentials
    let file_path = String::from("rithmic_credentials.toml".to_string());
    let credentials = RithmicCredentials::load_credentials_from_file(&file_path).unwrap();
    let rithmic_api = RithmicApiClient::new(credentials);
}
```
Step 3: Connect to the desired rithmic plant and return a receiver which will receive the messages from the reader as the bytes from inside the received `tungstenite::Message`.
An event loop will start on the receiver side of the stream and a heartbeat will automatically be sent to keep the connection alive if no other messages have been sent.
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
}
```
## Parsing and Reading Messages
We might have to do this directly in the fwd_receive_responses function I am not sure yet if we can call this receive function without specifying a concrete type.
It might be easier to just make your own implementation of fwd_receive_responses and send messages as concrete types.
```rust

/// we use extract_template_id() to get the template id using the field_number 154467, then we map to teh concrete type and handle that message
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
    let mut ticker_receiver: Receiver<Message> = rithmic_api.connect_and_login(SysInfraType::TickerPlant, 100).await?;
    
    // check we connected, note this function will not automatically tell us if the websocket was disconnected after the initial connection
    if !rithmic_api.is_connected(SysInfraType::TickerPlant).await {
        return
    }
    
    /// send a heartbeat request as a test message, 'RequestHeartbeat' Template number 18
    let heart_beat = RequestHeartbeat {
        template_id: 18,
        user_msg: vec![format!("{} Testing heartbeat", app_name)],
        ssboe: None,
        usecs: None,
    };
    // we can send the message to the specified plant.
    let send_message = rithmic_api.send_message(&SysInfraType::TickerPlant, &heart_beat).await?;
    
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
    let mut ticker_receiver: Receiver<Message> = rithmic_api.connect_and_login(SysInfraType::TickerPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::TickerPlant).await);

    // Sleep to simulate some work
    sleep(Duration::from_secs(5)).await;
    
    // Shutdown all connections
    rithmic_api.shutdown_all().await?;

    // or Logout and Shutdown a single connection
    rithmic_api.shutdown_split_websocket(SysInfraType::TickerPlant).await?;
    
    Ok(())
}
```