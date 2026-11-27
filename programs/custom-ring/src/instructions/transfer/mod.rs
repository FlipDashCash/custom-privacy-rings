//! `Transfer` and `AuthorityTransfer`.
//!
//! Read after `lib.rs`. Read before `instructions/transfer/processor.rs`.

pub(super) mod accounts;
mod approve_transfer;
mod processor;
pub(super) mod verify;

pub(crate) use approve_transfer::process_approve_transfer_ix;
pub(crate) use processor::process_transfer_ix;
