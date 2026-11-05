use eframe::egui;
use imagecropper::app::canvas::Canvas;
use imagecropper::selection::Selection;
use imagecropper::ui::{EdgeNudge, ImageMetrics, KeyboardState, ARROW_MOVE_STEP};

fn selection_from_coords(min: (f32, f32), max: (f32, f32)) -> Selection {
    Selection {
        rect: egui::Rect::from_min_max(egui::pos2(min.0, min.1), egui::pos2(max.0, max.1)),
    }
}

#[test]
fn handle_arrow_movement_translates_selection() {
    let mut canvas = Canvas::new();
    canvas
        .selections
        .push(selection_from_coords((10.0, 10.0), (20.0, 20.0)));
    let keys = KeyboardState {
        move_right: true,
        ..Default::default()
    };
    canvas.handle_arrow_movement(&keys, egui::vec2(100.0, 100.0));
    let selection = &canvas.selections[0];
    assert_eq!(selection.rect.min.x, 10.0 + ARROW_MOVE_STEP);
    assert_eq!(selection.rect.max.x, 20.0 + ARROW_MOVE_STEP);
}

#[test]
fn begin_selection_outside_image_clamps_to_image_border() {
    let mut canvas = Canvas::new();
    let canvas_rect = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(200.0, 200.0));
    let metrics = ImageMetrics::new(canvas_rect, egui::vec2(100.0, 100.0));

    canvas.begin_selection(
        egui::pos2(-40.0, 80.0),
        &metrics,
        egui::vec2(100.0, 100.0),
        false,
    );

    let selection = canvas.selections.last().unwrap();
    assert_eq!(selection.rect.min.x, 0.0);
    assert_eq!(selection.rect.max.x, 1.0);
}

/// A headless egui pass with `down` held and `modifiers` active, as the
/// window would report one frame of a held key.
fn ctx_with_keys(down: &[egui::Key], modifiers: egui::Modifiers) -> egui::Context {
    let ctx = egui::Context::default();
    let mut input = egui::RawInput::default();
    input.events.push(egui::Event::ModifiersChanged(modifiers));
    for &key in down {
        input.events.push(egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        });
    }
    ctx.begin_pass(input);
    ctx
}

#[test]
fn plain_arrows_move_and_alt_arrows_nudge_never_both() {
    let keys = KeyboardState::read(&ctx_with_keys(&[egui::Key::ArrowLeft], egui::Modifiers::NONE));
    assert!(keys.move_left);
    assert_eq!(keys.nudge_left, EdgeNudge::None);

    let keys = KeyboardState::read(&ctx_with_keys(&[egui::Key::ArrowLeft], egui::Modifiers::ALT));
    assert!(!keys.move_left);
    assert_eq!(keys.nudge_left, EdgeNudge::Grow);
    assert_eq!(keys.nudge_right, EdgeNudge::None);

    let keys = KeyboardState::read(&ctx_with_keys(
        &[egui::Key::ArrowDown],
        egui::Modifiers::ALT | egui::Modifiers::SHIFT,
    ));
    assert!(!keys.move_down);
    assert_eq!(keys.nudge_down, EdgeNudge::Shrink);

    // Shift alone still moves, as before.
    let keys = KeyboardState::read(&ctx_with_keys(&[egui::Key::ArrowUp], egui::Modifiers::SHIFT));
    assert!(keys.move_up);
    assert_eq!(keys.nudge_up, EdgeNudge::None);
}

#[test]
fn edge_nudge_grows_one_side_and_leaves_the_others() {
    let image = egui::vec2(100.0, 100.0);
    let mut canvas = Canvas::new();
    canvas
        .selections
        .push(selection_from_coords((40.0, 40.0), (60.0, 60.0)));

    let left = KeyboardState { nudge_left: EdgeNudge::Grow, ..Default::default() };
    canvas.handle_edge_nudge(&left, image);
    let r = canvas.selections[0].rect;
    assert_eq!((r.min.x, r.min.y, r.max.x, r.max.y), (40.0 - ARROW_MOVE_STEP, 40.0, 60.0, 60.0));

    let right = KeyboardState { nudge_right: EdgeNudge::Grow, ..Default::default() };
    canvas.handle_edge_nudge(&right, image);
    assert_eq!(canvas.selections[0].rect.max.x, 60.0 + ARROW_MOVE_STEP);

    let up = KeyboardState { nudge_up: EdgeNudge::Grow, ..Default::default() };
    canvas.handle_edge_nudge(&up, image);
    assert_eq!(canvas.selections[0].rect.min.y, 40.0 - ARROW_MOVE_STEP);

    let down = KeyboardState { nudge_down: EdgeNudge::Grow, ..Default::default() };
    canvas.handle_edge_nudge(&down, image);
    let r = canvas.selections[0].rect;
    assert_eq!(r.max.y, 60.0 + ARROW_MOVE_STEP);
    assert_eq!(r.min.x, 40.0 - ARROW_MOVE_STEP);
    assert_eq!(r.max.x, 60.0 + ARROW_MOVE_STEP);
}

#[test]
fn edge_nudge_shrinks_one_side() {
    let image = egui::vec2(100.0, 100.0);
    let mut canvas = Canvas::new();
    canvas
        .selections
        .push(selection_from_coords((40.0, 40.0), (60.0, 60.0)));

    let keys = KeyboardState {
        nudge_left: EdgeNudge::Shrink,
        nudge_down: EdgeNudge::Shrink,
        ..Default::default()
    };
    canvas.handle_edge_nudge(&keys, image);
    let r = canvas.selections[0].rect;
    assert_eq!((r.min.x, r.min.y, r.max.x, r.max.y), (40.0 + ARROW_MOVE_STEP, 40.0, 60.0, 60.0 - ARROW_MOVE_STEP));
}

#[test]
fn edge_nudge_stops_at_the_image_border_and_at_one_pixel() {
    let image = egui::vec2(100.0, 100.0);
    let mut canvas = Canvas::new();
    canvas
        .selections
        .push(selection_from_coords((1.0, 1.0), (3.0, 99.0)));

    // Growing past the border clamps to it.
    let grow = KeyboardState {
        nudge_left: EdgeNudge::Grow,
        nudge_down: EdgeNudge::Grow,
        ..Default::default()
    };
    canvas.handle_edge_nudge(&grow, image);
    let r = canvas.selections[0].rect;
    assert_eq!(r.min.x, 0.0);
    assert_eq!(r.max.y, 100.0);

    // Shrinking a 3px-wide selection from the right leaves at least 1px.
    let shrink = KeyboardState { nudge_right: EdgeNudge::Shrink, ..Default::default() };
    for _ in 0..10 {
        canvas.handle_edge_nudge(&shrink, image);
    }
    let r = canvas.selections[0].rect;
    assert!(r.width() >= 1.0, "width {}", r.width());
    assert_eq!(r.min.x, 0.0);
}

#[test]
fn edge_nudge_touches_only_the_active_selection() {
    let image = egui::vec2(100.0, 100.0);
    let mut canvas = Canvas::new();
    canvas
        .selections
        .push(selection_from_coords((10.0, 10.0), (20.0, 20.0)));
    canvas
        .selections
        .push(selection_from_coords((50.0, 50.0), (70.0, 70.0)));

    let keys = KeyboardState { nudge_right: EdgeNudge::Grow, ..Default::default() };
    canvas.handle_edge_nudge(&keys, image);
    assert_eq!(canvas.selections[0].rect.max.x, 20.0);
    assert_eq!(canvas.selections[1].rect.max.x, 70.0 + ARROW_MOVE_STEP);
}

#[test]
fn edge_nudge_without_selection_is_a_no_op() {
    let mut canvas = Canvas::new();
    let keys = KeyboardState { nudge_left: EdgeNudge::Grow, ..Default::default() };
    canvas.handle_edge_nudge(&keys, egui::vec2(100.0, 100.0));
    assert!(canvas.selections.is_empty());
}

#[test]
fn select_entire_image_covers_the_image_and_keeps_existing_selections() {
    let image = egui::vec2(640.0, 480.0);
    let mut canvas = Canvas::new();
    canvas.select_entire_image(image);
    assert_eq!(canvas.selections.len(), 1);
    let r = canvas.selections[0].rect;
    assert_eq!((r.min.x, r.min.y, r.max.x, r.max.y), (0.0, 0.0, 640.0, 480.0));
    assert_eq!(canvas.selections[0].to_u32_bounds(), Some((0, 0, 640, 480)));

    // Like Ctrl-drag, it adds beside what is there (the A key's behaviour).
    canvas.select_entire_image(image);
    assert_eq!(canvas.selections.len(), 2);

    // The start of a drag on a whole-image selection: pulling the right
    // edge in by 100px clips that side only.
    let after = canvas.selections[1]
        .clone()
        .adjusted(imagecropper::selection::SelectionHandle::Right, egui::vec2(-100.0, 0.0), image);
    assert_eq!(after.rect.max.x, 540.0);
    assert_eq!(after.rect.min, egui::pos2(0.0, 0.0));
    assert_eq!(after.rect.max.y, 480.0);
}
