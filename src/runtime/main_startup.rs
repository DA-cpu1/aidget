use std::sync::atomic::{AtomicBool, Ordering};

/// One-shot state shared by the main window and the background widget host.
#[derive(Default)]
pub struct MainStartup {
    page_loaded: AtomicBool,
    main_painted: AtomicBool,
    revealed: AtomicBool,
    widgets_started: AtomicBool,
    widgets_announced: AtomicBool,
}

impl MainStartup {
    /// Mark a newly created main window as waiting to be revealed.
    pub fn reset_reveal(&self) {
        self.page_loaded.store(false, Ordering::SeqCst);
        self.main_painted.store(false, Ordering::SeqCst);
        self.revealed.store(false, Ordering::SeqCst);
    }

    /// Record that the main document has finished loading.
    pub fn mark_page_loaded(&self) {
        self.page_loaded.store(true, Ordering::SeqCst);
    }

    /// Record that the main page has completed its first painted frame.
    pub fn mark_main_painted(&self) {
        self.main_painted.store(true, Ordering::SeqCst);
    }

    /// Claim the first reveal after both document and paint readiness are met.
    pub fn claim_ready_reveal(&self) -> bool {
        self.page_loaded.load(Ordering::SeqCst)
            && self.main_painted.load(Ordering::SeqCst)
            && !self.revealed.swap(true, Ordering::SeqCst)
    }

    /// Whether the main window has already been shown.
    pub fn is_revealed(&self) -> bool {
        self.revealed.load(Ordering::SeqCst)
    }

    /// Claim the single initial materialization of desktop widget windows.
    pub fn claim_widgets_start(&self) -> bool {
        !self.widgets_started.swap(true, Ordering::SeqCst)
    }

    /// Whether the initial widget materialization has been claimed.
    pub fn widgets_started(&self) -> bool {
        self.widgets_started.load(Ordering::SeqCst)
    }

    /// Release a failed initial start so the caller can surface the error and
    /// allow a later attempt.
    pub fn reset_widgets_start(&self) {
        self.widgets_started.store(false, Ordering::SeqCst);
    }

    /// Claim the one event that tells the main page widgets are available.
    pub fn claim_widgets_announcement(&self) -> bool {
        !self.widgets_announced.swap(true, Ordering::SeqCst)
    }
}
