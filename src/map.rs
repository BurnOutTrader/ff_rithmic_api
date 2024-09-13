use std::io::Cursor;
use prost::encoding::{decode_key, decode_varint, WireType};


pub fn extract_template_id(bytes: &[u8]) -> Option<i32> {
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