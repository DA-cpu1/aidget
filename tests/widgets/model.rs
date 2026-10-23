use aidget::widgets::model::WidgetFrame;

#[test]
fn drag_mode_defaults_to_all_and_rejects_unknown_values() {
    use aidget::widgets::model::WidgetMeta;
    use serde_json::json;

    let mut value = serde_json::to_value(WidgetMeta::new("note", "Note", false)).unwrap();
    assert_eq!(value["dragMode"], "all");
    value.as_object_mut().unwrap().remove("dragMode");
    let meta: WidgetMeta = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(meta).unwrap()["dragMode"], "all");
    value["dragMode"] = json!("handle");
    let meta: WidgetMeta = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(meta).unwrap()["dragMode"], "handle");
    value["dragMode"] = json!("disabled");
    assert!(serde_json::from_value::<WidgetMeta>(value).is_err());
}

#[test]
fn clamps_frames_into_the_work_area() {
    let frame = WidgetFrame {
        display: "primary".to_string(),
        x: 5000.0,
        y: -80.0,
        w: 900.0,
        h: 100.0,
    };
    let clamped = frame.clamped((1440.0, 900.0, 1440.0, 900.0));
    assert_eq!(clamped.x, 540.0);
    assert_eq!(clamped.y, 0.0);
    assert_eq!(clamped.w, 900.0);
    assert_eq!(clamped.h, 120.0);
}

#[test]
fn default_frames_do_not_overlap() {
    let a = WidgetFrame::default_for(0);
    let b = WidgetFrame::default_for(1);
    assert!(a.x + a.w <= b.x);
}

#[test]
fn fitting_a_frame_keeps_cross_display_offsets() {
    let frame = WidgetFrame {
        display: "Monitor #2".to_string(),
        x: -120.0,
        y: -300.0,
        w: 900.0,
        h: 100.0,
    };
    let fitted = frame.fitted((1512.0, 893.0, 1512.0, 893.0));
    assert_eq!(fitted.x, -120.0);
    assert_eq!(fitted.y, -300.0);
    assert_eq!(fitted.w, 900.0);
    assert_eq!(fitted.h, 120.0);
}
