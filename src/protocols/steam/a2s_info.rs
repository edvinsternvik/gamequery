use std::net::UdpSocket;
use crate::misc::{FromBytestream, ServerQueryError};
use super::a2s;

#[derive(Debug)]
pub struct A2SInfo {
    pub protocol: u8,
    pub name: String,
    pub map: String,
    pub folder: String,
    pub game: String,
    pub game_id: i16,
    pub players: u8,
    pub max_players: u8,
    pub bots: u8,
    pub server_type: ServerType,
    pub environment: Environment,
    pub password: bool,
    pub vac: bool,
}

impl A2SInfo {
    pub fn query(socket: &UdpSocket) -> Result<Self, ServerQueryError> {
        let req_buf_init: [u8; 25] = *b"\xFF\xFF\xFF\xFF\x54Source Engine Query\x00";
        let mut req_buf: [u8; 29] = *b"\xFF\xFF\xFF\xFF\x54Source Engine Query\x00\xFF\xFF\xFF\xFF";

        let mut data = a2s::a2s(socket, &req_buf_init, &mut req_buf, 0x49)?.into_iter();

        A2SInfo::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)
    }
}

impl FromBytestream for A2SInfo {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        Some(A2SInfo {
            protocol: u8::next_data_le(bytes)?,
            name: String::next_data_le(bytes)?,
            map: String::next_data_le(bytes)?,
            folder: String::next_data_le(bytes)?,
            game: String::next_data_le(bytes)?,
            game_id: i16::next_data_le(bytes)?,
            players: u8::next_data_le(bytes)?,
            max_players: u8::next_data_le(bytes)?,
            bots: u8::next_data_le(bytes)?,
            server_type: ServerType::next_data_le(bytes)?,
            environment: Environment::next_data_le(bytes)?,
            password: bool::next_data_le(bytes)?,
            vac: bool::next_data_le(bytes)?,
        })
    }
}

#[derive(Debug)]
pub enum ServerType {
    Dedicated,
    NonDedicated,
    Proxy,
}

#[derive(Debug)]
pub enum Environment {
    Linux,
    Windows,
    Mac,
}

impl FromBytestream for ServerType {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        match u8::next_data_le(bytes) {
            Some(b'd') => Some(ServerType::Dedicated),
            Some(b'l') => Some(ServerType::NonDedicated),
            Some(b'p') => Some(ServerType::Proxy),
            _ => None,
        }
    }
}

impl FromBytestream for Environment {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        match u8::next_data_le(bytes) {
            Some(b'l') => Some(Environment::Linux),
            Some(b'w') => Some(Environment::Windows),
            Some(b'm') => Some(Environment::Mac),
            Some(b'o') => Some(Environment::Mac),
            _ => None,
        }
    }
}
