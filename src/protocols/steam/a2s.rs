use super::packet::receive_bytes;
use crate::misc::ServerQueryError;
use std::{net::UdpSocket, vec::IntoIter};

pub fn a2s(
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
        return Err(ServerQueryError::InvalidData);
    }

    let mut it = bytes.into_iter();
    it.next();

    Ok(it)
}

fn set_challenge(buf: &mut [u8], challenge: &[u8]) {
    assert!(buf.len() >= challenge.len());

    let offset = buf.len() - challenge.len();
    for (i, c) in challenge.iter().enumerate() {
        buf[offset + i] = *c;
    }
}
