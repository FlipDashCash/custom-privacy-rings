//! Host-only admission for the opt-in local relay adapters.
use std::net::IpAddr;

// Raw 32-byte getGenesisHash values, not abbreviated CAIP cluster identifiers.
const PUBLIC_GENESIS: [[u8; 32]; 3] = [
    // 5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d (mainnet-beta)
    [
        0x45, 0x29, 0x69, 0x98, 0xa6, 0xf8, 0xe2, 0xa7, 0x84, 0xdb, 0x5d, 0x9f, 0x95, 0xe1, 0x8f,
        0xc2, 0x3f, 0x70, 0x44, 0x1a, 0x10, 0x39, 0x44, 0x68, 0x01, 0x08, 0x98, 0x79, 0xb0, 0x8c,
        0x7e, 0xf0,
    ],
    // EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG (devnet)
    [
        0xce, 0x59, 0xdb, 0x50, 0x80, 0xfc, 0x2c, 0x6d, 0x3b, 0xcf, 0x7c, 0xa9, 0x07, 0x12, 0xd3,
        0xc2, 0xe5, 0xe6, 0xc2, 0x8f, 0x27, 0xf0, 0xdf, 0xbb, 0x99, 0x53, 0xbd, 0xb0, 0x89, 0x4c,
        0x03, 0xab,
    ],
    // 4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY (testnet)
    [
        0x3a, 0x13, 0x2e, 0xce, 0x10, 0x30, 0x5e, 0xc1, 0x83, 0x07, 0x25, 0x50, 0x2f, 0xa2, 0xb7,
        0xe7, 0xeb, 0x81, 0x57, 0xe9, 0x12, 0x3d, 0x4c, 0x1f, 0x65, 0x4a, 0x71, 0x78, 0x71, 0x61,
        0xdc, 0x21,
    ],
];

/// Literal, explicit non-default port only. This is a small allowed grammar,
/// not a generic URL parser. It never resolves a domain or approves a proxy.
pub(crate) fn rpc_origin(rpc: &str) -> Option<&str> {
    let origin = rpc.strip_suffix('/').unwrap_or(rpc);
    let port = origin.strip_prefix("http://127.0.0.1:")?;
    if port.is_empty()
        || port.len() > 5
        || port.starts_with('0')
        || !port.bytes().all(|byte| byte.is_ascii_digit())
        || port == "80"
    {
        return None;
    }
    port.parse::<u16>().ok().map(|_| origin)
}

pub(crate) fn origin(bind: IpAddr, rpc: &str, genesis_hash: [u8; 32]) -> Option<&str> {
    if !bind.is_loopback() || genesis_hash == [0; 32] || PUBLIC_GENESIS.contains(&genesis_hash) {
        return None;
    }
    rpc_origin(rpc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_loopback_origins_are_canonicalized_without_resolving_any_url() {
        let local = IpAddr::from([127, 0, 0, 1]);
        let rpc = "http://127.0.0.1:8899";
        assert_eq!(origin(local, rpc, [7; 32]), Some(rpc));
        assert_eq!(origin(local, "http://127.0.0.1:8899/", [7; 32]), Some(rpc));
        assert_eq!(
            rpc_origin("http://127.0.0.1:65535"),
            Some("http://127.0.0.1:65535")
        );
    }

    #[test]
    fn public_bind_and_known_public_or_unset_genesis_are_refused() {
        let local = IpAddr::from([127, 0, 0, 1]);
        let rpc = "http://127.0.0.1:8899";
        assert_eq!(origin(IpAddr::from([0; 4]), rpc, [7; 32]), None);
        assert_eq!(origin(local, rpc, [0; 32]), None);
        for genesis in PUBLIC_GENESIS {
            assert_eq!(origin(local, rpc, genesis), None);
        }
    }

    #[test]
    fn no_implicit_default_port_alias_path_credentials_query_or_fragment() {
        let local = IpAddr::from([127, 0, 0, 1]);
        for nonlocal in [
            "https://127.0.0.1:8899",
            "http://127.0.0.1",
            "http://127.0.0.1:80",
            "http://127.0.0.1:08899",
            "http://127.0.0.1:0",
            "http://127.0.0.1:65536",
            "http://127.0.0.2:8899",
            "http://localhost:8899",
            "http://2130706433:8899",
            "http://operator@127.0.0.1:8899",
            "http://127.0.0.1:8899/status",
            "http://127.0.0.1:8899//",
            "http://127.0.0.1:8899/?q=x",
            "http://127.0.0.1:8899/#fragment",
            "http://127.0.0.1:８８９９",
        ] {
            assert_eq!(origin(local, nonlocal, [7; 32]), None);
        }
    }
}
