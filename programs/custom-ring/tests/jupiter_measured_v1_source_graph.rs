//! One owned-graph host regression; no transaction, signer, validator, or proof.
#![forbid(unsafe_code)]

use custom_ring_interface::{storage, TransferRoots};

#[path = "fixtures/jupiter_measured_v1_graph_case.rs"]
mod original_case;

#[test]
fn checks_one_stock_operation_then_consumes_its_own_binding_in_the_v1_context() {
    original_case::with_archived_graph(|_, _, _, _, _| {});
}
