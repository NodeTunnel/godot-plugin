use crate::protocol::error::ProtocolError;
use crate::protocol::packet::RoomInfo;

pub fn read_bool(bytes: &[u8]) -> Result<(bool, &[u8]), ProtocolError> {
    let (value, rest) = read_i32(bytes)?;
    Ok((value != 0, rest))
}

pub fn read_i32(bytes: &[u8]) -> Result<(i32, &[u8]), ProtocolError> {
    if bytes.len() < 4 {
        return Err(ProtocolError::NotEnoughBytes(
            format!("for i32 (need {} bytes, have {})", 4, bytes.len())
        ));
    }
    let value = i32::from_be_bytes(bytes[..4].try_into()?);
    Ok((value, &bytes[4..]))
}

pub fn read_string(bytes: &[u8]) -> Result<(String, &[u8]), ProtocolError> {
    let (len, rest) = read_i32(bytes)?;

    if rest.len() < len as usize {
        return Err(ProtocolError::NotEnoughBytes(
            format!("for string (need {} bytes, have {})", len, rest.len())
        ));
    }

    let string_bytes = &rest[..len as usize];
    let remaining = &rest[len as usize..];

    Ok((String::from_utf8(string_bytes.to_vec())?, remaining))
}

pub fn push_string(buf: &mut Vec<u8>, value: &str) {
    let bytes = value.as_bytes();
    buf.extend((bytes.len() as i32).to_be_bytes());
    buf.extend(bytes);
}

pub fn push_bool(buf: &mut Vec<u8>, value: bool) {
    push_i32(buf, if value { 1 } else { 0 });
}

pub fn push_i32(buf: &mut Vec<u8>, value: i32) {
    buf.extend(value.to_be_bytes());
}

pub fn read_room_info(bytes: &[u8]) -> Result<(RoomInfo, &[u8]), ProtocolError> {
    let (id, r) = read_string(bytes)?;
    let (metadata, r) = read_string(r)?;

    Ok((RoomInfo { id, metadata }, r))
}

pub fn read_vec_room_info(bytes: &[u8]) -> Result<(Vec<RoomInfo>, &[u8]), ProtocolError> {
    let (len, mut rest) = read_i32(bytes)?;

    if len < 0 {
        return Err(ProtocolError::NegativeVectorLength());
    }

    let mut rooms = Vec::with_capacity(len as usize);
    for _ in 0..len {
        let (room, remaining) = read_room_info(rest)?;
        rooms.push(room);
        rest = remaining;
    }

    Ok((rooms, rest))
}

pub fn push_vec_room_info(buf: &mut Vec<u8>, rooms: &[RoomInfo]) {
    push_i32(buf, rooms.len() as i32);
    for room in rooms {
        push_string(buf, &room.id);
        push_string(buf, &room.metadata);
    }
}
