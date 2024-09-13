use std::any::Any;
use std::collections::HashMap;
use std::io::Cursor;
use prost::Message;
use crate::errors::RithmicApiError;
use crate::rithmic_proto_objects::rti::ResponseHeartbeat;

pub async fn decode_response_heartbeat(data: Vec<u8>) -> Result<Box<ResponseHeartbeat>, RithmicApiError> {
    //messages will be forwarded here
    let mut cursor = Cursor::new(data);
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

    // Create a cursor to wrap the remaining data in the buffer.
    let mut cursor = Cursor::new(&message_buf);
    match ResponseHeartbeat::decode(&mut cursor) {
        Ok(decoded_msg) => {
            Ok(Box::new(decoded_msg))
        }
        Err(e) => {
            eprintln!("Failed to decode message: {}", e);
            Err(RithmicApiError::ServerErrorDebug("Unable to convert data".to_string()))
        }
    }
}

pub fn create_template_decoder_map() -> HashMap<i32, fn(Vec<u8>) -> Box<dyn Any>> {
    let mut map: HashMap<i32, fn(Vec<u8>) -> Box<dyn Any>> = HashMap::new();

    // Templates Shared across Infrastructure Plants
   /* map.insert(10, |data| Box::new(decode_login_request(data)) as Box<dyn std::any::Any>);
    map.insert(11, |data| Box::new(decode_login_response(data)) as Box<dyn std::any::Any>);
    map.insert(12, |data| Box::new(decode_logout_request(data)) as Box<dyn std::any::Any>);
    map.insert(13, |data| Box::new(decode_logout_response(data)) as Box<dyn std::any::Any>);
    map.insert(14, |data| Box::new(decode_reference_data_request(data)) as Box<dyn std::any::Any>);
    map.insert(15, |data| Box::new(decode_reference_data_response(data)) as Box<dyn std::any::Any>);
    map.insert(16, |data| Box::new(decode_rithmic_system_info_request(data)) as Box<dyn std::any::Any>);
    map.insert(17, |data| Box::new(decode_rithmic_system_info_response(data)) as Box<dyn std::any::Any>);
    map.insert(18, |data| Box::new(decode_request_heartbeat(data)) as Box<dyn std::any::Any>);*/
    map.insert(19, |data| Box::new(decode_response_heartbeat(data)) as Box<dyn std::any::Any>);

    /*
    map.insert(20, |data| Box::new(decode_rithmic_system_gateway_info_request(data)) as Box<dyn std::any::Any>);
    map.insert(21, |data| Box::new(decode_rithmic_system_gateway_info_response(data)) as Box<dyn std::any::Any>);
    map.insert(75, |data| Box::new(decode_reject(data)) as Box<dyn std::any::Any>);
    map.insert(76, |data| Box::new(decode_user_account_update(data)) as Box<dyn std::any::Any>);
    map.insert(77, |data| Box::new(decode_forced_logout(data)) as Box<dyn std::any::Any>);*/
/*
    // Templates Specific to Market Data Infrastructure
    map.insert(100, |data| Box::new(decode_market_data_update_request(data)) as Box<dyn std::any::Any>);
    map.insert(101, |data| Box::new(decode_market_data_update_response(data)) as Box<dyn std::any::Any>);
    map.insert(102, |data| Box::new(decode_get_instrument_by_underlying_request(data)) as Box<dyn std::any::Any>);
    map.insert(103, |data| Box::new(decode_get_instrument_by_underlying_response(data)) as Box<dyn std::any::Any>);
    map.insert(104, |data| Box::new(decode_get_instrument_by_underlying_keys_response(data)) as Box<dyn std::any::Any>);
    map.insert(105, |data| Box::new(decode_market_data_update_by_underlying_request(data)) as Box<dyn std::any::Any>);
    map.insert(106, |data| Box::new(decode_market_data_update_by_underlying_response(data)) as Box<dyn std::any::Any>);
    map.insert(107, |data| Box::new(decode_give_tick_size_type_table_request(data)) as Box<dyn std::any::Any>);
    map.insert(108, |data| Box::new(decode_give_tick_size_type_table_response(data)) as Box<dyn std::any::Any>);
    map.insert(109, |data| Box::new(decode_search_symbols_request(data)) as Box<dyn std::any::Any>);
    map.insert(110, |data| Box::new(decode_search_symbols_response(data)) as Box<dyn std::any::Any>);
    map.insert(111, |data| Box::new(decode_product_codes_request(data)) as Box<dyn std::any::Any>);
    map.insert(112, |data| Box::new(decode_product_codes_response(data)) as Box<dyn std::any::Any>);
    map.insert(113, |data| Box::new(decode_front_month_contract_request(data)) as Box<dyn std::any::Any>);
    map.insert(114, |data| Box::new(decode_front_month_contract_response(data)) as Box<dyn std::any::Any>);
    map.insert(115, |data| Box::new(decode_depth_by_order_snapshot_request(data)) as Box<dyn std::any::Any>);
    map.insert(116, |data| Box::new(decode_depth_by_order_snapshot_response(data)) as Box<dyn std::any::Any>);
    map.insert(117, |data| Box::new(decode_depth_by_order_updates_request(data)) as Box<dyn std::any::Any>);
    map.insert(118, |data| Box::new(decode_depth_by_order_updates_response(data)) as Box<dyn std::any::Any>);
    map.insert(119, |data| Box::new(decode_get_volume_at_price_request(data)) as Box<dyn std::any::Any>);
    map.insert(120, |data| Box::new(decode_get_volume_at_price_response(data)) as Box<dyn std::any::Any>);
    map.insert(121, |data| Box::new(decode_auxilliary_reference_data_request(data)) as Box<dyn std::any::Any>);
    map.insert(122, |data| Box::new(decode_auxilliary_reference_data_response(data)) as Box<dyn std::any::Any>);
    map.insert(150, |data| Box::new(decode_last_trade(data)) as Box<dyn std::any::Any>);
    map.insert(151, |data| Box::new(decode_best_bid_offer(data)) as Box<dyn std::any::Any>);
    map.insert(152, |data| Box::new(decode_trade_statistics(data)) as Box<dyn std::any::Any>);
    map.insert(153, |data| Box::new(decode_quote_statistics(data)) as Box<dyn std::any::Any>);
    map.insert(154, |data| Box::new(decode_indicator_prices(data)) as Box<dyn std::any::Any>);
    map.insert(155, |data| Box::new(decode_end_of_day_prices(data)) as Box<dyn std::any::Any>);
    map.insert(156, |data| Box::new(decode_order_book(data)) as Box<dyn std::any::Any>);
    map.insert(157, |data| Box::new(decode_market_mode(data)) as Box<dyn std::any::Any>);
    map.insert(158, |data| Box::new(decode_open_interest(data)) as Box<dyn std::any::Any>);
    map.insert(159, |data| Box::new(decode_front_month_contract_update(data)) as Box<dyn std::any::Any>);
    map.insert(160, |data| Box::new(decode_depth_by_order(data)) as Box<dyn std::any::Any>);
    map.insert(161, |data| Box::new(decode_depth_by_order_end_event(data)) as Box<dyn std::any::Any>);
    map.insert(162, |data| Box::new(decode_symbol_margin_rate(data)) as Box<dyn std::any::Any>);
    map.insert(163, |data| Box::new(decode_order_price_limits(data)) as Box<dyn std::any::Any>);*/
/*
    // Templates Specific to Order Plant Infrastructure
    map.insert(300, |data| Box::new(decode_login_info_request(data)) as Box<dyn std::any::Any>);
    map.insert(301, |data| Box::new(decode_login_info_response(data)) as Box<dyn std::any::Any>);
    map.insert(302, |data| Box::new(decode_account_list_request(data)) as Box<dyn std::any::Any>);
    map.insert(303, |data| Box::new(decode_account_list_response(data)) as Box<dyn std::any::Any>);
    map.insert(304, |data| Box::new(decode_account_rms_info_request(data)) as Box<dyn std::any::Any>);
    map.insert(305, |data| Box::new(decode_account_rms_info_response(data)) as Box<dyn std::any::Any>);
    map.insert(306, |data| Box::new(decode_product_rms_info_request(data)) as Box<dyn std::any::Any>);
    map.insert(307, |data| Box::new(decode_product_rms_info_response(data)) as Box<dyn std::any::Any>);
    map.insert(308, |data| Box::new(decode_subscribe_for_order_updates_request(data)) as Box<dyn std::any::Any>);
    map.insert(309, |data| Box::new(decode_subscribe_for_order_updates_response(data)) as Box<dyn std::any::Any>);
    map.insert(310, |data| Box::new(decode_trade_routes_request(data)) as Box<dyn std::any::Any>);
    map.insert(311, |data| Box::new(decode_trade_routes_response(data)) as Box<dyn std::any::Any>);
    map.insert(312, |data| Box::new(decode_new_order_request(data)) as Box<dyn std::any::Any>);
    map.insert(313, |data| Box::new(decode_new_order_response(data)) as Box<dyn std::any::Any>);
    map.insert(314, |data| Box::new(decode_modify_order_request(data)) as Box<dyn std::any::Any>);
    map.insert(315, |data| Box::new(decode_modify_order_response(data)) as Box<dyn std::any::Any>);
    map.insert(316, |data| Box::new(decode_cancel_order_request(data)) as Box<dyn std::any::Any>);
    map.insert(317, |data| Box::new(decode_cancel_order_response(data)) as Box<dyn std::any::Any>);
    map.insert(318, |data| Box::new(decode_show_order_history_dates_request(data)) as Box<dyn std::any::Any>);
    map.insert(319, |data| Box::new(decode_show_order_history_dates_response(data)) as Box<dyn std::any::Any>);
    map.insert(320, |data| Box::new(decode_show_orders_request(data)) as Box<dyn std::any::Any>);
    map.insert(321, |data| Box::new(decode_show_orders_response(data)) as Box<dyn std::any::Any>);
    map.insert(322, |data| Box::new(decode_show_order_history_request(data)) as Box<dyn std::any::Any>);
    map.insert(323, |data| Box::new(decode_show_order_history_response(data)) as Box<dyn std::any::Any>);
    map.insert(324, |data| Box::new(decode_show_order_history_summary_request(data)) as Box<dyn std::any::Any>);
    map.insert(325, |data| Box::new(decode_show_order_history_summary_response(data)) as Box<dyn std::any::Any>);
    map.insert(326, |data| Box::new(decode_show_order_history_detail_request(data)) as Box<dyn std::any::Any>);
    map.insert(327, |data| Box::new(decode_show_order_history_detail_response(data)) as Box<dyn std::any::Any>);
    map.insert(328, |data| Box::new(decode_oco_order_request(data)) as Box<dyn std::any::Any>);
    map.insert(329, |data| Box::new(decode_oco_order_response(data)) as Box<dyn std::any::Any>);
    map.insert(330, |data| Box::new(decode_bracket_order_request(data)) as Box<dyn std::any::Any>);
    map.insert(331, |data| Box::new(decode_bracket_order_response(data)) as Box<dyn std::any::Any>);
    map.insert(332, |data| Box::new(decode_update_target_bracket_level_request(data)) as Box<dyn std::any::Any>);
    map.insert(333, |data| Box::new(decode_update_target_bracket_level_response(data)) as Box<dyn std::any::Any>);
    map.insert(334, |data| Box::new(decode_update_stop_bracket_level_request(data)) as Box<dyn std::any::Any>);
    map.insert(335, |data| Box::new(decode_update_stop_bracket_level_response(data)) as Box<dyn std::any::Any>);
    map.insert(336, |data| Box::new(decode_subscribe_to_bracket_updates_request(data)) as Box<dyn std::any::Any>);
    map.insert(337, |data| Box::new(decode_subscribe_to_bracket_updates_response(data)) as Box<dyn std::any::Any>);
    map.insert(338, |data| Box::new(decode_show_brackets_request(data)) as Box<dyn std::any::Any>);
    map.insert(339, |data| Box::new(decode_show_brackets_response(data)) as Box<dyn std::any::Any>);
    map.insert(340, |data| Box::new(decode_show_bracket_stops_request(data)) as Box<dyn std::any::Any>);
    map.insert(341, |data| Box::new(decode_show_bracket_stops_response(data)) as Box<dyn std::any::Any>);
    map.insert(342, |data| Box::new(decode_list_exchange_permissions_request(data)) as Box<dyn std::any::Any>);
    map.insert(343, |data| Box::new(decode_list_exchange_permissions_response(data)) as Box<dyn std::any::Any>);
    map.insert(344, |data| Box::new(decode_link_orders_request(data)) as Box<dyn std::any::Any>);
    map.insert(345, |data| Box::new(decode_link_orders_response(data)) as Box<dyn std::any::Any>);
    map.insert(346, |data| Box::new(decode_cancel_all_orders_request(data)) as Box<dyn std::any::Any>);

    map.insert(346, Arc::new(|data| Box::new(decode_cancel_all_orders_request(data))as Box<dyn std::any::Any>);
    map.insert(347, Arc::new(|data| Box::new(decode_cancel_all_orders_response(data)) as Box<dyn Any + Send>));
    map.insert(348, Arc::new(|data| Box::new(decode_easy_to_borrow_list_request(data)) as Box<dyn Any + Send>));
    map.insert(349, Arc::new(|data| Box::new(decode_easy_to_borrow_list_response(data)) as Box<dyn Any + Send>));
    map.insert(3500, Arc::new(|data| Box::new(decode_modify_order_reference_data_request(data)) as Box<dyn Any + Send>));
    map.insert(3501, Arc::new(|data| Box::new(decode_modify_order_reference_data_response(data)) as Box<dyn Any + Send>));
    map.insert(3502, Arc::new(|data| Box::new(decode_order_session_config_request(data)) as Box<dyn Any + Send>));
    map.insert(3503, Arc::new(|data| Box::new(decode_order_session_config_response(data)) as Box<dyn Any + Send>));
    map.insert(3504, Arc::new(|data| Box::new(decode_exit_position_request(data)) as Box<dyn Any + Send>));
    map.insert(3505, Arc::new(|data| Box::new(decode_exit_position_response(data)) as Box<dyn Any + Send>));
    map.insert(3506, Arc::new(|data| Box::new(decode_replay_executions_request(data)) as Box<dyn Any + Send>));
    map.insert(3507, Arc::new(|data| Box::new(decode_replay_executions_response(data)) as Box<dyn Any + Send>));
    map.insert(3508, Arc::new(|data| Box::new(decode_account_rms_updates_request(data)) as Box<dyn Any + Send>));
    map.insert(3509, Arc::new(|data| Box::new(decode_account_rms_updates_response(data)) as Box<dyn Any + Send>));
    map.insert(350, Arc::new(|data| Box::new(decode_trade_route(data)) as Box<dyn Any + Send>));
    map.insert(351, Arc::new(|data| Box::new(decode_rithmic_order_notification(data)) as Box<dyn Any + Send>));
    map.insert(352, Arc::new(|data| Box::new(decode_exchange_order_notification(data)) as Box<dyn Any + Send>));
    map.insert(353, Arc::new(|data| Box::new(decode_bracket_updates(data)) as Box<dyn Any + Send>));
    map.insert(354, Arc::new(|data| Box::new(decode_account_list_updates(data)) as Box<dyn Any + Send>));
    map.insert(355, Arc::new(|data| Box::new(decode_update_easy_to_borrow_list(data)) as Box<dyn Any + Send>));
    map.insert(356, Arc::new(|data| Box::new(decode_account_rms_updates(data)) as Box<dyn Any + Send>));

    // Templates specific to History Plant Infrastructure
    map.insert(200, Arc::new(|data| Box::new(decode_time_bar_update_request(data)) as Box<dyn Any + Send>));
    map.insert(201, Arc::new(|data| Box::new(decode_time_bar_update_response(data)) as Box<dyn Any + Send>));
    map.insert(202, Arc::new(|data| Box::new(decode_time_bar_replay_request(data)) as Box<dyn Any + Send>));
    map.insert(203, Arc::new(|data| Box::new(decode_time_bar_replay_response(data)) as Box<dyn Any + Send>));
    map.insert(204, Arc::new(|data| Box::new(decode_tick_bar_update_request(data)) as Box<dyn Any + Send>));
    map.insert(205, Arc::new(|data| Box::new(decode_tick_bar_update_response(data)) as Box<dyn Any + Send>));
    map.insert(206, Arc::new(|data| Box::new(decode_tick_bar_replay_request(data)) as Box<dyn Any + Send>));
    map.insert(207, Arc::new(|data| Box::new(decode_tick_bar_replay_response(data)) as Box<dyn Any + Send>));
    map.insert(208, Arc::new(|data| Box::new(decode_volume_profile_minute_bars_request(data)) as Box<dyn Any + Send>));
    map.insert(209, Arc::new(|data| Box::new(decode_volume_profile_minute_bars_response(data)) as Box<dyn Any + Send>));
    map.insert(210, Arc::new(|data| Box::new(decode_resume_bars_request(data)) as Box<dyn Any + Send>));
    map.insert(211, Arc::new(|data| Box::new(decode_resume_bars_response(data)) as Box<dyn Any + Send>));
    map.insert(250, Arc::new(|data| Box::new(decode_time_bar(data)) as Box<dyn Any + Send>));
    map.insert(251, Arc::new(|data| Box::new(decode_tick_bar(data)) as Box<dyn Any + Send>));

    // Templates specific to PnL Plant
    map.insert(400, Arc::new(|data| Box::new(decode_pnl_position_updates_request(data)) as Box<dyn Any + Send>));
    map.insert(401, Arc::new(|data| Box::new(decode_pnl_position_updates_response(data)) as Box<dyn Any + Send>));
    map.insert(402, Arc::new(|data| Box::new(decode_pnl_position_snapshot_request(data)) as Box<dyn Any + Send>));
    map.insert(403, Arc::new(|data| Box::new(decode_pnl_position_snapshot_response(data)) as Box<dyn Any + Send>));
    map.insert(450, Arc::new(|data| Box::new(decode_instrument_pnl_position_update(data)) as Box<dyn Any + Send>));
    map.insert(451, Arc::new(|data| Box::new(decode_account_pnl_position_update(data)) as Box<dyn Any + Send>));

    // Templates specific to Repository Plant
    map.insert(500, Arc::new(|data| Box::new(decode_list_unaccepted_agreements_request(data)) as Box<dyn Any + Send>));
    map.insert(501, Arc::new(|data| Box::new(decode_list_unaccepted_agreements_response(data)) as Box<dyn Any + Send>));
    map.insert(502, Arc::new(|data| Box::new(decode_list_accepted_agreements_request(data)) as Box<dyn Any + Send>));
    map.insert(503, Arc::new(|data| Box::new(decode_list_accepted_agreements_response(data)) as Box<dyn Any + Send>));
    map.insert(504, Arc::new(|data| Box::new(decode_accept_agreement_request(data)) as Box<dyn Any + Send>));
    map.insert(505, Arc::new(|data| Box::new(decode_accept_agreement_response(data)) as Box<dyn Any + Send>));
    map.insert(506, Arc::new(|data| Box::new(decode_show_agreement_request(data)) as Box<dyn Any + Send>));
    map.insert(507, Arc::new(|data| Box::new(decode_show_agreement_response(data)) as Box<dyn Any + Send>));
    map.insert(508, Arc::new(|data| Box::new(decode_set_rithmic_marketdata_self_certification_status_request(data)) as Box<dyn Any + Send>));
    map.insert(509, Arc::new(|data| Box::new(decode_set_rithmic_marketdata_self_certification_status_response(data)) as Box<dyn Any + Send>));*/

    map
}