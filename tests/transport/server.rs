use aidget::transport::server::protocol::{embedded_cache_control, widget_csp, widget_csp_for};

#[test]
fn static_svg_documents_receive_the_script_ban_too() {
    use aidget::transport::server::protocol::serve_widget;
    use aidget::widgets::store::{Store, WidgetEdit};
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path());
    store.ensure_layout().unwrap();
    store.create("note", "Note", WidgetEdit::default()).unwrap();
    std::fs::write(
        store.widget_dir("note").unwrap().join("assets/evil.svg"),
        r#"<svg xmlns="http://www.w3.org/2000/svg"><script>window.executed=true</script></svg>"#,
    )
    .unwrap();
    let response = serve_widget(&store, "note/assets/evil.svg");
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "image/svg+xml");
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert!(response.headers()["content-security-policy"]
        .to_str()
        .unwrap()
        .contains("script-src 'none'"));
}

#[test]
fn embedded_html_revalidates_while_hashed_assets_are_immutable() {
    assert_eq!(embedded_cache_control("main.html"), "no-cache");
    assert_eq!(embedded_cache_control("widget.html"), "no-cache");
    assert_eq!(
        embedded_cache_control("assets/main-abc123.js"),
        "public, max-age=31536000, immutable"
    );
}

#[test]
fn script_csp_allows_widget_assets_but_not_cross_widget_reads() {
    let csp = widget_csp(true);
    assert!(csp.contains("'unsafe-eval'"));
    let connect = csp
        .split("connect-src ")
        .nth(1)
        .expect("connect-src directive");
    assert!(!connect.contains("aidget:"));
    assert!(connect.contains("https:"));
}

#[test]
fn script_csp_matches_both_origin_models() {
    let custom_scheme = widget_csp_for(true, true);
    assert!(custom_scheme.contains("default-src aidget:"));
    assert!(custom_scheme.contains("connect-src https: http:"));

    let shared_http_origin = widget_csp_for(true, false);
    assert!(shared_http_origin.contains("default-src 'self'"));
    assert!(shared_http_origin.contains("connect-src https:"));
    assert!(!shared_http_origin.contains("http:"));
}

#[test]
fn plain_csp_still_blocks_scripts() {
    let csp = widget_csp(false);
    for directive in [
        "script-src 'none'",
        "object-src 'none'",
        "frame-src 'none'",
        "worker-src 'none'",
    ] {
        assert!(csp.contains(directive), "missing {directive}");
    }
}
