use aidget::transport::origin::{origin, scripts_may_fetch_http};

#[test]
fn windows_serves_the_scheme_over_an_http_host() {
    if cfg!(windows) {
        assert_eq!(origin(), "http://aidget.localhost");
        assert!(!scripts_may_fetch_http());
    } else {
        assert_eq!(origin(), "aidget://localhost");
        assert!(scripts_may_fetch_http());
    }
}
