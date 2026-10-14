//! Adversarial Stress Test Suite for Task J002b (Spring-Loaded Z Zoom).
//! Tests timing boundary conditions (299ms vs 300ms vs 301ms),
//! dragging while Z held (<300ms), and modifier keys (Ctrl+Z, Shift+Z, etc.).
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use egui::{Key, Modifiers};
use serde_json::json;
use vectorcraft_ui_egui::VectorcraftApp;
use vectorcraft_ui_egui::shortcuts::handle;

fn setup_test_app() -> (VectorcraftApp, egui::Context) {
    let mut session = vectorcraft_engine::Session::new();
    let _ = session.execute("file.new", &json!({"width": 800, "height": 600}));
    let app = VectorcraftApp::new(session, Default::default());
    let ctx = egui::Context::default();
    (app, ctx)
}

fn run_frame(ctx: &egui::Context, app: &mut VectorcraftApp, time: f64, events: Vec<egui::Event>) {
    let mut out = ctx.run_ui(egui::RawInput { time: Some(time), events, ..Default::default() }, |ui| {
        handle(app, ui.ctx());
        app.logic(ui.ctx());
    });
    out.textures_delta.clear();
}

fn z_down_with(modifiers: Modifiers) -> egui::Event {
    egui::Event::Key { key: Key::Z, physical_key: None, pressed: true, repeat: false, modifiers }
}

fn z_up_with(modifiers: Modifiers) -> egui::Event {
    egui::Event::Key { key: Key::Z, physical_key: None, pressed: false, repeat: false, modifiers }
}

/// 1. Quick tap at 299ms (< 300ms): tool remains "zoom" permanently.
#[test]
fn test_timing_boundary_299ms_quick_tap() {
    let (mut app, ctx) = setup_test_app();
    app.select_tool("pen");
    assert_eq!(app.session.tool_id(), "pen");

    run_frame(&ctx, &mut app, 1.000, vec![z_down_with(Modifiers::NONE)]);
    assert_eq!(app.session.tool_id(), "zoom", "Z down must enter zoom");
    assert!(app.spring_zoom.is_some());

    // Release at 1.299s (held for 299ms < 300ms)
    run_frame(&ctx, &mut app, 1.299, vec![z_up_with(Modifiers::NONE)]);
    assert_eq!(app.session.tool_id(), "zoom", "299ms hold (< 300ms) is a tap and must remain in zoom tool");
    assert!(app.spring_zoom.is_none(), "Spring zoom state must be cleared after tap release");
}

/// 2. Hold at 301ms (> 300ms): spring-loaded zoom reverts to previous tool on release.
#[test]
fn test_timing_boundary_301ms_spring_loaded() {
    let (mut app, ctx) = setup_test_app();
    app.select_tool("pen");
    assert_eq!(app.session.tool_id(), "pen");

    run_frame(&ctx, &mut app, 3.000, vec![z_down_with(Modifiers::NONE)]);
    assert_eq!(app.session.tool_id(), "zoom");
    assert!(app.spring_zoom.is_some());

    // Release at 3.301s (held for 301ms > 300ms)
    run_frame(&ctx, &mut app, 3.301, vec![z_up_with(Modifiers::NONE)]);
    assert_eq!(app.session.tool_id(), "pen", "301ms hold (> 300ms) must revert to prior tool ('pen')");
    assert!(app.spring_zoom.is_none());
}

/// 3. Timing boundary at exactly 300ms:
/// Demonstrates floating-point cancellation defect in `now - start_time >= 0.300`:
/// In IEEE 754 f64, `2.3 - 2.0 = 0.2999999999999998` which is `< 0.300`.
/// Consequently, exact 300ms hold is mistakenly classified as a quick tap instead of spring-loaded!
#[test]
fn test_timing_boundary_exact_300ms_float_issue() {
    let (mut app, ctx) = setup_test_app();
    app.select_tool("pen");
    assert_eq!(app.session.tool_id(), "pen");

    run_frame(&ctx, &mut app, 2.000, vec![z_down_with(Modifiers::NONE)]);
    assert_eq!(app.session.tool_id(), "zoom");
    assert!(app.spring_zoom.is_some());

    // Release at exactly 2.300s (mathematically exactly 300ms hold)
    run_frame(&ctx, &mut app, 2.300, vec![z_up_with(Modifiers::NONE)]);

    let held_calc = 2.3f64 - 2.0f64;
    // Prove that IEEE 754 evaluates 2.3 - 2.0 strictly less than 0.300
    assert!(held_calc < 0.300, "Floating-point representation 2.3 - 2.0 is 0.2999999999999998 < 0.300");

    // With epsilon tolerance, exact 300ms hold reverts to "pen":
    assert_eq!(app.session.tool_id(), "pen", "With epsilon tolerance, exact 300ms hold must revert to prior tool ('pen')");
    assert!(app.spring_zoom.is_none(), "Spring zoom state must be cleared after reversion");
}

/// 4. Dragging while Z is held: verify drag marks spring-loaded even if total duration < 300ms.
#[test]
fn test_drag_while_z_held_sub_300ms() {
    let (mut app, ctx) = setup_test_app();

    // Case A: Held for only 50ms, but dragged
    app.select_tool("rectangle");
    assert_eq!(app.session.tool_id(), "rectangle");
    run_frame(&ctx, &mut app, 1.000, vec![z_down_with(Modifiers::NONE)]);
    assert_eq!(app.session.tool_id(), "zoom");

    // Simulate drag occurring during the hold
    app.spring_zoom.as_mut().unwrap().dragged = true;

    // Release at 1.050s (held 50ms < 300ms)
    run_frame(&ctx, &mut app, 1.050, vec![z_up_with(Modifiers::NONE)]);
    assert_eq!(app.session.tool_id(), "rectangle", "Dragging while Z is held must revert to prior tool even if held < 300ms");
    assert!(app.spring_zoom.is_none());

    // Case B: Held for 299ms with drag
    app.select_tool("ellipse");
    assert_eq!(app.session.tool_id(), "ellipse");
    run_frame(&ctx, &mut app, 2.000, vec![z_down_with(Modifiers::NONE)]);
    assert_eq!(app.session.tool_id(), "zoom");

    app.spring_zoom.as_mut().unwrap().dragged = true;

    // Release at 2.299s (held 299ms < 300ms)
    run_frame(&ctx, &mut app, 2.299, vec![z_up_with(Modifiers::NONE)]);
    assert_eq!(app.session.tool_id(), "ellipse", "Dragging with 299ms hold must revert to prior tool");
    assert!(app.spring_zoom.is_none());
}

/// 5. Modifiers: Ctrl+Z, Cmd+Z, Alt+Z, Ctrl+Shift+Z must NOT trigger spring zoom.
#[test]
fn test_ctrl_cmd_alt_modifiers_do_not_hijack_spring_zoom() {
    // 1. Ctrl+Z (Undo)
    {
        let (mut app, ctx) = setup_test_app();
        app.select_tool("pen");
        run_frame(&ctx, &mut app, 1.0, vec![z_down_with(Modifiers::CTRL)]);
        assert!(app.spring_zoom.is_none(), "Ctrl+Z must not trigger spring zoom");
        assert_eq!(app.session.tool_id(), "pen", "Ctrl+Z must not switch tool to zoom");
    }

    // 2. Cmd+Z (Undo)
    {
        let (mut app, ctx) = setup_test_app();
        app.select_tool("pen");
        run_frame(&ctx, &mut app, 1.0, vec![z_down_with(Modifiers::COMMAND)]);
        assert!(app.spring_zoom.is_none(), "Cmd+Z must not trigger spring zoom");
        assert_eq!(app.session.tool_id(), "pen", "Cmd+Z must not switch tool to zoom");
    }

    // 3. Alt+Z
    {
        let (mut app, ctx) = setup_test_app();
        app.select_tool("pen");
        run_frame(&ctx, &mut app, 1.0, vec![z_down_with(Modifiers::ALT)]);
        assert!(app.spring_zoom.is_none(), "Alt+Z must not trigger spring zoom");
        assert_eq!(app.session.tool_id(), "pen", "Alt+Z must not switch tool to zoom");
    }

    // 4. Ctrl+Shift+Z (Redo)
    {
        let (mut app, ctx) = setup_test_app();
        app.select_tool("pen");
        run_frame(&ctx, &mut app, 1.0, vec![z_down_with(Modifiers::CTRL | Modifiers::SHIFT)]);
        assert!(app.spring_zoom.is_none(), "Ctrl+Shift+Z must not trigger spring zoom");
        assert_eq!(app.session.tool_id(), "pen", "Ctrl+Shift+Z must not switch tool to zoom");
    }
}

/// 6. Shift+Z Hijacking Bug Demonstration:
/// Trowel specification §11:
/// - "Shift를 먼저 누르고 Z를 누르면 표고 도구입니다. 돋보기는 Z만 누릅니다."
/// - Shift+Z is Spot Elevation (표고 도구).
/// In `crates/ui-egui/src/shortcuts.rs`:
/// `matches!(e, egui::Event::Key { key: Key::Z, pressed: true, repeat: false, modifiers, .. } if !(modifiers.command || modifiers.ctrl || modifiers.alt))`
/// It omits `!modifiers.shift`, allowing `Shift+Z` to trigger `z_pressed` and hijack into spring zoom!
#[test]
fn test_shift_z_hijacks_spring_zoom_bug() {
    let (mut app, ctx) = setup_test_app();
    app.select_tool("pen");
    assert_eq!(app.session.tool_id(), "pen");

    // User presses Shift+Z intending Spot Elevation (§11)
    run_frame(&ctx, &mut app, 1.0, vec![z_down_with(Modifiers::SHIFT)]);

    // With !modifiers.shift guard, Shift+Z does NOT trigger spring zoom:
    assert_eq!(app.session.tool_id(), "pen", "Shift+Z must NOT switch tool to 'zoom', leaving it for spot elevation");
    assert!(app.spring_zoom.is_none(), "Shift+Z must NOT activate spring_zoom state");
}
