use std::{error::Error, fmt::Display};

use crate::grid::Mark;

const HELLO_MAGIC: u32 = 0xFD36_0084;
const EOG_MAGIC: u32 = 0x5CD9_0094;
const TERMINATOR: u8 = 0xFF;

#[derive(Debug, Clone)]
pub enum PacketParseError {
    InvalidSize,
    InvalidMagic,
}
impl Display for PacketParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error parsing packet: ")?;
        match self {
            Self::InvalidSize => write!(f, "Wrong packet size"),
            Self::InvalidMagic => write!(f, "Wrong magic value"),
        }
    }
}
impl Error for PacketParseError {}

#[derive(Debug, Clone, Copy)]
pub struct ClientHello;
impl TryFrom<&[u8]> for ClientHello {
    type Error = PacketParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 4 {
            return Err(PacketParseError::InvalidSize);
        }

        if value != HELLO_MAGIC.to_be_bytes() {
            return Err(PacketParseError::InvalidMagic);
        }

        Ok(Self)
    }
}
impl From<ClientHello> for [u8; 5] {
    fn from(_: ClientHello) -> Self {
        let mut pkt = [0_u8; 5];
        pkt[0..4].copy_from_slice(&HELLO_MAGIC.to_be_bytes());
        pkt[4] = TERMINATOR;
        pkt
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ServerHello {
    client_first: bool,
    client_mark: Mark,
}
impl TryFrom<&[u8]> for ServerHello {
    type Error = PacketParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 4 {
            return Err(PacketParseError::InvalidSize);
        }

        // Set last 2 bits to 0
        let mut x = [0_u8; 4];
        x.clone_from_slice(value);
        x[3] &= !0b11;
        if x != HELLO_MAGIC.to_be_bytes() {
            return Err(PacketParseError::InvalidMagic);
        }

        let client_first = (value[3] & 0b10) != 0;
        let client_mark = if (value[3] & 0b1) == 0 {
            Mark::O
        } else {
            Mark::X
        };

        Ok(Self {
            client_first,
            client_mark,
        })
    }
}
impl From<ServerHello> for [u8; 5] {
    fn from(value: ServerHello) -> Self {
        let mut pkt = [0_u8; 5];
        let magic_bytes = HELLO_MAGIC.to_be_bytes();
        pkt[0..4].copy_from_slice(&magic_bytes);

        let mut b = magic_bytes[3];
        if value.client_first {
            b |= 0b10;
        }
        if value.client_mark == Mark::X {
            b |= 1;
        }
        pkt[3] = b;
        pkt[4] = TERMINATOR;
        pkt
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerMove(usize, usize);
impl From<u8> for PlayerMove {
    fn from(value: u8) -> Self {
        let row = value >> 4;
        let col = value & 0b1111;
        Self(row as usize, col as usize)
    }
}
impl From<PlayerMove> for [u8; 2] {
    fn from(value: PlayerMove) -> Self {
        let mut pkt = [0_u8; 2];
        pkt[0] = (value.0 << 4) as u8 + (value.1 as u8 & 0b1111);
        pkt[1] = TERMINATOR;
        pkt
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EndOfGame;
impl TryFrom<&[u8]> for EndOfGame {
    type Error = PacketParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 4 {
            return Err(PacketParseError::InvalidSize);
        }

        if value != EOG_MAGIC.to_be_bytes() {
            return Err(PacketParseError::InvalidMagic);
        }
        Ok(Self)
    }
}
impl From<EndOfGame> for [u8; 5] {
    fn from(_: EndOfGame) -> Self {
        let mut pkt = [0_u8; 5];
        pkt[0..4].copy_from_slice(&EOG_MAGIC.to_be_bytes());
        pkt[4] = TERMINATOR;
        pkt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_client_hello_pkt_ser_de() {
        let bytes: [u8; 5] = ClientHello.into();
        assert_eq!(bytes[4], TERMINATOR);
        assert!(ClientHello::try_from(&bytes[0..4]).is_ok())
    }

    #[test]
    fn validate_server_hello_pkt_ser_de_1() {
        let pkt = ServerHello {
            client_first: true,
            client_mark: Mark::O,
        };
        let bytes: [u8; 5] = pkt.into();

        assert_eq!(bytes[4], TERMINATOR);
        let deserialized =
            ServerHello::try_from(&bytes[0..4]).expect("Error deserializing the byte value");
        assert_eq!(deserialized.client_mark, pkt.client_mark);
        assert_eq!(deserialized.client_first, pkt.client_first);
    }

    #[test]
    fn validate_server_hello_pkt_ser_de_2() {
        let pkt = ServerHello {
            client_first: false,
            client_mark: Mark::X,
        };
        let bytes: [u8; 5] = pkt.into();

        assert_eq!(bytes[4], TERMINATOR);
        let deserialized =
            ServerHello::try_from(&bytes[0..4]).expect("Error deserializing the byte value");
        assert_eq!(deserialized.client_mark, pkt.client_mark);
        assert_eq!(deserialized.client_first, pkt.client_first);
    }

    #[test]
    fn validate_player_move_pkt_ser_de() {
        let pkt = PlayerMove(15, 8);
        let bytes: [u8; 2] = pkt.into();

        assert_eq!(bytes[1], TERMINATOR);

        let deserialized = PlayerMove::from(bytes[0]);
        assert_eq!(pkt.0, deserialized.0);
        assert_eq!(pkt.1, deserialized.1);
    }

    #[test]
    fn validate_eog_pkt_ser_de() {
        let bytes: [u8; 5] = EndOfGame.into();
        assert_eq!(bytes[4], TERMINATOR);
        assert!(EndOfGame::try_from(&bytes[0..4]).is_ok())
    }
}
