use std::net::UdpSocket;
use crate::misc::{ServerQueryError, FromBytestream};

pub fn receive_bytes(socket: &UdpSocket) -> Result<Vec<u8>, ServerQueryError> {
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

