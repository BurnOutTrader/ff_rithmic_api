use std::io::{Cursor};
use prost::{Message as ProstMessage};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc};
use std::time::{Duration};
use dashmap::DashMap;
use futures_util::stream::{SplitSink, SplitStream};
use tokio::sync::{Mutex, RwLock};
use crate::credentials::RithmicCredentials;
use crate::rithmic_proto_objects::rti::request_login::SysInfraType;
use crate::rithmic_proto_objects::rti::{RequestHeartbeat, RequestLogin, RequestLogout, RequestRithmicSystemInfo, ResponseLogin, ResponseRithmicSystemInfo};
use crate::errors::RithmicApiError;
use prost::encoding::{decode_key, decode_varint, WireType};
use tokio::task::JoinHandle;
use tokio::time::{sleep_until, Instant};

///Server uses Big Endian format for binary data
pub struct RithmicApiClient {
    /// Credentials used for this instance of the api. we can have multiple instances for different brokers.
    credentials: RithmicCredentials,

    /// A list of the SysInfraType which we have logged into
    connected_plant: Arc<RwLock<Vec<SysInfraType>>>,

    /// The writer half of the websocket for the SysInfraType
    plant_writer: DashMap<SysInfraType, Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>>,

    /// The heartbeat intervals before time out. this was specified on logging in
    heart_beat_intervals: DashMap<SysInfraType, Duration>,

    /// The time the last message was sent, this is used to determine if we need to send a heartbeat.
    /// If we have a race condition in live testing this property object may need to incorporate a lock
    last_message_time: Arc<DashMap<SysInfraType, Instant>>,

    /// The system name for the associated plant
    system_name: DashMap<SysInfraType, String>,

    /// Keep a map of heartbeat tasks so that we can cut the loop when we shut down a plant conenction
    heartbeats: DashMap<SysInfraType, JoinHandle<()>>,

    heartbeat_required: DashMap<SysInfraType, Arc<RwLock<bool>>>,
}

impl RithmicApiClient {
    pub fn new(
        credentials: RithmicCredentials
    ) -> Self {
        Self {
            credentials,
            connected_plant: Default::default(),
            plant_writer: DashMap::with_capacity(5),
            heart_beat_intervals: DashMap::with_capacity(5),
            last_message_time: Arc::new(DashMap::with_capacity(5)),
            system_name: DashMap::with_capacity(5),
            heartbeats: DashMap::with_capacity(5),
            heartbeat_required: DashMap::with_capacity(5),
        }
    }

    pub async fn get_system_name(&self, plant: &SysInfraType) -> Option<String> {
        match self.system_name.get(plant) {
            None => None,
            Some(name) => Some(name.clone())
        }
    }

    /// get the writer for the specified plant
    pub async fn get_writer(&self,  plant: &SysInfraType) -> Option<Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>> {
        match self.plant_writer.get(plant) {
            None => None,
            Some(writer) => Some(writer.clone())
        }
    }

    // This function does not check if the connection is mainatained, only that it was established initially.
    pub async fn is_connected(
        &self,
        plant: SysInfraType
    ) -> bool {
        self.connected_plant.read().await.contains(&plant)
    }

    /// This function does not safely disconnect from rithmic, it simply dumps the existing references to the stream.
    pub async fn register_disconnect(
        &self,
        plant: SysInfraType
    ) {
        self.connected_plant.write().await.retain(|x|*x != plant);
        self.plant_writer.remove(&plant);
    }

    /// only used to register and login before splitting the stream.
    async fn send_single_protobuf_message<T: ProstMessage>(
        stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, message: &T
    ) -> Result<(), RithmicApiError> {
        let mut buf = Vec::new();

        match message.encode(&mut buf) {
            Ok(_) => {}
            Err(e) => return Err(RithmicApiError::ServerErrorDebug(format!("Failed to encode RithmicMessage: {}", e)))
        }

        let length = buf.len() as u32;
        let mut prefixed_msg = length.to_be_bytes().to_vec();
        prefixed_msg.extend(buf);
        stream.send(Message::Binary(prefixed_msg)).await.map_err(|e| RithmicApiError::WebSocket(e))
    }

    /// Used to receive system and login response before splitting the stream.
    async fn read_single_protobuf_message<T: ProstMessage + Default>(
        stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>
    ) -> Result<T, RithmicApiError> {
        while let Some(msg) = stream.next().await { //todo change from while to if and test
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
                let _ = tokio::io::AsyncReadExt::read_exact(&mut cursor, &mut length_buf).await.map_err(RithmicApiError::Io);
                let length = u32::from_be_bytes(length_buf) as usize;

                // Read the Protobuf message
                let mut message_buf = vec![0u8; length];
                tokio::io::AsyncReadExt::read_exact(&mut cursor, &mut message_buf).await.map_err(RithmicApiError::Io)?;

                // Decode the Protobuf message
                return match T::decode(&message_buf[..]) {
                    Ok(decoded_msg) => Ok(decoded_msg),
                    Err(e) => Err(RithmicApiError::ProtobufDecode(e)), // Use the ProtobufDecode variant
                }
            }
        }
        Err(RithmicApiError::ServerErrorDebug("No valid message received".to_string()))
    }

    /// Connect to the desired plant and sign in with your credentials.
    pub async fn connect_and_login(
        &self,
        plant: SysInfraType,
    ) -> Result<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, RithmicApiError> {

        if plant as i32 > 5 {
            return Err(RithmicApiError::ClientErrorDebug("Incorrect value for rithmic SysInfraType".to_string()))
        }
        // establish TCP connection to get the server details
        let (mut stream, response) = match connect_async(self.credentials.base_url.clone()).await {
            Ok((stream, response)) => (stream, response),
            Err(e) => return Err(RithmicApiError::ServerErrorDebug(format!("Failed to connect to rithmic: {}", e)))
        };
        println!("Rithmic connection established: {:?}", response);
        // Rithmic System Info Request 16 From Client
        let request = RequestRithmicSystemInfo {
            template_id: 16,
            user_msg: vec![format!("{} Signing In", self.credentials.app_name)],
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
        //println!("{}", rithmic_server_name);
        stream.close(None).await.map_err(RithmicApiError::WebSocket)?;

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
            system_name: Some(rithmic_server_name.clone()),
            infra_type: Some(plant as i32),
            mac_addr: vec![],
            os_version: None,
            os_platform: None,
            aggregated_quotes: Some(self.credentials.aggregated_quotes.clone()),
        };
        RithmicApiClient::send_single_protobuf_message(&mut stream, &login_request).await?;

        // Login Response 11 From Server
        let response: ResponseLogin = RithmicApiClient::read_single_protobuf_message(&mut stream).await?;
        if let Some(heartbeat_interval) = response.heartbeat_interval {
            self.heart_beat_intervals.insert( plant.clone(), Duration::from_secs(heartbeat_interval as u64));
        }

        let (ws_writer, ws_reader) = stream.split();
        self.connected_plant.write().await.push(plant.clone());
        self.plant_writer.insert(plant.clone(), Arc::new(Mutex::new(ws_writer)));
        self.system_name.insert(plant.clone(), rithmic_server_name.clone());
        match self.start_heartbeat(plant).await {
            Ok(_) => Ok(ws_reader),
            Err(e) => {
                match self.shutdown_plant(plant).await {
                    Ok(_) => {}
                    Err(_) => {}
                }
                Err(RithmicApiError::ClientErrorDebug(format!("{}", e)))
            },
        }
    }

    /// Send a message on the write half of the plant stream.
    pub async fn send_message<T: ProstMessage>(
        &self,
        plant: &SysInfraType,
        message: &T
    ) -> Result<(), RithmicApiError> {
        let mut buf = Vec::new();

        match message.encode(&mut buf) {
            Ok(_) => {}
            Err(e) => return Err(RithmicApiError::ServerErrorDebug(format!("Failed to encode RithmicMessage: {}", e)))
        }

        let length = buf.len() as u32;
        let mut prefixed_msg = length.to_be_bytes().to_vec();
        prefixed_msg.extend(buf);

        let writer = match self.plant_writer.get(plant) {
            None => return Err(RithmicApiError::ClientErrorDebug(format!("You have not ran connect_and_login for this plant: {:?}", plant))),
            Some(writer) => writer
        };

        let mut writer = writer.value().lock().await;
        match writer.send(Message::Binary(prefixed_msg)).await {
            Ok(_) => {
                self.last_message_time.insert(plant.clone(), Instant::now());
                Ok(())
            },
            Err(e) => Err(RithmicApiError::ServerErrorDebug(format!("Failed to send RithmicMessage, possible disconnect, try reconnecting to plant {:?}: {}", plant, e)))
        }
    }

    /// Signs out of rithmic with the specific plant safely shuts down the web socket. removing references from our api object.
    pub async fn shutdown_plant(
        &self,
        plant: SysInfraType
    ) -> Result<(), RithmicApiError> {
        //Logout Request 12
        let logout_request = RequestLogout {
            template_id: 12,
            user_msg: vec![format!("{} Signing Out", self.credentials.app_name)],
        };
        self.send_message(&plant, &logout_request).await?;

        let  (_, ws_writer) = match self.plant_writer.remove(&plant) {
            None => return Err(RithmicApiError::ServerErrorDebug(format!("No writer found for rithmic plant: {:?}", plant))),
            Some(writer) => writer
        };

        // Send a close frame using the writer
        let mut ws_writer= ws_writer.lock().await;
        ws_writer.send(Message::Close(None)).await.map_err(RithmicApiError::WebSocket)?;

        self.connected_plant.write().await.retain(|x| *x != plant);
        self.system_name.remove(&plant);
        if let Some(heartbeat_task) = self.heartbeats.get(&plant) {
            heartbeat_task.value().abort();
        }
        println!("Safely shutdown rithmic split stream");
        Ok(())
    }

    /// Log out and shutdown all plant connections for the API instance
    pub async fn shutdown_all(
        &self
    ) -> Result<(), RithmicApiError> {
        println!("Logging out and shutting down all connections");
        let connected_plant = self.connected_plant.read().await.clone();
        let mut results = vec![];
        for plant in connected_plant {
            results.push(RithmicApiClient::shutdown_plant(self, plant.clone()).await);
        }
        for result in results {
            match result {
                Ok(_) => println!("Shutdown Success"),
                Err(e) => return Err(RithmicApiError::ServerErrorDebug(format!("Failed to properly shutdown a rithmic plant connection: {}", e)))
            }
        }
        Ok(())
    }

    /// This function updates the last message time DashMap to reset the heartbeat countdown.
    /// If we have a race condition in live testing this property object may need to incorporate a lock
    pub fn update_heartbeat(&self, plant: SysInfraType) {
        self.last_message_time.insert(plant, Instant::now());
    }

    /// Change the requirements for heart beat, if we are streaming data from the plant we can switch this to no to disable the heartbeat and stop the heartbeat task.
    /// if yes and no heartbeat task is present one will be started.
    /// if no and a heartbeat task is started it will be stopped.
    pub async fn switch_heartbeat_required(&self, plant: &SysInfraType, requirement: bool) -> Result<(), RithmicApiError> {
        match self.heartbeat_required.get(plant) {
            None => {
                self.heartbeat_required.insert(plant.clone(), Arc::new(RwLock::new(requirement)));
                if requirement == true {
                    return match self.start_heartbeat(plant.clone()).await {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e)
                    }
                }
                Ok(())
            }
            Some(required_lock) => {
                let original_requirement = required_lock.read().await.clone();
                if original_requirement == requirement {
                    return Ok(())
                }
                *required_lock.write().await = requirement;
                //require a heartbeat and don't have one, start it.
                if !self.heartbeats.contains_key(plant) && requirement == true {
                    return match self.start_heartbeat(plant.clone()).await {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e)
                    }
                }
                //if we no longer require a heartbeat and if we have one, abort it.
                else if requirement == false {
                    if let Some((_, heartbeat)) = self.heartbeats.remove(plant) {
                        heartbeat.abort();
                    }
                }
                Ok(())
            }
        }
    }

    pub async fn start_heartbeat(
        &self,
        plant: SysInfraType,
    ) -> Result<(), RithmicApiError> {
        // Interval for heartbeat checks
        let heartbeat_interval = match self.heart_beat_intervals.get(&plant) {
            None => return Err(RithmicApiError::ClientErrorDebug("No heartbeat interval recorded at log in, please logout and login again".to_string())),
            Some(hb) => hb
        }.clone();


        let last_message = self.last_message_time.clone();
        let writer = match self.plant_writer.get(&plant) {
            None => return Err(RithmicApiError::ClientErrorDebug("No writer stored at log in, please logout and login again".to_string())),
            Some(writer) => writer
        }.value().clone();

        let heart_beat_request = RequestHeartbeat {
            template_id: 18,
            user_msg: vec![],
            ssboe: None,
            usecs: None,
        };

        let mut buf = Vec::new();
        match heart_beat_request.encode(&mut buf) {
            Ok(_) => {}
            Err(e) => return Err(RithmicApiError::ClientErrorDebug(format!("Failed to encode RithmicMessage: {}", e)))
        }

        let length = buf.len() as u32;
        let mut prefixed_msg = length.to_be_bytes().to_vec();
        prefixed_msg.extend(buf);

        let last_message_time = self.last_message_time.clone();

        // Send an initial heartbeat request
        {
            let mut sender = writer.lock().await;
            match sender.send(Message::Binary(prefixed_msg.clone())).await {
                Ok(_) => {},
                Err(e) => eprintln!("Failed to send RithmicMessage, possible disconnect, try reconnecting to plant {:?}: {}", plant, e)
            }
            last_message_time.insert(plant.clone(), Instant::now());
        }

        // Spawn the heartbeat task and store the handle
        let task_handle = tokio::task::spawn({
            let plant = plant.clone();
            let last_message = last_message.clone();
            let writer = writer.clone();
            async move {
                let mut expiration_time: Instant = Instant::now() + heartbeat_interval - Duration::from_millis(500);
                loop {
                    sleep_until(expiration_time).await;
                    if let Some(last_msg_time) = last_message.get(&plant) {
                        expiration_time = *last_msg_time + heartbeat_interval - Duration::from_millis(500);
                        if Instant::now() < expiration_time {
                            continue
                        }
                        let mut sender = writer.lock().await;
                        match sender.send(Message::Binary(prefixed_msg.clone())).await {
                            Ok(_) => {},
                            Err(e) => {
                                eprintln!("Failed to send RithmicMessage, possible disconnect, try reconnecting to plant {:?}: {}", plant, e);
                                break;
                            }
                        }
                        last_message.insert(plant.clone(), Instant::now());
                    }
                }
            }
        });
        // Store the task handle in the DashMap
        self.heartbeats.insert(plant, task_handle);
        Ok(())
    }

    /// Dynamically get the template_id field from a generic type T.
    pub fn extract_template_id(&self, bytes: &[u8]) -> Option<i32> {
        let mut cursor = Cursor::new(bytes);
        while let Ok((field_number, wire_type)) = decode_key(&mut cursor) {
            if field_number == 154467 && wire_type == WireType::Varint {
                // We've found the template_id field
                return decode_varint(&mut cursor).ok().map(|v| v as i32);
            } else {
                // Skip this field
                match wire_type {
                    WireType::Varint => { let _ = decode_varint(&mut cursor); }
                    WireType::SixtyFourBit => { let _ = cursor.set_position(cursor.position() + 8); }
                    WireType::LengthDelimited => {
                        if let Ok(len) = decode_varint(&mut cursor) {
                            let _ = cursor.set_position(cursor.position() + len as u64);
                        } else {
                            return None; // Error decoding length
                        }
                    }
                    WireType::StartGroup | WireType::EndGroup => {} // These are deprecated and shouldn't appear
                    WireType::ThirtyTwoBit => { let _ = cursor.set_position(cursor.position() + 4); }
                }
            }
        }

        None // template_id field not found
    }
}
