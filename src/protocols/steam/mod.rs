mod types;

use crate::misc::*;
use std::net::UdpSocket;
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

        let res_buf = a2s(socket, &req_buf_init, &mut req_buf, 0x49)?;
        let mut data = res_buf.into_iter().skip(5);

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

        let res_buf = a2s(socket, &req_buf_init, &mut req_buf, 0x44)?;
        let mut data = res_buf.into_iter().skip(5);

        let num_players = u8::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?;
        let mut players = Vec::new();
        for _ in 0..num_players {
            let player = Player::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?;
            players.push(player);
        }

        Ok(A2SPlayer { players })
    }
}

#[derive(Debug)]
pub struct A2Rules {
    pub rules: Vec<Rule>,
}

impl A2Rules {
    pub fn query(socket: &UdpSocket) -> Result<A2Rules, ServerQueryError> {
        let req_buf_init: [u8; 9] = *b"\xFF\xFF\xFF\xFF\x56\xFF\xFF\xFF\xFF";
        let mut req_buf: [u8; 9] = *b"\xFF\xFF\xFF\xFF\x56\xFF\xFF\xFF\xFF";

        let res_buf = a2s(socket, &req_buf_init, &mut req_buf, 0x45)?;
        let mut data = res_buf.into_iter().skip(5);

        let mut rules = Vec::new();
        let num_rules = u8::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?;
        for _ in 0..num_rules {
            let rule = Rule::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)?;
            rules.push(rule);
        }

        Ok(A2Rules { rules })
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
) -> Result<[u8; 1400], ServerQueryError> {
    let mut res_buf: [u8; 1400] = [0; 1400];
    let offset = 4;

    socket
        .send(&first_req)
        .map_err(ServerQueryError::CouldNotSend)?;

    socket
        .recv(&mut res_buf)
        .map_err(ServerQueryError::CouldNotReceive)?;

    // The server returned a challenge
    if res_buf[offset] == 0x41 {
        set_challenge(req_buf, &res_buf[offset + 1..offset + 5]);

        socket
            .send(&req_buf)
            .map_err(ServerQueryError::CouldNotSend)?;

        socket
            .recv(&mut res_buf)
            .map_err(ServerQueryError::CouldNotReceive)?;
    }

    if res_buf[offset] != res_header {
        return Err(ServerQueryError::InvalidData);
    }

    Ok(res_buf)
}
