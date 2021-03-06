//! Serialise and deserialise common types in packets

use byteorder::{ByteOrder, NetworkEndian};
use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Direction {
    ToGameserver,
    FromGameserver,
    ToLoginserver,
    FromLoginserver,
}

impl From<u8> for Direction {
    fn from(item: u8) -> Self {
        match item {
            0 => Direction::ToGameserver,
            1 => Direction::FromGameserver,
            2 => Direction::ToLoginserver,
            _ => Direction::FromLoginserver,
        }
    }
}

impl From<Direction> for u8 {
    fn from(item: Direction) -> Self {
        match item {
            Direction::ToGameserver => 0,
            Direction::FromGameserver => 1,
            Direction::ToLoginserver => 2,
            Direction::FromLoginserver => 3,
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Direction::ToGameserver => "C  -> GS",
                Direction::FromGameserver => "C <-  GS",
                Direction::ToLoginserver => "C  -> LS",
                Direction::FromLoginserver => "C <-  LS",
            }
        )
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct PoePacket {
    pub direction: Direction,
    pub ip: std::net::Ipv4Addr,
    pub port: u16,
    pub payload: Vec<u8>,
}

impl fmt::Display for PoePacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}({}) id: {:02x?}, size: {}",
            self.direction,
            self.ip,
            &self.payload[..2],
            self.payload.len()
        )
    }
}

impl PoePacket {
    pub fn new(
        payload_slice: &[u8],
        direction: Direction,
        ip: std::net::Ipv4Addr,
        port: u16,
    ) -> Self {
        let mut payload = Vec::with_capacity(payload_slice.len());
        payload.extend_from_slice(&payload_slice[..]);

        PoePacket {
            direction,
            payload,
            ip,
            port,
        }
    }

    pub fn to_buf(&self) -> Vec<u8> {
        let mut buf = vec![0u8; 9 + self.payload.len()];
        buf[0] = self.direction.into();
        NetworkEndian::write_u16(&mut buf[1..3], self.port);
        NetworkEndian::write_u32(&mut buf[3..7], self.ip.into());
        NetworkEndian::write_u16(&mut buf[7..9], self.payload.len() as u16);
        buf.extend_from_slice(&self.payload[..]);

        buf
    }

    pub fn from_buf(buf: &[u8]) -> Result<Self, crate::Error> {
        if buf.len() < 9 {
            return Err(crate::Error::CanNotDeserialize);
        }
        let direction = buf[0].into();
        let port = NetworkEndian::read_u16(&buf[1..3]);
        let ip = NetworkEndian::read_u32(&buf[3..7]).into();
        let size = NetworkEndian::read_u16(&buf[7..9]) as usize;

        if buf.len() < 9 + size {
            return Err(crate::Error::CanNotDeserialize);
        }

        // TODO: payload begins from index 11 or 15, instead of expected 9
        let mut payload = Vec::with_capacity(size);
        payload.extend_from_slice(&buf[buf.len() - size..]);

        let packet = PoePacket {
            direction,
            payload,
            ip,
            port,
        };

        Ok(packet)
    }
}
