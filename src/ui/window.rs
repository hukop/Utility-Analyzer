//! Window resize handling for frameless windows.
//!
//! This module provides window resize support for borderless/frameless windows
//! by detecting mouse position near window edges and initiating OS-level resize
//! operations via egui's ViewportCommand system.
//!
//! ## Usage
//!
//! Call `handle_window_resize` at the start of each frame, before rendering UI:
//!
//! ```ignore
//! fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//!     handle_window_resize(ctx, &mut self.resize_state);
//!     // ... rest of UI
//!     self.resize_state.apply_cursor(ctx); // Call at end to override UI cursors
//! }
//! ```

use eframe::egui::{self, CursorIcon, Pos2, Rect, ResizeDirection, ViewportCommand};

/// Width of the resize border in logical pixels.
const RESIZE_BORDER_WIDTH: f32 = 5.0;

/// Corner grab area size (slightly larger than edge for easier corner detection).
const CORNER_GRAB_SIZE: f32 = 10.0;

/// Height of the title bar area to exclude from north edge resize detection.
/// This prevents cursor flicker conflicts with window control buttons.
const TITLE_BAR_HEIGHT: f32 = 32.0;

/// Width of the button area on the right side of the title bar.
/// This area contains window control buttons (close, maximize, minimize).
const TITLE_BAR_BUTTON_AREA_WIDTH: f32 = 120.0;

/// State for tracking window resize operations.
#[derive(Debug, Clone, Default)]
pub struct WindowResizeState {
    /// Currently detected resize direction (None if not hovering edge).
    current_direction: Option<ResizeDirection>,
    /// Whether we're actively in a resize operation.
    is_resizing: bool,
    /// Cursor to apply at end of frame (to override UI element cursors).
    pending_cursor: Option<CursorIcon>,
}

impl WindowResizeState {
    /// Create a new resize state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if currently resizing.
    #[allow(dead_code)]
    pub fn is_resizing(&self) -> bool {
        self.is_resizing
    }

    /// Apply the pending resize cursor if one was set.
    /// Call this at the END of your update() function to ensure resize cursor
    /// takes priority over UI element cursors.
    pub fn apply_cursor(&mut self, ctx: &egui::Context) {
        if let Some(cursor) = self.pending_cursor.take() {
            ctx.set_cursor_icon(cursor);
        }
    }
}

/// Handle window resize for borderless windows.
///
/// This function should be called at the start of each frame, before rendering
/// any UI elements. It:
///
/// 1. Detects if the mouse is hovering over a resize edge/corner
/// 2. Changes the cursor icon to indicate resize capability
/// 3. Initiates resize operation when mouse is pressed
///
/// Returns `true` if a resize operation was initiated.
pub fn handle_window_resize(ctx: &egui::Context, state: &mut WindowResizeState) -> bool {
    // Don't handle resize if window is maximized
    let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    if is_maximized {
        state.current_direction = None;
        state.is_resizing = false;
        state.pending_cursor = None;
        return false;
    }

    // Get pointer position and mouse state
    let (pointer_pos, primary_pressed, primary_down) = ctx.input(|i| {
        let pos = i.pointer.hover_pos();
        let pressed = i.pointer.primary_pressed();
        let down = i.pointer.primary_down();
        (pos, pressed, down)
    });

    let window_rect = ctx.screen_rect();

    let Some(pointer_pos) = pointer_pos else {
        if !primary_down {
            state.current_direction = None;
            state.is_resizing = false;
            state.pending_cursor = None;
        }
        return false;
    };

    // If we're in a resize operation, continue until mouse is released
    if state.is_resizing {
        if !primary_down {
            state.is_resizing = false;
            state.current_direction = None;
            state.pending_cursor = None;
        }
        return true;
    }

    // Detect resize direction based on pointer position
    let direction = detect_resize_direction(window_rect, pointer_pos);

    // Update state
    state.current_direction = direction;

    // Set cursor based on direction
    if let Some(dir) = direction {
        let desired_cursor = direction_to_cursor(dir);
        state.pending_cursor = Some(desired_cursor);

        // Initiate resize on mouse press
        if primary_pressed {
            ctx.send_viewport_cmd(ViewportCommand::BeginResize(dir));
            state.is_resizing = true;
            return true;
        }
    } else {
        state.pending_cursor = None;
    }

    false
}

/// Detect which resize direction (if any) the pointer is in.
fn detect_resize_direction(window_rect: Rect, pointer_pos: Pos2) -> Option<ResizeDirection> {
    let min = window_rect.min;
    let max = window_rect.max;

    // Check if pointer is in the title bar exclusion zone
    let in_title_bar = pointer_pos.y < min.y + TITLE_BAR_HEIGHT;
    let in_button_area = pointer_pos.x > max.x - TITLE_BAR_BUTTON_AREA_WIDTH;
    let disable_north_resize = in_title_bar && in_button_area;

    // Check if pointer is near each edge
    let near_left = pointer_pos.x < min.x + RESIZE_BORDER_WIDTH;
    let near_right = pointer_pos.x > max.x - RESIZE_BORDER_WIDTH;
    let near_top = pointer_pos.y < min.y + RESIZE_BORDER_WIDTH;
    let near_bottom = pointer_pos.y > max.y - RESIZE_BORDER_WIDTH;

    // Check if pointer is in corner zones
    let in_left_zone = pointer_pos.x < min.x + CORNER_GRAB_SIZE;
    let in_right_zone = pointer_pos.x > max.x - CORNER_GRAB_SIZE;
    let in_top_zone = pointer_pos.y < min.y + CORNER_GRAB_SIZE;
    let in_bottom_zone = pointer_pos.y > max.y - CORNER_GRAB_SIZE;

    // Corners take priority
    // NorthWest corner
    if (near_left || in_left_zone)
        && (near_top || in_top_zone)
        && pointer_pos.x < min.x + CORNER_GRAB_SIZE
        && pointer_pos.y < min.y + CORNER_GRAB_SIZE
        && !disable_north_resize
    {
        return Some(ResizeDirection::NorthWest);
    }

    // NorthEast corner - disabled in title bar
    if (near_right || in_right_zone)
        && (near_top || in_top_zone)
        && pointer_pos.x > max.x - CORNER_GRAB_SIZE
        && pointer_pos.y < min.y + CORNER_GRAB_SIZE
        && !in_title_bar
    {
        return Some(ResizeDirection::NorthEast);
    }

    // SouthWest corner
    if (near_left || in_left_zone)
        && (near_bottom || in_bottom_zone)
        && pointer_pos.x < min.x + CORNER_GRAB_SIZE
        && pointer_pos.y > max.y - CORNER_GRAB_SIZE
    {
        return Some(ResizeDirection::SouthWest);
    }

    // SouthEast corner
    if (near_right || in_right_zone)
        && (near_bottom || in_bottom_zone)
        && pointer_pos.x > max.x - CORNER_GRAB_SIZE
        && pointer_pos.y > max.y - CORNER_GRAB_SIZE
    {
        return Some(ResizeDirection::SouthEast);
    }

    // Check for corner zones to exclude from edge detection
    let in_northwest_corner = in_left_zone && in_top_zone;
    let in_northeast_corner = in_right_zone && in_top_zone;
    let in_southwest_corner = in_left_zone && in_bottom_zone;
    let in_southeast_corner = in_right_zone && in_bottom_zone;

    // Edge detection - exclude corner zones
    if near_left && !in_northwest_corner && !in_southwest_corner {
        return Some(ResizeDirection::West);
    }
    if near_right && !in_northeast_corner && !in_southeast_corner {
        return Some(ResizeDirection::East);
    }
    if near_top && !in_left_zone && !in_right_zone && !disable_north_resize {
        return Some(ResizeDirection::North);
    }
    if near_bottom && !in_left_zone && !in_right_zone {
        return Some(ResizeDirection::South);
    }

    None
}

/// Convert a resize direction to the appropriate cursor icon.
fn direction_to_cursor(direction: ResizeDirection) -> CursorIcon {
    match direction {
        ResizeDirection::North => CursorIcon::ResizeNorth,
        ResizeDirection::South => CursorIcon::ResizeSouth,
        ResizeDirection::East => CursorIcon::ResizeEast,
        ResizeDirection::West => CursorIcon::ResizeWest,
        ResizeDirection::NorthEast => CursorIcon::ResizeNorthEast,
        ResizeDirection::NorthWest => CursorIcon::ResizeNorthWest,
        ResizeDirection::SouthEast => CursorIcon::ResizeSouthEast,
        ResizeDirection::SouthWest => CursorIcon::ResizeSouthWest,
    }
}

/// Height of the custom title bar.
pub const CUSTOM_TITLE_BAR_HEIGHT: f32 = 32.0;


/// Render a custom title bar as a TopBottomPanel.
pub fn render_title_bar(ctx: &egui::Context, title: &str) -> bool {
    let mut drag_initiated = false;

    egui::TopBottomPanel::top("title_bar")
        .exact_height(CUSTOM_TITLE_BAR_HEIGHT)
        .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(8.0);

                // Title - make most of the bar draggable
                let title_rect = ui.available_rect_before_wrap();
                let title_response = ui.allocate_rect(
                    egui::Rect::from_min_size(
                        title_rect.min,
                        egui::vec2(title_rect.width() - 120.0, CUSTOM_TITLE_BAR_HEIGHT),
                    ),
                    egui::Sense::click_and_drag(),
                );

                // Draw title text
                ui.painter().text(
                    egui::pos2(title_rect.min.x + 4.0, title_rect.center().y + 5.0),
                    egui::Align2::LEFT_CENTER,
                    title,
                    egui::FontId::proportional(14.0),
                    ui.visuals().text_color(),
                );

                // Handle window drag
                if title_response.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    drag_initiated = true;
                }

                // Handle double-click to maximize/restore
                if title_response.double_clicked() {
                    let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                }

                // Window control buttons on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Remove side space so close button can tuck into the rounded corner
                    // ui.add_space(4.0);

                    // Close button
                    let close_btn = ui.add(
                        egui::Button::new(egui::RichText::new("X").size(12.0).strong())
                            .frame(false)
                            .min_size(egui::vec2(32.0, 28.0))
                    );
                    if close_btn.clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if close_btn.hovered() {
                        let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                        let window_rounding = if is_maximized { 2.0 } else { crate::ui::styles::WINDOW_ROUNDING };

                        ui.painter().rect_filled(
                            close_btn.rect,
                            egui::Rounding {
                                nw: 2.0,
                                ne: window_rounding,
                                sw: 2.0,
                                se: 2.0,
                            },
                            egui::Color32::from_rgb(196, 43, 28),
                        );
                    }

                    // Maximize/Restore button
                    let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                    let max_icon = if is_maximized { "❐" } else { "□" };
                    let max_btn = ui.add(
                        egui::Button::new(egui::RichText::new(max_icon).size(12.0))
                            .frame(false)
                            .min_size(egui::vec2(32.0, 28.0))
                    );
                    if max_btn.clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                    }

                    // Minimize button
                    let min_btn = ui.add(
                        egui::Button::new(egui::RichText::new("—").size(12.0))
                            .frame(false)
                            .min_size(egui::vec2(32.0, 28.0))
                    );
                    if min_btn.clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                });
            });
        });

    drag_initiated
}
