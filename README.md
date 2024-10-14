# ff_rithmic_api
This rithmic api was written for [Fund Forge](https://github.com/BurnOutTrader/fund-forge), an algorithmic trading platform written in rust. (fund-forge available once live testing is underway).

available to import from crates.io as 'ff_rithmic_api'

The crate manages signing in and out of the various rithmic plants.

It contains all the rithmic proto api objects.

It is a base implementation for building a full api.

Be aware! Tests will fail when the market is closed.

You will need a servers.toml file for your API, you can use this template, you only need an address for the specific `RithmicServer`s that you intend to use.
## Servers
```toml
[rithmic_servers]
Chicago = "wss://{ASK_RITHMIC_FOR_DEV_KIT}"
Sydney = "wss://{ASK_RITHMIC_FOR_DEV_KIT}"
SaoPaolo = "wss://{ASK_RITHMIC_FOR_DEV_KIT}"
Colo75 = "wss://{ASK_RITHMIC_FOR_DEV_KIT}"
Frankfurt = "wss://{ASK_RITHMIC_FOR_DEV_KIT}"
HongKong = "wss://{ASK_RITHMIC_FOR_DEV_KIT}"
Ireland = "wss://{ASK_RITHMIC_FOR_DEV_KIT}"
Mumbai = "wss://{ASK_RITHMIC_FOR_DEV_KIT}"
Seoul = "wss://{ASK_RITHMIC_FOR_DEV_KIT}"
CapeTown = "wss://{ASK_RITHMIC_FOR_DEV_KIT}"
Tokyo = "wss://{ASK_RITHMIC_FOR_DEV_KIT}"
Singapore = "wss://{ASK_RITHMIC_FOR_DEV_KIT}"
Test = "wss://{ASK_RITHMIC_FOR_DEV_KIT}"
```

## Complete
This Api allows complete dynamic functionality for all Infrastructure Plants, Requests and Response types.
All possible proto responses and request are already compiled into rust code and they should be visible in your IDE by starting to type Response or Request. \
See [tests.rs](https://github.com/BurnOutTrader/ff_rithmic_api/blob/master/src/test.rs) for copy-paste function templates of all response message types for each rithmic plant connection variable. \
Hint: some Response types don't start with the word Response as shown in the Rithmic Docs, try typing the actual name of the response object or task eg: instead of "ReponseOrderBook" try typing "OrderBook".
## Not Included
No rate limiting. \
No Auto reconnect. \
No Heart Beat. \
This crate just handles the connection and returns the stream
Not ensuring SSL, we are using a  MaybeTlsStream, since the domain name is "wss://" I assume this is properly completing the handshake. \
Not thoroughly tested, if you experience a locking behaviour, try applying a lock to the fn `api_client.update_heartbeat():' or simply don't use it, I am not sure how this fn will keep up in async contexts if misused.

Note: If the Proto version is ever updated we will need to uncomment the build.rs code and rerun the build.
## Login and connect
Step 1a: Enter the server urls for each Server in server.toml, if you are only using Test you will only need to enter the url for Test, just leave the others as they are, I am not allowed to share them, you must apply for dev kit.

Step 1b: Enter your api details provided by rithmic into the rithmic_credentials.toml, if the toml does not exist, then you can create new credentials and save them to a file.

Step 2: Load credentials and create an instance of a RithmicApiClient:
```rust
#[tokio::main]
async fn main() {
    // On first run create the credentials
    let new_credentials = RithmicCredentials {
        user: "{ASK_RITHMIC_FOR_CREDENTIALS}".to_string(),
        app_name: "Example".to_string(),
        app_version: "1.0".to_string(),
        server_name: RithmicServer::Test,
        system_name: RithmicSystem::Test,
        password: "password".to_string(),
        fcm_id: Some("XXXFIRM".to_string()),
        ib_id: Some("XXXFIRM".to_string()),
        user_type: Some(UserType::Trader.into()),
        subscribe_data: true, 
        aggregated_quotes: false //for some reason using true does not parse correctly on the server side, I don't know what causes this
    };
    // Save credentials to file "rithmic_credentials.toml" is in the .gitignore
    new_credentials.save_credentials_to_file(new_credentials.file_name()).unwrap();

    // Define the file path for credentials
    let file_path = String::from("rithmic_credentials.toml".to_string());
    let credentials = RithmicCredentials::load_credentials_from_file(&file_path).unwrap();
    let rithmic_api = RithmicApiClient::new(credentials);
}
```
Step 3: Connect to a plant and the receiving half of the WebSocket for the specific plant will be returned
See examples.rs for a full copy paste handler for each plant type.
```rust
#[tokio::main]
async fn main() {
    let file_path = String::from("rithmic_credentials.toml".to_string());

    let new_credentials = RithmicCredentials {
        user: "{ASK_RITHMIC_FOR_CREDENTIALS}".to_string(),
        server_name: RithmicServer::Test,
        system_name: RithmicSystem::Test,
        app_name: "Example".to_string(),
        app_version: "1.0".to_string(),
        password: "password".to_string(),
        fcm_id: Some("XXXFIRM".to_string()),
        ib_id: Some("XXXFIRM".to_string()),
        user_type: Some(UserType::Trader.into()),
    };
    new_credentials.save_credentials_to_file(&file_path)?;

    // Define the file path for credentials


    // Define credentials
    let credentials = RithmicCredentials::load_credentials_from_file(&file_path).unwrap();
    let app_name: String = "".to_string();
    let app_version: String = "".to_string();
    let aggregated_quotes: bool = false;
    let server_domains_toml: String = "servers.toml".to_string();
    // Save credentials to file
    //credentials.save_credentials_to_file(&file_path)?;

    // Create a new RithmicApiClient instance
    let rithmic_api = RithmicApiClient::new(credentials, aggregated_quotes, server_domains_toml).unwrap();
    let rithmic_api_arc = Arc::new(rithmic_api);

    let (sender, mut receiver) = mpsc::channel(100);
    
    // the same need to be done for all SysInfraType Variants
    let order_stream:  WebSocketStream<MaybeTlsStream<TcpStream>> =rithmic_api_arc.connect_and_login(SysInfraType::OrderPlant).await?;

    let (order_writer, order_receiver) = order_stream.split();
    
    assert!(rithmic_api_arc.is_connected(SysInfraType::OrderPlant).await);
    handle_received_responses(rithmic_api_arc.clone(), order_receiver, SysInfraType::OrderPlant,sender).await?;

    let accounts = RequestAccountList {
        template_id: 302,
        user_msg: vec![],
        fcm_id: None,
        ib_id: None,
        user_type: Some(UserType::Trader.into())
    };
    
    // we can start or stop the async heartbeat task by updating our requirements, in a streaming situation heartbeat is not an api requirement.
    //rithmic_api_arc.switch_heartbeat_required(SysInfraType::TickerPlant, false).await.unwrap();
    // rithmic_api_arc.switch_heartbeat_required(SysInfraType::TickerPlant, true).await.unwrap();

    while let Some(_message) = receiver.recv().await {
        sleep(Duration::from_secs(10));
        break;
    }

    // Logout and Shutdown all connections
    rithmic_api_arc.shutdown_all().await?;
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
