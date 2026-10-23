use aidget::runtime::main_startup::MainStartup;

#[test]
fn main_window_reveal_is_a_one_shot_gate() {
    let startup = MainStartup::default();

    assert!(!startup.is_revealed());
    startup.mark_page_loaded();
    assert!(!startup.claim_ready_reveal());
    startup.mark_main_painted();
    assert!(startup.claim_ready_reveal());
    assert!(startup.is_revealed());
    assert!(!startup.claim_ready_reveal());

    startup.reset_reveal();
    startup.mark_main_painted();
    assert!(!startup.claim_ready_reveal());
    startup.mark_page_loaded();
    assert!(startup.claim_ready_reveal());
}

#[test]
fn initial_widget_start_and_announcement_are_one_shot() {
    let startup = MainStartup::default();

    assert!(!startup.widgets_started());
    assert!(startup.claim_widgets_start());
    assert!(startup.widgets_started());
    assert!(!startup.claim_widgets_start());

    assert!(startup.claim_widgets_announcement());
    assert!(!startup.claim_widgets_announcement());
}
