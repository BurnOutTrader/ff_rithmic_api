# ff_rithmic_api
This rithmic api was written for [Fund Forge](https://github.com/BurnOutTrader/fund-forge), an algorithmic trading platform written in rust. (fund-forge available once live testing is underway).

The api is currently contains the full functionality for rithmic RProtocol api. 

Be aware! Tests will fail when the market is closed.

## Complete
This Api allows complete dynamic functionality for all Infrastructure Plants, Requests and Response types.
All possible proto responses and request are already compiled into rust code and they should be visible in your IDE by starting to type Response or Request. \
See [tests.rs](https://github.com/BurnOutTrader/ff_rithmic_api/blob/master/src/test.rs) for copy-paste function templates of all response message types for each rithmic plant connection variable. \
Hint: some Response types don't start with the word Response as shown in the Rithmic Docs, try typing the actual name of the response object or task eg: instead of "ReponseOrderBook" try typing "OrderBook".
## Not Included
No rate limiting. \
No Auto reconnect. \
Not ensuring SSL, we are using a  MaybeTlsStream, since the domain name is "wss://" I assume this is properly completing the handshake.
Not thoroughly tested, if you experience a locking behaviour, try applying a lock to the fn `api_client.update_heartbeat();` I am not sure how this fn will keep up in async contexts if misused.

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
        base_url: "wss://rituz00100.rithmic.com:443".to_string(),
        broker: "Test"
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

    // Establish connections, sign in and receive back the websocket readers
    let ticker_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> = rithmic_api.connect_and_login(SysInfraType::TickerPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::TickerPlant).await);

    let _history_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> = rithmic_api.connect_and_login(SysInfraType::HistoryPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::HistoryPlant).await);

    let _order_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api.connect_and_login(SysInfraType::OrderPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::OrderPlant).await);

    let _pnl_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api.connect_and_login(SysInfraType::PnlPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::PnlPlant).await);

    // Only for signing contracts etc
    let _repo_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api.connect_and_login(SysInfraType::RepositoryPlant).await?;
    assert!(rithmic_api.is_connected(SysInfraType::RepositoryPlant).await);
}
```
## Parsing and Reading Messages
You receive a tokio_tungstenite::tungstenite::protocol::Message containing a prost::Message, referred to as ProstMessage. If you attempt to treat the original message directly as a ProstMessage, you will encounter the following compile-time error:
```
error[E0782]: trait objects must include the dyn keyword
–> rithmic_api/handle_tick_plant.rs:xx:xx
|
24 |                         ProstMessage::Text(text) => {
|                         ^^^^^^^^^^^^
|
help: add `dyn` keyword before this trait
|
24 |                         ::Text(text) => {
|                         ++++
```
This is how it should be
```rust
use tokio_tungstenite::tungstenite::protocol::Message;
use prost::{Message as ProstMessage};
fn example() {
while let Some(message) = reader.next().await {
    println!("Message received: {:?}", message);
    match message {
        Ok(message) => {
            match message {
                // This is a tungstenite::protocol::Message
                Message::Binary(vector_bytes) => {
                    
                    // The bytes are a prost::Message as ProstMessage
                    println!("{}", bytes)
                }
                // NOT THIS!
               /* ProstMessage::Binary(vector_bytes) => {

                    // The bytes are a prost::Message as ProstMessage
                    println!("{}", bytes)
                }*/
            }
        }
    }
}

```

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
    
    // on receiving messages we can manually reset the heartbeat timer
    rithmic_api.update_heartbeat(SysInfraType::TickerPlant);
    
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
                rithmic_api.update_heartbeat(SysInfraType::TickerPlant);
                match message {
                    tokio_tungstenite::tungstenite::protocol::Message::Text(text) => {
                        println!("{}", text)
                    }
                    tokio_tungstenite::tungstenite::protocol::Message::Binary(bytes) => {
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
                    tokio_tungstenite::tungstenite::protocol::Message::Ping(ping) => {
                        println!("{:?}", ping)
                    }
                    tokio_tungstenite::tungstenite::protocol::Message::Pong(pong) => {
                        println!("{:?}", pong)
                    }
                    tokio_tungstenite::tungstenite::protocol::Message::Close(close) => {
                        // receive this message when market is closed.
                        // received: Ok(Close(Some(CloseFrame { code: Normal, reason: "normal closure" })))
                        println!("{:?}", close)
                    }
                    tokio_tungstenite::tungstenite::protocol::Message::Frame(frame) => {
                        //This message is sent on weekends, you can use this message to schedule a reconnection attempt for market open.
                        /* Example of received market closed message
                            Some(CloseFrame { code: Normal, reason: "normal closure" })
                            Error: ServerErrorDebug("Failed to send RithmicMessage, possible disconnect, try reconnecting to plant TickerPlant: Trying to work with closed connection")
                        */
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