//! Ring addresses, the audit stream cipher and the ring keys of a shielded keypair.

mod dummy_key;
mod error;
mod keys;
mod list_salt;
pub mod ring_address;
mod stream_cipher;

pub use dummy_key::{dummy_recipient_key, DUMMY_KEY_DOMAIN};
pub use error::RingKeyError;
pub use keys::{
    RingDeliveryRequest, RingKeys, RingTagFrame, RingTagKind, RingTagProbe, RotatingRingKeys,
    ViewTagRequest, ViewTagStream,
};
pub use list_salt::{ListSaltHolder, LIST_SALT_LABEL};
pub use ring_address::{RingAddress, RingPaymentRequest};
pub use stream_cipher::{symmetric_apply_audit, RingStream};
