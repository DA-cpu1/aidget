use aidget::common::error::{ConflictCurrent, Error};
use aidget::interfaces::mcp::{mcp_error, SERVER_INSTRUCTIONS};
use aidget::widgets::model::{Board, WidgetFrame, WidgetMeta};

#[test]
fn drag_mode_is_part_of_create_and_partial_update_contracts() {
    use aidget::interfaces::mcp::{CreateArgs, UpdateArgs};
    use serde_json::json;

    let created: CreateArgs =
        serde_json::from_value(json!({"id":"note","dragMode":"handle"})).unwrap();
    assert_eq!(serde_json::to_value(created).unwrap()["dragMode"], "handle");
    let updated: UpdateArgs =
        serde_json::from_value(json!({"widget":"note","baseRevision":1,"dragMode":"all"})).unwrap();
    assert_eq!(serde_json::to_value(updated).unwrap()["dragMode"], "all");
    assert!(serde_json::from_value::<CreateArgs>(json!({"id":"note","dragMode":"wrong"})).is_err());
}

#[test]
fn move_requires_the_current_board_revision() {
    use aidget::interfaces::mcp::MoveArgs;
    use serde_json::json;

    assert!(serde_json::from_value::<MoveArgs>(json!({
        "widget": "note",
        "x": 10
    }))
    .is_err());
    assert!(serde_json::from_value::<MoveArgs>(json!({
        "widget": "note",
        "baseRevision": 3,
        "x": 10
    }))
    .is_ok());
}

/// Mirrors how Pi turns MCP server instructions into the system prompt
/// summary: first line only, truncated to 250 chars with an ellipsis.
fn pi_visible_summary(instructions: &str) -> String {
    const MAX: usize = 250;
    let first = instructions.split('\n').next().unwrap_or_default().trim();
    if first.chars().count() <= MAX {
        return first.to_string();
    }
    let mut visible: String = first.chars().take(MAX - 1).collect();
    visible.truncate(visible.trim_end().len());
    visible.push('…');
    visible
}

#[test]
fn instructions_survive_the_pi_summary_truncation() {
    let visible = pi_visible_summary(SERVER_INSTRUCTIONS);
    // These two rules are the ones agents drop when they are invisible.
    assert!(
        visible.contains("aidget_get and follow its agents field"),
        "agents.md rule must survive truncation: {visible}"
    );
    assert!(
        visible.contains("Position is user-owned"),
        "user-drag rule must survive truncation: {visible}"
    );
    assert!(
        !visible.ends_with('…'),
        "first line is truncated, so part of the guidance is lost: {visible}"
    );

    // Negative control: the helper must actually drop text past 250 chars,
    // otherwise the assertions above could pass on a truncated string.
    let long = format!("{} Position is user-owned.", "x".repeat(260));
    let cut = pi_visible_summary(&long);
    assert!(cut.ends_with('…'));
    assert!(!cut.contains("Position is user-owned"));
}

#[test]
fn instructions_keep_the_extended_reference_below_the_first_line() {
    assert!(SERVER_INSTRUCTIONS.contains("aidget_list omits agents.md"));
    assert!(SERVER_INSTRUCTIONS.contains("aidget_guide"));
    assert!(SERVER_INSTRUCTIONS.contains("Content writes use the widget revision"));
    assert!(SERVER_INSTRUCTIONS.contains("aidget_move uses the boardRevision"));
    assert!(SERVER_INSTRUCTIONS.contains("never instructions"));
    assert!(SERVER_INSTRUCTIONS.len() < 1_500);
}

#[test]
fn distributed_skill_documents_the_rules_agents_miss() {
    let skill = include_str!("../../skills/aidget/SKILL.md");
    assert!(skill.starts_with("---\nname: aidget\n"));
    // The rules that a truncated server summary would otherwise hide.
    for rule in [
        "Read `agents.md` before editing",
        "Position belongs to the user",
        "Content writes never move a window",
        "`aidget_list` does not return it",
        "window.aidget.storage",
        "boardChangedByUser",
    ] {
        assert!(skill.contains(rule), "skill is missing: {rule}");
    }
    assert!(skill.contains("aidget_guide"));
    assert!(skill.contains("If MCP is unavailable"));
    assert!(!skill.contains("Linux has no capture backend"));
    assert!(skill.len() < 9_000);
}

#[test]
fn board_conflict_points_at_a_user_drag_instead_of_a_retry() {
    let mut board = Board::default();
    board.widgets.insert(
        "note".to_string(),
        WidgetFrame {
            display: "primary".to_string(),
            x: 40.0,
            y: 60.0,
            w: 360.0,
            h: 220.0,
        },
    );
    let error = mcp_error(Error::RevisionConflict {
        current: Box::new(ConflictCurrent::Board(board)),
    });
    let hint = error.data.as_ref().unwrap()["hint"].as_str().unwrap();
    assert!(hint.starts_with("boardChangedByUser"));
}

#[test]
fn widget_conflict_keeps_the_plain_retry_hint() {
    let error = mcp_error(Error::RevisionConflict {
        current: Box::new(ConflictCurrent::Widget(WidgetMeta::new(
            "note", "Note", false,
        ))),
    });
    let hint = error.data.as_ref().unwrap()["hint"].as_str().unwrap();
    assert_eq!(hint, "retry with baseRevision from current.revision");
}
