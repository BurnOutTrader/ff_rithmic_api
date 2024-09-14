use std::io::Cursor;
use std::sync::Arc;
use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use crate::api_client::RithmicApiClient;
use crate::credentials::RithmicCredentials;
use crate::rithmic_proto_objects::rti::request_login::SysInfraType;
use crate::rithmic_proto_objects::rti::{BestBidOffer, DepthByOrder, DepthByOrderEndEvent, EndOfDayPrices, FrontMonthContractUpdate, IndicatorPrices, LastTrade, MarketMode, OpenInterest, OrderBook, OrderPriceLimits, QuoteStatistics, RequestHeartbeat, RequestRithmicSystemGatewayInfo, ResponseAuxilliaryReferenceData, ResponseDepthByOrderSnapshot, ResponseDepthByOrderUpdates, ResponseFrontMonthContract, ResponseGetInstrumentByUnderlying, ResponseGetInstrumentByUnderlyingKeys, ResponseGetVolumeAtPrice, ResponseGiveTickSizeTypeTable, ResponseHeartbeat, ResponseLogin, ResponseLogout, ResponseMarketDataUpdate, ResponseMarketDataUpdateByUnderlying, ResponseProductCodes, ResponseReferenceData, ResponseRithmicSystemInfo, ResponseSearchSymbols, SymbolMarginRate, TradeStatistics};
use crate::errors::RithmicApiError;
use prost::{Message as ProstMessage};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::Message;


/// This Test will fail when the market is closed.
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
    let rithmic_api_arc = Arc::new(rithmic_api);

    // Establish connections, sign in and receive back the websocket readers
    let ticker_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> = rithmic_api_arc.connect_and_login(SysInfraType::TickerPlant).await?;
    assert!(rithmic_api_arc.is_connected(SysInfraType::TickerPlant).await);

    let _history_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> = rithmic_api_arc.connect_and_login(SysInfraType::HistoryPlant).await?;
    assert!(rithmic_api_arc.is_connected(SysInfraType::HistoryPlant).await);

    let _order_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api_arc.connect_and_login(SysInfraType::OrderPlant).await?;
    assert!(rithmic_api_arc.is_connected(SysInfraType::OrderPlant).await);

    let _pnl_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api_arc.connect_and_login(SysInfraType::PnlPlant).await?;
    assert!(rithmic_api_arc.is_connected(SysInfraType::PnlPlant).await);

    let _repo_receiver:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> =rithmic_api_arc.connect_and_login(SysInfraType::RepositoryPlant).await?;
    assert!(rithmic_api_arc.is_connected(SysInfraType::RepositoryPlant).await);


    // send a heartbeat request as a test message, 'RequestHeartbeat' Template number 18
    let heart_beat = RequestHeartbeat {
        template_id: 18,
        user_msg: vec![format!("{} Testing heartbeat", app_name)],
        ssboe: None,
        usecs: None,
    };

    // We can send messages with only a reference to the client, so we can wrap our client in Arc or share it between threads and still utilise all associated functions.
    match rithmic_api_arc.send_message(&SysInfraType::TickerPlant, &heart_beat).await {
        Ok(_) => println!("Heart beat sent"),
        Err(e) => eprintln!("Heartbeat send failed: {}", e)
    }

    handle_received_responses(rithmic_api_arc.clone(), ticker_receiver, SysInfraType::TickerPlant).await?;
    let _ = rithmic_api_arc.send_message(&SysInfraType::TickerPlant, &heart_beat).await?;

    // Logout and Shutdown all connections
    rithmic_api_arc.shutdown_all().await?;

    // or Logout and Shutdown a single connection
    //RithmicApiClient::shutdown_split_websocket(&rithmic_api, SysInfraType::TickerPlant).await?;

    Ok(())
}

pub async fn handle_received_responses(
    client: Arc<RithmicApiClient>,
    reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    plant: SysInfraType
) -> Result<(), RithmicApiError> {
    match plant {
        SysInfraType::TickerPlant => handle_responses_from_ticker_plant(client, reader).await,
        SysInfraType::OrderPlant => panic!("Not yet implemented"),
        SysInfraType::HistoryPlant => panic!("Not yet implemented"),
        SysInfraType::PnlPlant => panic!("Not yet implemented"),
        SysInfraType::RepositoryPlant => panic!("Not yet implemented"),
    }
}
/// we use extract_template_id() to get the template id using the field_number 154467 without casting to any concrete type, then we map to the concrete type and handle that message.
pub async fn handle_responses_from_ticker_plant(
    client: Arc<RithmicApiClient>,
    mut reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Result<(), RithmicApiError> {
    tokio::task::spawn(async move {
        const PLANT: SysInfraType = SysInfraType::TickerPlant;
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

                                // spawn a new task so that we can handle next message faster.
                                let client = client.clone();
                                tokio::task::spawn(async move {
                                    match template_id {
                                        11 => {
                                            if let Ok(msg) = ResponseLogin::decode(&message_buf[..]) {
                                                // Login Response
                                                // From Server
                                                println!("Login Response (Template ID: 11) from Server: {:?}", msg);
                                            }
                                        },
                                        13 => {
                                            if let Ok(msg) = ResponseLogout::decode(&message_buf[..]) {
                                                // Logout Response
                                                // From Server
                                                println!("Logout Response (Template ID: 13) from Server: {:?}", msg);
                                            }
                                        },
                                        15 => {
                                            if let Ok(msg) = ResponseReferenceData::decode(&message_buf[..]) {
                                                // Reference Data Response
                                                // From Server
                                                println!("Reference Data Response (Template ID: 15) from Server: {:?}", msg);
                                            }
                                        },
                                        17 => {
                                            if let Ok(msg) = ResponseRithmicSystemInfo::decode(&message_buf[..]) {
                                                // Rithmic System Info Response
                                                // From Server
                                                println!("Rithmic System Info Response (Template ID: 17) from Server: {:?}", msg);
                                            }
                                        },
                                        19 => {
                                            if let Ok(msg) = ResponseHeartbeat::decode(&message_buf[..]) {
                                                // Response Heartbeat
                                                // From Server
                                                println!("Response Heartbeat (Template ID: 19) from Server: {:?}", msg);

                                                // Example of sending a system gateway info request afterward
                                                let request = RequestRithmicSystemGatewayInfo {
                                                    template_id: 20,
                                                    user_msg: vec![],
                                                    system_name: Some(client.get_system_name(&PLANT).await.unwrap()),
                                                };
                                                client.send_message(&PLANT, &request).await.unwrap();
                                            }
                                        },
                                        101 => {
                                            if let Ok(msg) = ResponseMarketDataUpdate::decode(&message_buf[..]) {
                                                // Market Data Update Response
                                                // From Server
                                                println!("Market Data Update Response (Template ID: 101) from Server: {:?}", msg);
                                            }
                                        },
                                        103 => {
                                            if let Ok(msg) = ResponseGetInstrumentByUnderlying::decode(&message_buf[..]) {
                                                // Get Instrument by Underlying Response
                                                // From Server
                                                println!("Get Instrument by Underlying Response (Template ID: 103) from Server: {:?}", msg);
                                            }
                                        },
                                        104 => {
                                            if let Ok(msg) = ResponseGetInstrumentByUnderlyingKeys::decode(&message_buf[..]) {
                                                // Get Instrument by Underlying Keys Response
                                                // From Server
                                                println!("Get Instrument by Underlying Keys Response (Template ID: 104) from Server: {:?}", msg);
                                            }
                                        },
                                        106 => {
                                            if let Ok(msg) = ResponseMarketDataUpdateByUnderlying::decode(&message_buf[..]) {
                                                // Market Data Update by Underlying Response
                                                // From Server
                                                println!("Market Data Update by Underlying Response (Template ID: 106) from Server: {:?}", msg);
                                            }
                                        },
                                        108 => {
                                            if let Ok(msg) = ResponseGiveTickSizeTypeTable::decode(&message_buf[..]) {
                                                // Give Tick Size Type Table Response
                                                // From Server
                                                println!("Give Tick Size Type Table Response (Template ID: 108) from Server: {:?}", msg);
                                            }
                                        },
                                        110 => {
                                            if let Ok(msg) = ResponseSearchSymbols::decode(&message_buf[..]) {
                                                // Search Symbols Response
                                                // From Server
                                                println!("Search Symbols Response (Template ID: 110) from Server: {:?}", msg);
                                            }
                                        },
                                        112 => {
                                            if let Ok(msg) = ResponseProductCodes::decode(&message_buf[..]) {
                                                // Product Codes Response
                                                // From Server
                                                println!("Product Codes Response (Template ID: 112) from Server: {:?}", msg);
                                            }
                                        },
                                        114 => {
                                            if let Ok(msg) = ResponseFrontMonthContract::decode(&message_buf[..]) {
                                                // Front Month Contract Response
                                                // From Server
                                                println!("Front Month Contract Response (Template ID: 114) from Server: {:?}", msg);
                                            }
                                        },
                                        116 => {
                                            if let Ok(msg) = ResponseDepthByOrderSnapshot::decode(&message_buf[..]) {
                                                // Depth By Order Snapshot Response
                                                // From Server
                                                println!("Depth By Order Snapshot Response (Template ID: 116) from Server: {:?}", msg);
                                            }
                                        },
                                        118 => {
                                            if let Ok(msg) = ResponseDepthByOrderUpdates::decode(&message_buf[..]) {
                                                // Depth By Order Updates Response
                                                // From Server
                                                println!("Depth By Order Updates Response (Template ID: 118) from Server: {:?}", msg);
                                            }
                                        },
                                        120 => {
                                            if let Ok(msg) = ResponseGetVolumeAtPrice::decode(&message_buf[..]) {
                                                // Get Volume At Price Response
                                                // From Server
                                                println!("Get Volume At Price Response (Template ID: 120) from Server: {:?}", msg);
                                            }
                                        },
                                        122 => {
                                            if let Ok(msg) = ResponseAuxilliaryReferenceData::decode(&message_buf[..]) {
                                                // Auxiliary Reference Data Response
                                                // From Server
                                                println!("Auxiliary Reference Data Response (Template ID: 122) from Server: {:?}", msg);
                                            }
                                        },
                                        150 => {
                                            if let Ok(msg) = LastTrade::decode(&message_buf[..]) {
                                                // Last Trade
                                                // From Server
                                                println!("Last Trade (Template ID: 150) from Server: {:?}", msg);
                                            }
                                        },
                                        151 => {
                                            if let Ok(msg) = BestBidOffer::decode(&message_buf[..]) {
                                                // Best Bid Offer
                                                // From Server
                                                println!("Best Bid Offer (Template ID: 151) from Server: {:?}", msg);
                                            }
                                        },
                                        152 => {
                                            if let Ok(msg) = TradeStatistics::decode(&message_buf[..]) {
                                                // Trade Statistics
                                                // From Server
                                                println!("Trade Statistics (Template ID: 152) from Server: {:?}", msg);
                                            }
                                        },
                                        153 => {
                                            if let Ok(msg) = QuoteStatistics::decode(&message_buf[..]) {
                                                // Quote Statistics
                                                // From Server
                                                println!("Quote Statistics (Template ID: 153) from Server: {:?}", msg);
                                            }
                                        },
                                        154 => {
                                            if let Ok(msg) = IndicatorPrices::decode(&message_buf[..]) {
                                                // Indicator Prices
                                                // From Server
                                                println!("Indicator Prices (Template ID: 154) from Server: {:?}", msg);
                                            }
                                        },
                                        155 => {
                                            if let Ok(msg) = EndOfDayPrices::decode(&message_buf[..]) {
                                                // End Of Day Prices
                                                // From Server
                                                println!("End Of Day Prices (Template ID: 155) from Server: {:?}", msg);
                                            }
                                        },
                                        156 => {
                                            if let Ok(msg) = OrderBook::decode(&message_buf[..]) {
                                                // Order Book
                                                // From Server
                                                println!("Order Book (Template ID: 156) from Server: {:?}", msg);
                                            }
                                        },
                                        157 => {
                                            if let Ok(msg) = MarketMode::decode(&message_buf[..]) {
                                                // Market Mode
                                                // From Server
                                                println!("Market Mode (Template ID: 157) from Server: {:?}", msg);
                                            }
                                        },
                                        158 => {
                                            if let Ok(msg) = OpenInterest::decode(&message_buf[..]) {
                                                // Open Interest
                                                // From Server
                                                println!("Open Interest (Template ID: 158) from Server: {:?}", msg);
                                            }
                                        },
                                        159 => {
                                            if let Ok(msg) = FrontMonthContractUpdate::decode(&message_buf[..]) {
                                                // Front Month Contract Update
                                                // From Server
                                                println!("Front Month Contract Update (Template ID: 159) from Server: {:?}", msg);
                                            }
                                        },
                                        160 => {
                                            if let Ok(msg) = DepthByOrder::decode(&message_buf[..]) {
                                                // Depth By Order
                                                // From Server
                                                println!("Depth By Order (Template ID: 160) from Server: {:?}", msg);
                                            }
                                        },
                                        161 => {
                                            if let Ok(msg) = DepthByOrderEndEvent::decode(&message_buf[..]) {
                                                // Depth By Order End Event
                                                // From Server
                                                println!("DepthByOrderEndEvent (Template ID: 161) from Server: {:?}", msg);
                                            }
                                        },
                                        162 => {
                                            if let Ok(msg) = SymbolMarginRate::decode(&message_buf[..]) {
                                                // Symbol Margin Rate
                                                // From Server
                                                println!("Symbol Margin Rate (Template ID: 162) from Server: {:?}", msg);
                                            }
                                        },
                                        163 => {
                                            if let Ok(msg) = OrderPriceLimits::decode(&message_buf[..]) {
                                                // Order Price Limits
                                                // From Server
                                                println!("Order Price Limits (Template ID: 163) from Server: {:?}", msg);
                                            }
                                        },
                                        _ => println!("Failed to extract template_id")
                                    }
                                });
                            }
                        }
                        Message::Ping(ping) => {
                            println!("{:?}", ping)
                        }
                        Message::Pong(pong) => {
                            println!("{:?}", pong)
                        }
                        Message::Close(close) => {
                            // receive this message when market is closed.
                            // received: Ok(Close(Some(CloseFrame { code: Normal, reason: "normal closure" })))
                            println!("{:?}", close)
                        }
                        Message::Frame(frame) => {
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
    });
    Ok(())
}

