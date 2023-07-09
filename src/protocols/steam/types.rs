use crate::misc::FromBytestream;

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

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub score: i32,
    pub duration: f32,
}

#[derive(Debug)]
pub struct Rule {
    pub name: String,
    pub value: String,
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
impl FromBytestream for Rule {
    fn next_data_le(bytes: &mut impl Iterator<Item = u8>) -> Option<Self> {
        let name = String::next_data_le(bytes)?;
        let value = String::next_data_le(bytes)?;

        Some(Rule { name, value })
    }
}


