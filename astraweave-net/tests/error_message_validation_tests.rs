fn assert_message_sane(msg: &str) {
    assert!(
        msg.len() >= 10,
        "error message too short / not descriptive: {msg:?}"
    );
    assert!(
        msg.chars().any(|c| c.is_ascii_alphabetic()),
        "error message should contain some words: {msg:?}"
    );

    for forbidden in ["C:\\Users\\", "\\Users\\", "/home/", "/Users/"] {
        assert!(
            !msg.contains(forbidden),
            "error message leaks local path ({forbidden}): {msg:?}"
        );
    }
}

#[cfg(feature = "tls")]
#[test]
fn tls_missing_cert_error_is_descriptive_and_no_path_leak() {
    use astraweave_net::tls::TlsServerConfig;

    // Use an absolute path (typical source of local path leaks).
    let abs = std::env::current_dir()
        .expect("current_dir")
        .join("definitely_missing_cert.pem");

    let result = TlsServerConfig::from_pem_files(&abs, "also_missing_key.pem");
    let err = match result {
        Ok(_) => panic!("expected error for missing cert file"),
        Err(e) => e,
    };
    let msg = err.to_string();

    assert!(msg.contains("Failed to open cert file"));
    assert!(msg.contains("definitely_missing_cert.pem"));
    assert_message_sane(&msg);
}
