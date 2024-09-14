# ff_rithmic_api
This rithmic api was written for [Fund Forge](https://github.com/BurnOutTrader/fund-forge), an algorithmic trading platform written in rust. (fund-forge available once live testing is underway).

The api is currently contains the full functionality for rithmic RProtocol api. 

## Complete
This Api allows complete dynamic functionality for all Infrastructure Plants, Requests and Response types.
I will hard code the responses and the template_id's into a unique `fn handle_received_responses()` for each rithmic plant, so that they can just be copy pasted in the future.
All possible proto responses are already compiled into rust code and so they should be visible in your IDE by starting to type Response.

## Not Done
No rate limiting. \
No Auto reconnect. \
Not ensuring SSL, we are using a  MaybeTlsStream, since the domain name is "wss://" I assume this is properly completing the handshake.

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
Step 3: Connect to a plant and the receiving half of the WebSocket for the specific plant will be returned
```rust
#[tokio::main]
async fn main() {
    // Define the file path for credentials
    let file_path = String::from("rithmic_credentials.toml".to_string());
    
    // load credentials
    let credentials = RithmicCredentials::load_credentials_from_file(&file_path).unwrap();
    
    //create api instance
    let rithmic_api = RithmicApiClient::new(credentials);
    
    // connect to plants and get the receiving half of the WebSocket for the specific plant returned
    let mut ticker_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> = rithmic_api.connect_and_login(SysInfraType::TickerPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::TickerPlant).await);

    let _history_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> = rithmic_api.connect_and_login(SysInfraType::HistoryPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::HistoryPlant).await);

    let _order_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api.connect_and_login(SysInfraType::OrderPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::OrderPlant).await);

    let _pnl_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api.connect_and_login(SysInfraType::PnlPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::PnlPlant).await);

    let _repo_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api.connect_and_login(SysInfraType::RepositoryPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::RepositoryPlant).await);
}
```
## Parsing and Reading Messages
We can use the receiver of the websocket connection to receive the `prost::Message`s from rithmic anywhere in our code base, Note that in the examples I am importing `use prost::{Message as ProstMessage};`.
To send messages to rithmic we will only need a reference to the specific `RithmicApiClient` instance.
We do not need a mutable client to send messages to rithmic as the writer half of the stream is stored in a DashMap.
```rust

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

    // Establish connections, sign in and receive back the websocket readers
    let ticker_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> = rithmic_api.connect_and_login(SysInfraType::TickerPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::TickerPlant).await);

    let _history_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> = rithmic_api.connect_and_login(SysInfraType::HistoryPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::HistoryPlant).await);

    let _order_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api.connect_and_login(SysInfraType::OrderPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::OrderPlant).await);

    let _pnl_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api.connect_and_login(SysInfraType::PnlPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::PnlPlant).await);

    let _repo_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api.connect_and_login(SysInfraType::RepositoryPlant).await?;
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

    // Logout and Shutdown all connections
    rithmic_api.shutdown_all().await?;

    // or Logout and Shutdown a single connection
    //RithmicApiClient::shutdown_split_websocket(&rithmic_api, SysInfraType::TickerPlant).await?;

    Ok(())
}

/// we use extract_template_id() to get the template id using the field_number 154467 without casting to any concrete type, then we map to the concrete type and handle that message.
pub async fn handle_received_responses(
    client: &RithmicApiClient,
    mut reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    _plant: SysInfraType,
) -> Result<(), RithmicApiError> {
    //tokio::task::spawn(async move {
    while let Some(message) = reader.next().await {
        println!("Message received: {:?}", message);
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
                                        println!("Decoded as: {:?}", msg);

                                        // now send a gateway info request to test that we can actually parse multiple types
                                        let request = RequestRithmicSystemGatewayInfo {
                                            template_id: 20,
                                            user_msg: vec![],
                                            system_name: Some(client.get_system_name(&plant).await.unwrap()),
                                        };
                                        client.send_message(&plant, &request).await?
                                    }
                                },
                                21 => {
                                    if let Ok(msg) = ResponseRithmicSystemInfo::decode(&message_buf[..]) {
                                        println!("Decoded as: {:?}", msg);
                                        //for the sake of the example I am breaking the loop early
                                        break;
                                    }
                                }
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
    let mut ticker_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> = rithmic_api.connect_and_login(SysInfraType::TickerPlant, 100).await?;
    
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
    let mut ticker_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> = rithmic_api.connect_and_login(SysInfraType::TickerPlant, 100).await?;
    assert!(rithmic_api.is_connected(SysInfraType::TickerPlant).await);

    // Sleep to simulate some work
    sleep(Duration::from_secs(5)).await;
    
    // Shutdown all connections
    rithmic_api.shutdown_all().await?;

    // or Logout and Shutdown a single connection
    rithmic_api.shutdown_plant(SysInfraType::TickerPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::TickerPlant).await == false);
    
    Ok(())
}
```