use std::io::Error as IOError;

#[derive(Debug)]
pub enum ServerQueryError {
    CouldNotCreateSocket(IOError),
    CouldNotConnect(IOError),
    CouldNotSend(IOError),
    CouldNotReceive(IOError),
    InvalidData,
}

pub trait FromBytestream: Sized {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self>;
}

impl FromBytestream for u8 {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        Some(u8::from_le_bytes([bytes.next()?]))
    }
}

impl FromBytestream for bool {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        Some(u8::from_le_bytes([bytes.next()?]) != 0)
    }
}

impl FromBytestream for i16 {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        Some(i16::from_le_bytes([bytes.next()?, bytes.next()?]))
    }
}

impl FromBytestream for i32 {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        Some(i32::from_le_bytes([
            bytes.next()?,
            bytes.next()?,
            bytes.next()?,
            bytes.next()?,
        ]))
    }
}

impl FromBytestream for f32 {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        Some(f32::from_le_bytes([
            bytes.next()?,
            bytes.next()?,
            bytes.next()?,
            bytes.next()?,
        ]))
    }
}

impl FromBytestream for u64 {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        Some(u64::from_le_bytes([
            bytes.next()?,
            bytes.next()?,
            bytes.next()?,
            bytes.next()?,
            bytes.next()?,
            bytes.next()?,
            bytes.next()?,
            bytes.next()?,
        ]))
    }
}

impl FromBytestream for String {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        let chars: Vec<u8> = bytes.take_while(|&b| b != 0).collect();
        Some(String::from_utf8_lossy(&chars).to_string())
    }
}

