use std::error::Error;

pub fn read_i32(bytes: &[u8]) -> Result<(i32, &[u8]), &'static str> {
    if bytes.len() < 4 {
        return Err("Not enough bytes for i32");
    }
    let value = i32::from_be_bytes(bytes[..4].try_into().unwrap());
    Ok((value, &bytes[4..]))
}

pub fn read_string(bytes: &[u8]) -> Result<(String, &[u8]), Box<dyn Error>> {
    let (len, rest) = read_i32(bytes)?;        // read length prefix

    if rest.len() < len as usize {
        return Err("Not enough bytes for string".into());
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