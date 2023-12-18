// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP19
//!
//! <https://github.com/nostr-protocol/nips/blob/master/19.md>

#![allow(missing_docs)]

use alloc::string::FromUtf8Error;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

use bitcoin::bech32::{self, FromBase32, ToBase32, Variant};
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{self, SecretKey, XOnlyPublicKey};

use super::nip01::Coordinate;
use crate::event::id::{self, EventId};
use crate::Kind;

pub const PREFIX_BECH32_SECRET_KEY: &str = "nsec";
pub const PREFIX_BECH32_PUBLIC_KEY: &str = "npub";
pub const PREFIX_BECH32_NOTE_ID: &str = "note";
pub const PREFIX_BECH32_PROFILE: &str = "nprofile";
pub const PREFIX_BECH32_EVENT: &str = "nevent";
pub const PREFIX_BECH32_PARAMETERIZED_REPLACEABLE_EVENT: &str = "naddr";

pub const SPECIAL: u8 = 0;
pub const RELAY: u8 = 1;
pub const AUTHOR: u8 = 2;
pub const KIND: u8 = 3;

/// `NIP19` error
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// Bech32 error.
    Bech32(bech32::Error),
    /// UFT-8 error
    UTF8(FromUtf8Error),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Hash error
    Hash(bitcoin::hashes::Error),
    /// EventId error
    EventId(id::Error),
    /// Wrong prefix or variant
    WrongPrefixOrVariant,
    /// Not implemented
    NotImplemented,
    /// Field missing
    FieldMissing(String),
    /// TLV error
    TLV,
    /// From slice error
    TryFromSlice,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bech32(e) => write!(f, "Bech32: {e}"),
            Self::UTF8(e) => write!(f, "UTF8: {e}"),
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
            Self::Hash(e) => write!(f, "Hash: {e}"),
            Self::EventId(e) => write!(f, "Event ID: {e}"),
            Self::WrongPrefixOrVariant => write!(f, "Wrong prefix or variant"),
            Self::FieldMissing(name) => write!(f, "Field missing: {name}"),
            Self::TLV => write!(f, "TLV (type-length-value) error"),
            Self::TryFromSlice => write!(f, "Impossible to perform conversion from slice"),
            Self::NotImplemented => write!(f, "Not implemented"),
        }
    }
}

impl From<bech32::Error> for Error {
    fn from(e: bech32::Error) -> Self {
        Self::Bech32(e)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Self::UTF8(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<bitcoin::hashes::Error> for Error {
    fn from(e: bitcoin::hashes::Error) -> Self {
        Self::Hash(e)
    }
}

impl From<id::Error> for Error {
    fn from(e: id::Error) -> Self {
        Self::EventId(e)
    }
}

/// To ensure total matching on prefixes when decoding a [`Nip19`] object
enum Nip19Prefix {
    /// nsec
    NSec,
    /// npub
    NPub,
    /// note
    Note,
    /// nprofile
    NProfile,
    /// nevent
    NEvent,
    /// naddr
    NAddr,
}

/// Convert NIP19 [`&str`] prefixes to [`Nip19Prefix`]
impl TryFrom<&str> for Nip19Prefix {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            PREFIX_BECH32_SECRET_KEY => Ok(Nip19Prefix::NSec),
            PREFIX_BECH32_PUBLIC_KEY => Ok(Nip19Prefix::NPub),
            PREFIX_BECH32_NOTE_ID => Ok(Nip19Prefix::Note),
            PREFIX_BECH32_PROFILE => Ok(Nip19Prefix::NProfile),
            PREFIX_BECH32_EVENT => Ok(Nip19Prefix::NEvent),
            PREFIX_BECH32_PARAMETERIZED_REPLACEABLE_EVENT => Ok(Nip19Prefix::NAddr),
            _ => Err(Error::WrongPrefixOrVariant),
        }
    }
}

/// Convert NIP19 [`String`] prefixes to [`Nip19Prefix`]
impl TryFrom<String> for Nip19Prefix {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

/// A representation any `fNIP19` bech32 nostr object. Useful for decoding
/// `NIP19` bech32 strings without necessarily knowing what you're decoding
/// ahead of time.
#[derive(Debug, Eq, PartialEq)]
pub enum Nip19 {
    /// nsec
    Secret(SecretKey),
    /// npub
    Pubkey(XOnlyPublicKey),
    /// nprofile
    Profile(Nip19Profile),
    /// note
    EventId(EventId),
    /// nevent
    Event(Nip19Event),
    /// naddr
    Coordinate(Coordinate),
}

pub trait FromBech32: Sized {
    type Err;
    fn from_bech32<S>(s: S) -> Result<Self, Self::Err>
    where
        S: AsRef<str>;
}

impl FromBech32 for SecretKey {
    type Err = Error;
    fn from_bech32<S>(secret_key: S) -> Result<Self, Self::Err>
    where
        S: AsRef<str>,
    {
        let (hrp, data, checksum) = bech32::decode(secret_key.as_ref())?;

        if hrp != PREFIX_BECH32_SECRET_KEY || checksum != Variant::Bech32 {
            return Err(Error::WrongPrefixOrVariant);
        }

        let data = Vec::<u8>::from_base32(&data)?;
        Ok(Self::from_slice(data.as_slice())?)
    }
}

impl FromBech32 for XOnlyPublicKey {
    type Err = Error;
    fn from_bech32<S>(public_key: S) -> Result<Self, Self::Err>
    where
        S: AsRef<str>,
    {
        let (hrp, data, checksum) = bech32::decode(public_key.as_ref())?;

        if hrp != PREFIX_BECH32_PUBLIC_KEY || checksum != Variant::Bech32 {
            return Err(Error::WrongPrefixOrVariant);
        }

        let data = Vec::<u8>::from_base32(&data)?;
        Ok(Self::from_slice(data.as_slice())?)
    }
}

impl FromBech32 for Nip19 {
    type Err = Error;
    fn from_bech32<S>(hash: S) -> Result<Self, Self::Err>
    where
        S: AsRef<str>,
    {
        let (hrp, data, checksum) = bech32::decode(hash.as_ref())?;
        let prefix: Nip19Prefix = hrp.try_into()?;

        if checksum != Variant::Bech32 {
            return Err(Error::WrongPrefixOrVariant);
        }

        let data: Vec<u8> = Vec::<u8>::from_base32(&data)?;

        match prefix {
            Nip19Prefix::NSec => Ok(Self::Secret(SecretKey::from_slice(data.as_slice())?)),
            Nip19Prefix::NPub => Ok(Self::Pubkey(XOnlyPublicKey::from_slice(data.as_slice())?)),
            Nip19Prefix::NProfile => Ok(Self::Profile(Nip19Profile::from_bech32_data(data)?)),
            Nip19Prefix::NEvent => Ok(Self::Event(Nip19Event::from_bech32_data(data)?)),
            Nip19Prefix::Note => Ok(Self::EventId(EventId::from_slice(data.as_slice())?)),
            Nip19Prefix::NAddr => Ok(Self::Coordinate(Coordinate::from_bech32_data(data)?)),
        }
    }
}

impl ToBech32 for Nip19 {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        match self {
            Nip19::Secret(sec) => sec.to_bech32(),
            Nip19::Pubkey(pubkey) => pubkey.to_bech32(),
            Nip19::Event(event) => event.to_bech32(),
            Nip19::Profile(profile) => profile.to_bech32(),
            Nip19::EventId(event_id) => event_id.to_bech32(),
            Nip19::Coordinate(coordinate) => coordinate.to_bech32(),
        }
    }
}

impl FromBech32 for EventId {
    type Err = Error;
    fn from_bech32<S>(hash: S) -> Result<Self, Self::Err>
    where
        S: AsRef<str>,
    {
        let (hrp, data, checksum) = bech32::decode(hash.as_ref())?;

        if hrp != PREFIX_BECH32_NOTE_ID || checksum != Variant::Bech32 {
            return Err(Error::WrongPrefixOrVariant);
        }

        let data: Vec<u8> = Vec::<u8>::from_base32(&data)?;
        Ok(Self::from_slice(data.as_slice())?)
    }
}

pub trait ToBech32 {
    type Err;
    fn to_bech32(&self) -> Result<String, Self::Err>;
}

impl ToBech32 for XOnlyPublicKey {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let data = self.serialize().to_base32();
        Ok(bech32::encode(
            PREFIX_BECH32_PUBLIC_KEY,
            data,
            Variant::Bech32,
        )?)
    }
}

impl ToBech32 for SecretKey {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let data = self.secret_bytes().to_base32();
        Ok(bech32::encode(
            PREFIX_BECH32_SECRET_KEY,
            data,
            Variant::Bech32,
        )?)
    }
}

// Note ID
impl ToBech32 for EventId {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let data = self.to_base32();
        Ok(bech32::encode(
            PREFIX_BECH32_NOTE_ID,
            data,
            Variant::Bech32,
        )?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Nip19Event {
    pub event_id: EventId,
    pub kind: Option<Kind>,
    pub author: Option<XOnlyPublicKey>,
    pub relays: Vec<String>,
}

impl Nip19Event {
    pub fn new<I, S>(event_id: EventId, relays: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            event_id,
            author: None,
            kind: None,
            relays: relays.into_iter().map(|u| u.into()).collect(),
        }
    }

    fn from_bech32_data(mut data: Vec<u8>) -> Result<Self, Error> {
        let mut event_id: Option<EventId> = None;
        let mut author: Option<XOnlyPublicKey> = None;
        let mut kind: Option<Kind> = None;
        let mut relays: Vec<String> = Vec::new();

        while !data.is_empty() {
            let t = data.first().ok_or(Error::TLV)?;
            let l = data.get(1).ok_or(Error::TLV)?;
            let l = *l as usize;

            let bytes = data.get(2..l + 2).ok_or(Error::TLV)?;

            match *t {
                SPECIAL => {
                    if event_id.is_none() {
                        event_id = Some(EventId::from_slice(bytes)?);
                    }
                }
                // from nip19: "for nevent, *optionally*, the 32 bytes of
                // the pubkey of the event"
                AUTHOR => {
                    if author.is_none() {
                        author = Some(XOnlyPublicKey::from_slice(bytes)?);
                    }
                }
                // nip19: "for nevent, optionally, the 32-bit unsigned
                // integer of the kind, big-endian"
                KIND => {
                    if kind.is_none() {
                        let bytes: [u8; 4] = match bytes.try_into() {
                            Ok(bytes) => bytes,
                            Err(_) => return Err(Error::TryFromSlice),
                        };
                        // we only have From<u64> for Kind
                        let u64_kind = u32::from_be_bytes(bytes) as u64;
                        kind = Some(u64_kind.into());
                    }
                }
                RELAY => {
                    relays.push(String::from_utf8(bytes.to_vec())?);
                }
                _ => (),
            };

            data.drain(..l + 2);
        }

        Ok(Self {
            event_id: event_id.ok_or_else(|| Error::FieldMissing("event id".to_string()))?,
            author,
            kind,
            relays,
        })
    }
}

impl FromBech32 for Nip19Event {
    type Err = Error;
    fn from_bech32<S>(s: S) -> Result<Self, Self::Err>
    where
        S: AsRef<str>,
    {
        let (hrp, data, checksum) = bech32::decode(s.as_ref())?;

        if hrp != PREFIX_BECH32_EVENT || checksum != Variant::Bech32 {
            return Err(Error::WrongPrefixOrVariant);
        }

        let data: Vec<u8> = Vec::from_base32(&data)?;
        Self::from_bech32_data(data)
    }
}

impl ToBech32 for Nip19Event {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let mut bytes: Vec<u8> = vec![SPECIAL, 32];
        bytes.extend(self.event_id.inner().as_byte_array());

        for relay in self.relays.iter() {
            bytes.extend([RELAY, relay.len() as u8]);
            bytes.extend(relay.as_bytes());
        }

        let data = bytes.to_base32();
        Ok(bech32::encode(PREFIX_BECH32_EVENT, data, Variant::Bech32)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Nip19Profile {
    pub public_key: XOnlyPublicKey,
    pub relays: Vec<String>,
}

impl Nip19Profile {
    pub fn new<I, S>(public_key: XOnlyPublicKey, relays: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            public_key,
            relays: relays.into_iter().map(|u| u.into()).collect(),
        }
    }

    fn from_bech32_data(mut data: Vec<u8>) -> Result<Self, Error> {
        let mut public_key: Option<XOnlyPublicKey> = None;
        let mut relays: Vec<String> = Vec::new();

        while !data.is_empty() {
            let t = data.first().ok_or(Error::TLV)?;
            let l = data.get(1).ok_or(Error::TLV)?;
            let l = *l as usize;

            let bytes = data.get(2..l + 2).ok_or(Error::TLV)?;

            match *t {
                SPECIAL => {
                    if public_key.is_none() {
                        public_key = Some(XOnlyPublicKey::from_slice(bytes)?);
                    }
                }
                RELAY => {
                    relays.push(String::from_utf8(bytes.to_vec())?);
                }
                _ => (),
            };

            data.drain(..l + 2);
        }

        Ok(Self {
            public_key: public_key.ok_or_else(|| Error::FieldMissing("pubkey".to_string()))?,
            relays,
        })
    }
}

impl ToBech32 for Nip19Profile {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let mut bytes: Vec<u8> = vec![SPECIAL, 32];
        bytes.extend(self.public_key.serialize());

        for relay in self.relays.iter() {
            bytes.extend([RELAY, relay.len() as u8]);
            bytes.extend(relay.as_bytes());
        }

        let data = bytes.to_base32();
        Ok(bech32::encode(
            PREFIX_BECH32_PROFILE,
            data,
            Variant::Bech32,
        )?)
    }
}

impl FromBech32 for Nip19Profile {
    type Err = Error;
    fn from_bech32<S>(s: S) -> Result<Self, Self::Err>
    where
        S: AsRef<str>,
    {
        let (hrp, data, checksum) = bech32::decode(s.as_ref())?;

        if hrp != PREFIX_BECH32_PROFILE || checksum != Variant::Bech32 {
            return Err(Error::WrongPrefixOrVariant);
        }

        let data: Vec<u8> = Vec::from_base32(&data)?;
        Self::from_bech32_data(data)
    }
}

impl Coordinate {
    fn from_bech32_data(mut data: Vec<u8>) -> Result<Self, Error> {
        let mut identifier: Option<String> = None;
        let mut pubkey: Option<XOnlyPublicKey> = None;
        let mut kind: Option<Kind> = None;
        let mut relays: Vec<String> = Vec::new();

        while !data.is_empty() {
            let t = data.first().ok_or(Error::TLV)?;
            let l = data.get(1).ok_or(Error::TLV)?;
            let l = *l as usize;

            let bytes: &[u8] = data.get(2..l + 2).ok_or(Error::TLV)?;

            match *t {
                SPECIAL => {
                    if identifier.is_none() {
                        identifier = Some(String::from_utf8(bytes.to_vec())?);
                    }
                }
                RELAY => {
                    relays.push(String::from_utf8(bytes.to_vec())?);
                }
                AUTHOR => {
                    if pubkey.is_none() {
                        pubkey = Some(XOnlyPublicKey::from_slice(bytes)?);
                    }
                }
                KIND => {
                    if kind.is_none() {
                        let k: u64 =
                            u32::from_be_bytes(bytes.try_into().map_err(|_| Error::TryFromSlice)?)
                                as u64;
                        kind = Some(Kind::from(k));
                    }
                }
                _ => (),
            };

            data.drain(..l + 2);
        }

        Ok(Self {
            kind: kind.ok_or_else(|| Error::FieldMissing("kind".to_string()))?,
            pubkey: pubkey.ok_or_else(|| Error::FieldMissing("pubkey".to_string()))?,
            identifier: identifier.ok_or_else(|| Error::FieldMissing("identifier".to_string()))?,
            relays,
        })
    }
}

impl FromBech32 for Coordinate {
    type Err = Error;
    fn from_bech32<S>(s: S) -> Result<Self, Self::Err>
    where
        S: AsRef<str>,
    {
        let (hrp, data, checksum) = bech32::decode(s.as_ref())?;

        if hrp != PREFIX_BECH32_PARAMETERIZED_REPLACEABLE_EVENT || checksum != Variant::Bech32 {
            return Err(Error::WrongPrefixOrVariant);
        }

        let data: Vec<u8> = Vec::from_base32(&data)?;
        Self::from_bech32_data(data)
    }
}

impl ToBech32 for Coordinate {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let mut bytes: Vec<u8> = Vec::new();

        // Identifier
        bytes.extend([SPECIAL, self.identifier.len() as u8]);
        bytes.extend(self.identifier.as_bytes());

        for relay in self.relays.iter() {
            bytes.extend([RELAY, relay.len() as u8]);
            bytes.extend(relay.as_bytes());
        }

        // Author
        bytes.extend([AUTHOR, 32]);
        bytes.extend(self.pubkey.serialize());

        // Kind
        bytes.extend([KIND, 4]);
        bytes.extend(self.kind.as_u32().to_be_bytes());

        let data = bytes.to_base32();
        Ok(bech32::encode(
            PREFIX_BECH32_PARAMETERIZED_REPLACEABLE_EVENT,
            data,
            Variant::Bech32,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;

    #[test]
    fn to_bech32_public_key() {
        let public_key = XOnlyPublicKey::from_str(
            "aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4",
        )
        .unwrap();
        assert_eq!(
            "npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy".to_string(),
            public_key.to_bech32().unwrap()
        );
    }

    #[test]
    fn to_bech32_secret_key() {
        let secret_key =
            SecretKey::from_str("9571a568a42b9e05646a349c783159b906b498119390df9a5a02667155128028")
                .unwrap();
        assert_eq!(
            "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99".to_string(),
            secret_key.to_bech32().unwrap()
        );
    }

    #[test]
    fn to_bech32_note() {
        let event_id =
            EventId::from_hex("d94a3f4dd87b9a3b0bed183b32e916fa29c8020107845d1752d72697fe5309a5")
                .unwrap();
        assert_eq!(
            "note1m99r7nwc0wdrkzldrqan96gklg5usqspq7z9696j6unf0ljnpxjspqfw99".to_string(),
            event_id.to_bech32().unwrap()
        );
    }

    #[test]
    fn from_bech32_nip19_event() {
        let expected_event_id =
            EventId::from_hex("d94a3f4dd87b9a3b0bed183b32e916fa29c8020107845d1752d72697fe5309a5")
                .unwrap();

        let nip19 =
            Nip19::from_bech32("note1m99r7nwc0wdrkzldrqan96gklg5usqspq7z9696j6unf0ljnpxjspqfw99")
                .unwrap();

        assert_eq!(Nip19::EventId(expected_event_id), nip19);
    }

    #[test]
    fn from_bech32_nip19_profile() {
        let expected_pubkey = XOnlyPublicKey::from_str(
            "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245",
        )
        .unwrap();

        let nprofile = "nprofile1qqsr9cvzwc652r4m83d86ykplrnm9dg5gwdvzzn8ameanlvut35wy3gpz3mhxue69uhhyetvv9ujuerpd46hxtnfduyu75sw";
        let nip19 = Nip19::from_bech32(nprofile).unwrap();

        assert_eq!(
            Nip19::Profile(Nip19Profile::new(
                expected_pubkey,
                vec!["wss://relay.damus.io"]
            )),
            nip19
        );

        assert_eq!(nip19.to_bech32().unwrap(), nprofile);
    }

    #[test]
    fn from_bech32_nevent_author() {
        let expected_pubkey = XOnlyPublicKey::from_str(
            "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245",
        )
        .unwrap();
        let nevent = "nevent1qqsdhet4232flykq3048jzc9msmaa3hnxuesxy3lnc33vd0wt9xwk6szyqewrqnkx4zsaweutf739s0cu7et29zrntqs5elw70vlm8zudr3y24sqsgy";
        let event = Nip19Event::from_bech32(nevent).unwrap();
        assert_eq!(event.author, Some(expected_pubkey));
    }
}
