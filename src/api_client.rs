use std::io::{Cursor};
use prost::{Message as RithmicMessage};
use tokio::io::{AsyncReadExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use std::error::Error;
use std::sync::{Arc};
use dashmap::DashMap;
use futures_util::stream::{SplitSink, SplitStream};
use tokio::sync::{Mutex, RwLock};
use crate::credentials::RithmicCredentials;
use crate::rithmic_proto_objects::rti::request_login::SysInfraType;
use crate::rithmic_proto_objects::rti::{RequestLogin, RequestLogout, RequestRithmicSystemInfo, ResponseLogin, ResponseRithmicSystemInfo};
use crate::RithmicApiError;

///Server uses Big Endian format for binary data
pub struct RithmicApiClient {
    credentials: RithmicCredentials,
    connected_plant: Arc<RwLock<Vec<SysInfraType>>>,
    plant_writer:Arc<DashMap<SysInfraType, Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>>>,
    plant_reader: Arc<DashMap<SysInfraType, Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>>>,
}

impl RithmicApiClient {
    pub fn new(credentials: RithmicCredentials) -> Self {
        Self {
            credentials,
            connected_plant: Default::default(),
            plant_writer: Arc::new(DashMap::new()),
            plant_reader: Arc::new(DashMap::new()),
        }
    }

    async fn send_single_protobuf_message<T: RithmicMessage>(stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, message: &T) -> Result<(), RithmicApiError> {
        let mut buf = Vec::new();

        match message.encode(&mut buf) {
            Ok(_) => {}
            Err(e) => return Err(RithmicApiError::ServerErrorDebug(format!("Failed to encode RithmicMessage: {}", e)))
        }

        let length = buf.len() as u32;
        let mut prefixed_msg = length.to_be_bytes().to_vec();
        prefixed_msg.extend(buf);

        match stream.send(Message::Binary(prefixed_msg)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RithmicApiError::ServerErrorDebug(format!("Failed to send RithmicMessage: {}", e)))
        }
    }

    async fn read_single_protobuf_message<T: RithmicMessage + Default>(
        stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>
    ) -> Result<T, RithmicApiError> {
        while let Some(msg) = stream.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(e) => return Err(RithmicApiError::ServerErrorDebug(format!("Failed to read RithmicMessage: {}", e)))
            };
            if let Message::Binary(data) = msg {
                //println!("Received binary data: {:?}", data);

                // Create a cursor for reading the data
                let mut cursor = Cursor::new(data);

                // Read the 4-byte length header
                let mut length_buf = [0u8; 4];
                tokio::io::AsyncReadExt::read_exact(&mut cursor, &mut length_buf).map_err(|e| Box::new(e) as Box<dyn Error>).await;
                let length = u32::from_be_bytes(length_buf) as usize;

                // Read the Protobuf message
                let mut message_buf = vec![0u8; length];
                tokio::io::AsyncReadExt::read_exact(&mut cursor, &mut message_buf).map_err(|e| Box::new(e) as Box<dyn Error>).await;

                // Decode the Protobuf message
                return match T::decode(&message_buf[..]).map_err(|e| Box::new(e) as Box<dyn Error>) {
                    Ok(decoded_msg) => Ok(decoded_msg),
                    Err(e) => Err(RithmicApiError::ServerErrorDebug(format!("Failed to decode RithmicMessage: {}", e)))
                }
            }
        }
        Err(RithmicApiError::ServerErrorDebug("No valid message received".to_string()))
    }

    pub async fn connect_and_login(&self, plant: SysInfraType) -> Result<(), RithmicApiError> {
        if plant as i32 > 5 {
            return Err(RithmicApiError::ClientErrorDebug("Incorrect value for rithmic SysInfraType".to_string()))
        }
        // establish TCP connection to get the server details
        let (mut stream, response) = match connect_async(self.credentials.base_url.clone()).await {
            Ok((stream, response)) => (stream, response),
            Err(e) => return Err(RithmicApiError::ServerErrorDebug(format!("Failed to connect to rithmic: {}", e)))
        };

        println!("Rithmic connection: {:?}", response);
        // Rithmic System Info Request 16 From Client
        let request = RequestRithmicSystemInfo {
            template_id: 16,
            user_msg: vec!["Rust Fund Forge Signing In".to_string()],
        };

        RithmicApiClient::send_single_protobuf_message(&mut stream, &request).await?;
        // Rithmic System Info Response 17
        // Step 2: Read the full message based on the length
        let message: ResponseRithmicSystemInfo = RithmicApiClient::read_single_protobuf_message(&mut stream).await?;

        // Now we have the system name we can do the handshake
        let rithmic_server_name = match message.system_name.first() {
            Some(name) => name.clone(),
            None => {
                return Err(RithmicApiError::ServerErrorDebug(
                    "No system name found in response".to_string(),
                ));
            }
        };
        println!("{}", rithmic_server_name);
        stream.close(None).await.map_err(|e| Box::new(e) as Box<dyn Error>).unwrap();

        let (mut stream, _) = match connect_async(self.credentials.base_url.clone()).await {
            Ok((stream, response)) => (stream, response),
            Err(e) => return Err(RithmicApiError::ServerErrorDebug(format!("Failed to connect to rithmic, for login message: {}", e)))
        };

        // After handshake, we can send confidential data
        // Login Request 10 From Client
        let login_request = RequestLogin {
            template_id: 10,
            template_version: Some(self.credentials.template_version.clone()),
            user_msg: vec![],
            user: Some(self.credentials.user.clone()),
            password: Some(self.credentials.password.clone()),
            app_name: Some(self.credentials.app_name.clone()),
            app_version: Some(self.credentials.app_version.clone()),
            system_name: Some(rithmic_server_name),
            infra_type: Some(plant as i32),
            mac_addr: vec![],
            os_version: None,
            os_platform: None,
            aggregated_quotes: Some(self.credentials.aggregated_quotes.clone()),
        };
        RithmicApiClient::send_single_protobuf_message(&mut stream, &login_request).await?;

        // Login Response 11 From Server
        let message: ResponseLogin = RithmicApiClient::read_single_protobuf_message(&mut stream).await?;
        println!("{:?}", message);

        let (ws_writer, ws_reader) = stream.split();
        self.connected_plant.write().await.push(plant.clone());
        self.plant_writer.insert(plant.clone(), Arc::new(Mutex::new(ws_writer)));
        self.plant_reader.insert(plant.clone(), Arc::new(Mutex::new(ws_reader)));
        Ok(())
    }

    async fn send_message_split_streams<T: RithmicMessage>(api_instance: &Self, plant: &SysInfraType, message: &T) -> Result<(), RithmicApiError> {
        let mut buf = Vec::new();

        match message.encode(&mut buf) {
            Ok(_) => {}
            Err(e) => return Err(RithmicApiError::ServerErrorDebug(format!("Failed to encode RithmicMessage: {}", e)))
        }

        let length = buf.len() as u32;
        let mut prefixed_msg = length.to_be_bytes().to_vec();
        prefixed_msg.extend(buf);

        let writer = match api_instance.plant_writer.get(plant) {
            None => return Err(RithmicApiError::ClientErrorDebug(format!("You have not ran connect_and_login for this plant: {:?}", plant))),
            Some(writer) => writer
        };

        let mut writer = writer.value().lock().await;
        match writer.send(Message::Binary(prefixed_msg)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RithmicApiError::ServerErrorDebug(format!("Failed to send RithmicMessage, possible disconnect, try reconnecting to plant {:?}: {}", plant, e)))
        }
    }

    pub async fn shutdown_split_websocket(
        api_instance: &Self,
        plant: SysInfraType
    ) -> Result<(), RithmicApiError> {
        //Logout Request 12
        let logout_request = RequestLogout {
            template_id: 12,
            user_msg: vec!["Rust Fund Forge Signing Out".to_string()],
        };
        RithmicApiClient::send_message_split_streams(&api_instance, &plant, &logout_request).await?;

        let  (_, mut ws_writer) = match api_instance.plant_writer.remove(&plant) {
            None => return Err(RithmicApiError::ServerErrorDebug(format!("No writer found for rithmic plant: {:?}", plant))),
            Some(mut writer) => writer
        };

        let  (_, mut ws_reader) = match api_instance.plant_reader.remove(&plant) {
            None => return Err(RithmicApiError::ServerErrorDebug(format!("No writer found for rithmic plant: {:?}", plant))),
            Some(reader) => reader
        };

        // Send a close frame using the writer
        let mut ws_writer= ws_writer.lock().await;
        ws_writer.send(Message::Close(None)).await.unwrap();

        // Drain the reader to ensure the connection closes properly
        let mut ws_reader = ws_reader.lock().await;
        while let Some(msg) = ws_reader.next().await {
            match msg {
                Ok(Message::Close(_)) => break, // Close confirmed by the server
                Ok(_) => continue,              // Ignore other messages
                Err(e) => return  Err(RithmicApiError::ServerErrorDebug(format!("Failed safely shutdown stream: {}", e)))
            }
        }
        api_instance.connected_plant.write().await.retain(|x| *x != plant);
        println!("Safely shutdown rithmic split stream");
        Ok(())
    }

    /// Log out and shutdown all plant connections for the API instance
    pub async fn shutdown_all(api_instance: &Self) -> Result<(), RithmicApiError> {
        println!("Logging out and shutting down all connections");
        let connected_plant = api_instance.connected_plant.read().await.clone();
        let mut results = vec![];
        for plant in connected_plant {
            results.push(RithmicApiClient::shutdown_split_websocket(api_instance, plant.clone()).await);
        }
        for result in results {
            match result {
                Ok(_) => println!("Shutdown Success"),
                Err(e) => return Err(RithmicApiError::ServerErrorDebug(format!("Failed to properly shutdown a rithmic plant connection: {}", e)))
            }
        }
        Ok(())
    }

    async fn manage_responses (
        mut ws_writer: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    ) -> Result<(), RithmicApiError> {
        /*   tokio::task::spawn(async move {
                   while let Some(messages) = ws_reader.
           });*/
        Ok(())
    }

    /// Use this when we don't have any active subscriptions to persist the connection
    pub async fn idle_heart_beat(&self) {
        // Heartbeats
        /* Heartbeats responses from the server are a way of monitoring the communication link between client and server.
        Upon making a successful login to the Rithmic Infrastructure, clients are expected to send at least a heartbeat request
        (if no other requests are sent) to the server in order to keep the connection active. The heartbeat interval is specified in the login response.
         If clients don’t subscribe to any updates, nor send any queries, including heartbeats, then over a threshold amount of time the server will terminate
         such connections for not keeping the link active.
         Heartbeat requests from clients are not required when the client application is already receiving updates or responses from the server within the threshold period.*/
    }
}
