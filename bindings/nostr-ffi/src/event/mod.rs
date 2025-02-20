// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::{Event as EventSdk, JsonUtil};
use uniffi::Object;

mod builder;
mod id;
pub mod tag;
mod unsigned;

pub use self::builder::EventBuilder;
pub use self::id::EventId;
pub use self::tag::{RelayMetadata, Tag, TagEnum, TagKind, TagKindKnown};
pub use self::unsigned::UnsignedEvent;
use crate::error::Result;
use crate::nips::nip01::Coordinate;
use crate::{PublicKey, Timestamp};

#[derive(Object)]
pub struct Event {
    inner: EventSdk,
}

impl From<EventSdk> for Event {
    fn from(inner: EventSdk) -> Self {
        Self { inner }
    }
}

impl Deref for Event {
    type Target = EventSdk;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl Event {
    pub fn id(&self) -> Arc<EventId> {
        Arc::new(self.inner.id.into())
    }

    pub fn pubkey(&self) -> Arc<PublicKey> {
        Arc::new(self.inner.pubkey.into())
    }

    pub fn created_at(&self) -> Arc<Timestamp> {
        Arc::new(self.inner.created_at.into())
    }

    pub fn kind(&self) -> u64 {
        self.inner.kind.into()
    }

    pub fn tags(&self) -> Vec<Arc<Tag>> {
        self.inner
            .tags
            .clone()
            .into_iter()
            .map(|t| Arc::new(t.into()))
            .collect()
    }

    pub fn content(&self) -> String {
        self.inner.content.clone()
    }

    pub fn signature(&self) -> String {
        self.inner.sig.to_string()
    }

    /// Verify both `EventId` and `Signature`
    pub fn verify(&self) -> bool {
        self.inner.verify().is_ok()
    }

    /// Verify if the `EventId` it's composed correctly
    pub fn verify_id(&self) -> Result<()> {
        Ok(self.inner.verify_id()?)
    }

    /// Verify only event `Signature`
    pub fn verify_signature(&self) -> Result<()> {
        Ok(self.inner.verify_signature()?)
    }

    /// Get `Timestamp` expiration if set
    pub fn expiration(&self) -> Option<Arc<Timestamp>> {
        self.inner.expiration().map(|t| Arc::new((*t).into()))
    }

    /// Returns `true` if the event has an expiration tag that is expired.
    /// If an event has no `Expiration` tag, then it will return `false`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    pub fn is_expired(&self) -> bool {
        self.inner.is_expired()
    }

    /// Check if `Kind` is a NIP90 job request
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    pub fn is_job_request(&self) -> bool {
        self.inner.is_job_request()
    }

    /// Check if `Kind` is a NIP90 job result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    pub fn is_job_result(&self) -> bool {
        self.inner.is_job_result()
    }

    /// Check if event `Kind` is `Regular`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_regular(&self) -> bool {
        self.inner.is_regular()
    }

    /// Check if event `Kind` is `Replaceable`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_replaceable(&self) -> bool {
        self.inner.is_replaceable()
    }

    /// Check if event `Kind` is `Ephemeral`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_ephemeral(&self) -> bool {
        self.inner.is_ephemeral()
    }

    /// Check if event `Kind` is `Parameterized replaceable`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_parameterized_replaceable(&self) -> bool {
        self.inner.is_parameterized_replaceable()
    }

    /// Extract identifier (`d` tag), if exists.
    pub fn identifier(&self) -> Option<String> {
        self.inner.identifier().map(|i| i.to_string())
    }

    /// Extract public keys from tags (`p` tag)
    pub fn public_keys(&self) -> Vec<Arc<PublicKey>> {
        self.inner
            .public_keys()
            .copied()
            .map(|p| Arc::new(p.into()))
            .collect()
    }

    /// Extract event IDs from tags (`e` tag)
    pub fn event_ids(&self) -> Vec<Arc<EventId>> {
        self.inner
            .event_ids()
            .copied()
            .map(|p| Arc::new(p.into()))
            .collect()
    }

    /// Extract coordinates from tags (`a` tag)
    pub fn coordinates(&self) -> Vec<Coordinate> {
        self.inner.coordinates().map(|p| p.into()).collect()
    }

    #[uniffi::constructor]
    pub fn from_json(json: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: EventSdk::from_json(json)?,
        }))
    }

    pub fn as_json(&self) -> String {
        self.inner.as_json()
    }
}
