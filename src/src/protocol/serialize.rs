use crate::protocol::error::ProtocolError;

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

pub fn push_i32(buf: &mut Vec<u8>, value: i32) {
    buf.extend(value.to_be_bytes());
}