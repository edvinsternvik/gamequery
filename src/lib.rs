mod misc;
pub mod protocols;
use crate::misc::*;
use protocols::steam::{a2s_info::A2SInfo, a2s_players::A2SPlayers};
use std::{
    net::{ToSocketAddrs, UdpSocket},
    time::Duration,
};

pub struct ServerQuerySettings<A: ToSocketAddrs> {
    pub ip: A,
    pub timeout: Option<Duration>,
}

impl<A: ToSocketAddrs> ServerQuerySettings<A> {
    pub fn create_socket(&self) -> Result<UdpSocket, ServerQueryError> {
        let socket =
            UdpSocket::bind("0.0.0.0:0").map_err(ServerQueryError::CouldNotCreateSocket)?;

        socket
            .set_read_timeout(self.timeout)
            .map_err(ServerQueryError::CouldNotCreateSocket)?;
        socket
            .set_write_timeout(self.timeout)
            .map_err(ServerQueryError::CouldNotCreateSocket)?;

        socket
            .connect(&self.ip)
            .map_err(ServerQueryError::CouldNotConnect)?;

        return Ok(socket);
    }
}

pub trait ServerInfo: Sized {
    fn query<A: ToSocketAddrs>(settings: ServerQuerySettings<A>) -> Result<Self, ServerQueryError>;

    fn name(&self) -> &String;
    fn players(&self) -> &Vec<String>;
}

#[derive(Debug)]
pub struct SteamServerInfo {
    name: String,
    players: Vec<String>,
}

impl ServerInfo for SteamServerInfo {
    fn query<A: ToSocketAddrs>(settings: ServerQuerySettings<A>) -> Result<Self, ServerQueryError> {
        let socket = settings.create_socket()?;

        let a2s_info = A2SInfo::query(&socket)?;
        let a2s_players = A2SPlayers::query(&socket)?;

        Ok(Self {
            name: a2s_info.name,
            players: a2s_players.players.iter().map(|p| p.name.clone()).collect(),
        })
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn players(&self) -> &Vec<String> {
        &self.players
    }
}
