use super::a2s;
use crate::misc::{FromBytestream, ServerQueryError};
use std::net::UdpSocket;

#[derive(Debug)]
pub struct A2SPlayers {
    pub players: Vec<Player>,
}

impl A2SPlayers {
    pub fn query(socket: &UdpSocket) -> Result<Self, ServerQueryError> {
        let req_buf_init: [u8; 9] = *b"\xFF\xFF\xFF\xFF\x55\xFF\xFF\xFF\xFF";
        let mut req_buf: [u8; 9] = *b"\xFF\xFF\xFF\xFF\x55\xFF\xFF\xFF\xFF";

        let mut data = a2s::a2s(socket, &req_buf_init, &mut req_buf, 0x44)?;

        A2SPlayers::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)
    }
}

impl FromBytestream for A2SPlayers {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        let num_players = u8::next_data_le(bytes)?;

        let mut players = Vec::new();
        for _ in 0..num_players as usize {
            if let Some(player) = Player::next_data_le(bytes) {
                players.push(player);
            } else {
                println!("Warning: Could not read all players");
                break;
            }
        }

        Some(A2SPlayers { players })
    }
}

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub score: i32,
    pub duration: f32,
}

impl FromBytestream for Player {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        let _index = u8::next_data_le(bytes)?;
        let name = String::next_data_le(bytes)?;
        let score = i32::next_data_le(bytes)?;
        let duration = f32::next_data_le(bytes)?;

        Some(Player {
            name,
            score,
            duration,
        })
    }
}
