use aidget::transport::ipc::pipe_name;
use std::path::Path;

#[test]
fn pipe_names_are_stable_and_root_specific() {
    let a = pipe_name(Path::new("/home/me/.aidget/run/ui.sock"));
    assert_eq!(a, pipe_name(Path::new("/home/me/.aidget/run/ui.sock")));
    assert_ne!(a, pipe_name(Path::new("/tmp/other/.aidget/run/ui.sock")));
}
