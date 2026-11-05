use eframe::egui::{self, Pos2, Rect, Vec2};

use crate::selection::Selection;

pub const ARROW_MOVE_STEP: f32 = 2.0;

pub struct ImageMetrics {
    pub image_rect: Rect,
    pub image_size: Vec2,
    pub scale: f32,
}

impl ImageMetrics {
    pub fn new(canvas: Rect, image_size: Vec2) -> Self {
        let (display, scale) = fit_within(image_size, canvas.size());
        let offset = (canvas.size() - display) * 0.5;
        let image_rect = Rect::from_min_size(canvas.min + offset, display);
        Self {
            image_rect,
            image_size,
            scale,
        }
    }

    pub fn screen_to_image(&self, pos: Pos2) -> Pos2 {
        let rel = pos - self.image_rect.min;
        egui::pos2(
            (rel.x / self.scale).clamp(0.0, self.image_size.x),
            (rel.y / self.scale).clamp(0.0, self.image_size.y),
        )
    }

    pub fn selection_rect(&self, selection: &Selection) -> Rect {
        let min = egui::pos2(
            self.image_rect.min.x + selection.rect.min.x * self.scale,
            self.image_rect.min.y + selection.rect.min.y * self.scale,
        );
        let max = egui::pos2(
            self.image_rect.min.x + selection.rect.max.x * self.scale,
            self.image_rect.min.y + selection.rect.max.y * self.scale,
        );
        Rect::from_min_max(min, max)
    }
}

pub fn fit_within(image_size: Vec2, available: Vec2) -> (Vec2, f32) {
    let safe_size = egui::vec2(image_size.x.max(1.0), image_size.y.max(1.0));
    let scale = (available.x / safe_size.x)
        .min(available.y / safe_size.y)
        .max(0.01);
    (safe_size * scale, scale)
}

/// What Alt+arrow asks of one edge of the active selection: Alt pushes the
/// edge outward (the selection grows on that side), Shift+Alt pulls it
/// inward (it shrinks on that side). Read per frame while the key is held,
/// like the plain-arrow move.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EdgeNudge {
    #[default]
    None,
    Grow,
    Shrink,
}

impl EdgeNudge {
    /// The nudge for an arrow key that is `down` while Alt (`alt`) and
    /// Shift (`shift`) are in their current state: nothing without Alt.
    pub fn from_keys(down: bool, alt: bool, shift: bool) -> Self {
        match (down && alt, shift) {
            (false, _) => Self::None,
            (true, false) => Self::Grow,
            (true, true) => Self::Shrink,
        }
    }

    /// The edge's travel along its outward direction: `+step` grows,
    /// `-step` shrinks, `0` leaves it.
    pub fn outward(self, step: f32) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Grow => step,
            Self::Shrink => -step,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct KeyboardState {
    pub next_image: bool,
    pub prev_image: bool,
    pub save_selection: bool,
    pub delete: bool,
    pub escape: bool,
    pub move_up: bool,
    pub move_down: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub preview: bool,
    pub rotate_cw: bool,
    pub rotate_ccw: bool,
    pub full_height: bool,
    pub full_width: bool,
    pub select_all: bool,
    pub skip: bool,
    /// Alt+arrow / Shift+Alt+arrow on the active selection's edges, one per
    /// arrow key: Left and Right move the left and right sides, Up and
    /// Down the top and bottom.
    pub nudge_left: EdgeNudge,
    pub nudge_right: EdgeNudge,
    pub nudge_up: EdgeNudge,
    pub nudge_down: EdgeNudge,
}

impl KeyboardState {
    /// This frame's keys. The arrows alone move every selection; with Alt
    /// held they nudge the active selection's edges instead (`EdgeNudge`),
    /// so a plain move never fires together with a nudge.
    pub fn read(ctx: &egui::Context) -> Self {
        ctx.input(|input| {
            let alt = input.modifiers.alt;
            let shift = input.modifiers.shift;
            let nudge = |key: egui::Key| EdgeNudge::from_keys(input.key_down(key), alt, shift);
            Self {
                next_image: input.key_pressed(egui::Key::Space),
                prev_image: input.key_pressed(egui::Key::Backspace),
                save_selection: input.key_pressed(egui::Key::Enter),
                delete: input.key_pressed(egui::Key::Delete),
                escape: input.key_pressed(egui::Key::Escape),
                move_up: input.key_down(egui::Key::ArrowUp) && !alt,
                move_down: input.key_down(egui::Key::ArrowDown) && !alt,
                move_left: input.key_down(egui::Key::ArrowLeft) && !alt,
                move_right: input.key_down(egui::Key::ArrowRight) && !alt,
                preview: input.key_down(egui::Key::P),
                rotate_cw: input.key_pressed(egui::Key::R) && !shift,
                rotate_ccw: input.key_pressed(egui::Key::R) && shift,
                full_height: input.key_pressed(egui::Key::H),
                full_width: input.key_pressed(egui::Key::W),
                select_all: input.key_pressed(egui::Key::A),
                skip: input.key_pressed(egui::Key::S),
                nudge_left: nudge(egui::Key::ArrowLeft),
                nudge_right: nudge(egui::Key::ArrowRight),
                nudge_up: nudge(egui::Key::ArrowUp),
                nudge_down: nudge(egui::Key::ArrowDown),
            }
        })
    }
}
