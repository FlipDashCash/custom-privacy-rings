fn main() {
    println!("cargo:rerun-if-env-changed=PYROL_RING_DELIVERY_V1_KEYS_RS");
    println!("cargo:rerun-if-env-changed=PYROL_INSECURE_LOCAL_DELIVERY");
    if std::env::var_os("CARGO_FEATURE_EXPERIMENTAL_DELIVERY_LOCALNET").is_some() {
        assert_eq!(
            std::env::var("PYROL_INSECURE_LOCAL_DELIVERY").as_deref(),
            Ok("1"),
            "delivery Localnet setup requires explicit opt-in"
        );
        let source = std::env::var("PYROL_RING_DELIVERY_V1_KEYS_RS")
            .expect("missing distinct delivery-v1 keys");
        assert!(
            std::path::Path::new(&source).is_absolute() && std::path::Path::new(&source).is_file(),
            "delivery key module must be an absolute existing path"
        );
        println!("cargo:rerun-if-changed={source}");
    }
}
