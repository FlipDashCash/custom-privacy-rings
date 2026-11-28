//! `Transfer` and `AuthorityTransfer`.
//!
//! Read after `lib.rs`. Read before `instructions/transfer/processor.rs`.

mod accounts;
mod actual_output;
mod approve_transfer;
mod processor;
mod verify;

pub(crate) use actual_output::process_actual_output_ix;
pub(crate) use approve_transfer::process_approve_transfer_ix;
pub(crate) use processor::process_transfer_ix;
