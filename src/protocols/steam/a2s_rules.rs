use super::a2s;
use crate::misc::{FromBytestream, ServerQueryError};
use std::net::UdpSocket;

#[derive(Debug)]
pub struct A2SRules {
    pub rules: Vec<Rule>,
}

impl A2SRules {
    pub fn query(socket: &UdpSocket) -> Result<A2SRules, ServerQueryError> {
        let req_buf_init: [u8; 9] = *b"\xFF\xFF\xFF\xFF\x56\xFF\xFF\xFF\xFF";
        let mut req_buf: [u8; 9] = *b"\xFF\xFF\xFF\xFF\x56\xFF\xFF\xFF\xFF";

        let mut data = a2s::a2s(socket, &req_buf_init, &mut req_buf, 0x45)?;

        A2SRules::next_data_le(&mut data).ok_or(ServerQueryError::InvalidData)
    }
}

impl FromBytestream for A2SRules {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        let mut rules = Vec::new();

        let num_rules = u8::next_data_le(bytes)?;
        for _ in 0..num_rules {
            if let Some(rule) = Rule::next_data_le(bytes) {
                rules.push(rule);
            } else {
                println!("Warning: Could not read all rules");
                break;
            }
        }

        Some(A2SRules { rules })
    }
}

#[derive(Debug)]
pub struct Rule {
    pub name: String,
    pub value: String,
}

impl FromBytestream for Rule {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        let name = String::next_data_le(bytes)?;
        let value = String::next_data_le(bytes)?;

        Some(Rule { name, value })
    }
}
