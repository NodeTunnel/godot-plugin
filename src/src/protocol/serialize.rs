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

pub fn read_vec_i32(bytes: &[u8]) -> Result<(Vec<i32>, &[u8]), ProtocolError> {
    let (len, mut rest) = read_i32(bytes)?;

    if len < 0 {
        return Err(ProtocolError::NegativeVectorLength());
    }

    let mut values = Vec::with_capacity(len as usize);
    for _ in 0..len {
        let (value, remaining) = read_i32(rest)?;
        values.push(value);
        rest = remaining;
    }

    Ok((values, rest))
}

pub fn push_string(buf: &mut Vec<u8>, value: &str) {
    let bytes = value.as_bytes();
    buf.extend((bytes.len() as i32).to_be_bytes());
    buf.extend(bytes);
}

pub fn push_i32(buf: &mut Vec<u8>, value: i32) {
    buf.extend(value.to_be_bytes());
}

pub fn push_vec_i32(buf: &mut Vec<u8>, values: &[i32]) {
    push_i32(buf, values.len() as i32);
    for value in values {
        push_i32(buf, *value);
    }
}