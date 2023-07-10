mod types;

use crate::misc::*;
use std::{net::UdpSocket, vec::IntoIter};
use types::{Environment, Player, Rule, ServerType};

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

        let mut data = a2s(socket, &req_buf_init, &mut req_buf, 0x49)?.into_iter();

        Ok(A2SInfo {
            protocol: u8::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?,
            name: String::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?,
            map: String::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?,
            folder: String::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?,
            game: String::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?,
            game_id: i16::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?,
            players: u8::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?,
            max_players: u8::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?,
            bots: u8::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?,
            server_type: ServerType::next_data_le(&mut data)
                .ok_or(ServerQueryError::InvalidData)?,
            environment: Environment::next_data_le(&mut data)
                .ok_or(ServerQueryError::InvalidData)?,
            password: bool::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?,
            vac: bool::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?,
        })
    }
}

#[derive(Debug)]
pub struct A2SPlayer {
    pub players: Vec<Player>,
}

impl A2SPlayer {
    pub fn query(socket: &UdpSocket) -> Result<Self, ServerQueryError> {
        let req_buf_init: [u8; 9] = *b"\xFF\xFF\xFF\xFF\x55\xFF\xFF\xFF\xFF";
        let mut req_buf: [u8; 9] = *b"\xFF\xFF\xFF\xFF\x55\xFF\xFF\xFF\xFF";

        let mut data = a2s(socket, &req_buf_init, &mut req_buf, 0x44)?;

        let num_players = u8::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?;
        let mut players = Vec::new();
        for _ in 0..num_players as usize {
            if let Some(player) = Player::next_data_le(&mut data) {
                players.push(player);
            } else {
                println!("Warning: Could not read all players");
                break;
            }
        }

        Ok(A2SPlayer { players })
    }
}

#[derive(Debug)]
pub struct A2SRules {
    pub rules: Vec<Rule>,
}

impl A2SRules {
    pub fn query(socket: &UdpSocket) -> Result<A2SRules, ServerQueryError> {
        let req_buf_init: [u8; 9] = *b"\xFF\xFF\xFF\xFF\x56\xFF\xFF\xFF\xFF";
        let mut req_buf: [u8; 9] = *b"\xFF\xFF\xFF\xFF\x56\xFF\xFF\xFF\xFF";

        let mut data = a2s(socket, &req_buf_init, &mut req_buf, 0x45)?;

        let mut rules = Vec::new();
        let num_rules = u8::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?;
        for _ in 0..num_rules {
            if let Some(rule) = Rule::next_data_le(&mut data) {
                rules.push(rule);
            }
            else {
                println!("Warning: Could not read all rules");
                break;
            }
        }

        Ok(A2SRules { rules })
    }
}

fn set_challenge(buf: &mut [u8], challenge: &[u8]) {
    assert!(buf.len() >= challenge.len());

    let offset = buf.len() - challenge.len();
    for (i, c) in challenge.iter().enumerate() {
        buf[offset + i] = *c;
    }
}

fn a2s(
    socket: &UdpSocket,
    first_req: &[u8],
    req_buf: &mut [u8],
    res_header: u8,
) -> Result<IntoIter<u8>, ServerQueryError> {
    socket
        .send(&first_req)
        .map_err(ServerQueryError::CouldNotSend)?;

    let mut bytes = receive_bytes(socket)?;

    // The server returned a challenge
    while bytes.len() >= 5 && bytes[0] == 0x41 {
        set_challenge(req_buf, &bytes[1..5]);

        socket
            .send(&req_buf)
            .map_err(ServerQueryError::CouldNotSend)?;

        bytes = receive_bytes(socket)?;
    }

    if bytes.is_empty() || bytes[0] != res_header {
        println!("ABC: {:x}", bytes[0]);
        return Err(ServerQueryError::InvalidData);
    }

    let mut it = bytes.into_iter();
    it.next();

    Ok(it)
}

fn receive_bytes(socket: &UdpSocket) -> Result<Vec<u8>, ServerQueryError> {
    let mut res_buf: [u8; 1400] = [0; 1400];
    let bytes: Vec<u8>;

    socket
        .recv(&mut res_buf)
        .map_err(ServerQueryError::CouldNotReceive)?;

    let mut data = res_buf.into_iter();
    let header = i32::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?;

    // Multi-packet response
    if header == -2 {
        let first_packet =
            Multipacket::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?;

        let mut packets = vec![Vec::<u8>::new(); first_packet.total as usize];
        packets[first_packet.number as usize] = first_packet.payload;

        for _ in 1..first_packet.total as usize {
            socket
                .recv(&mut res_buf)
                .map_err(ServerQueryError::CouldNotReceive)?;

            data = res_buf.into_iter();
            let header = i32::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?;

            if header != -2 {
                return Err(ServerQueryError::InvalidData);
            }

            let packet =
                Multipacket::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?;

            packets[packet.number as usize] = packet.payload;
        }

        bytes = packets.into_iter().flatten().collect();
    } else {
        bytes = data.collect();
    }

    Ok(bytes)
}

struct Multipacket {
    _id: i32,
    total: u8,
    number: u8,
    _max_packet_size: i16,
    _size: Option<i32>,
    _crc32_sum: Option<i32>,
    payload: Vec<u8>,
}

impl FromBytestream for Multipacket {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        let id_field = i32::next_data_le(bytes)?;
        let _id = id_field & 0x7FFFFFFF;
        let compressed = (id_field & 80000000) != 0;
        let total = u8::next_data_le(bytes)?;
        let number = u8::next_data_le(bytes)?;

        // Warning: Some older games does not have this field
        let _max_packet_size = i16::next_data_le(bytes)?;

        let (_size, _crc32_sum) = (None, None);

        if _id == 0 && compressed {
            println!("ERROR: Compressed packets are not suppored");
            return None;
        }

        let mut payload: Vec<u8> = bytes.collect();
        
        // Some games put the single packet header in multi packet payload.
        // If that is the case, then we want to remove it
        if payload.len() >= 4 && payload[0..4] == *b"\xFF\xFF\xFF\xFF" {
            payload = payload[4..].to_vec();
        }

        Some(Multipacket {
            _id,
            total,
            number,
            _max_packet_size,
            _size,
            _crc32_sum,
            payload,
        })
    }
}
