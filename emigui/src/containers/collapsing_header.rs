use crate::{layout::Direction, widgets::Label, *};

#[derive(Clone, Copy, Debug, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(default)]
pub(crate) struct State {
    open: bool,

    #[serde(skip)] // Times are relative, and we don't want to continue animations anyway
    toggle_time: f64,

    /// Height of the region when open. Used for animations
    open_height: Option<f32>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            open: false,
            toggle_time: -f64::INFINITY,
            open_height: None,
        }
    }
}

pub struct CollapsingHeader {
    label: Label,
    default_open: bool,
}

impl CollapsingHeader {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: Label::new(label)
                .text_style(TextStyle::Button)
                .multiline(false),
            default_open: false,
        }
    }

    pub fn default_open(mut self) -> Self {
        self.default_open = true;
        self
    }
}

impl CollapsingHeader {
    pub fn show(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) -> GuiResponse {
        assert!(
            ui.direction() == Direction::Vertical,
            "Horizontal collapsing is unimplemented"
        );
        let Self {
            label,
            default_open,
        } = self;

        // TODO: horizontal layout, with icon and text as labels. Inser background behind using Frame.

        let title = &label.text; // TODO: not this
        let id = ui.make_unique_id(title);
        let text_pos = ui.cursor() + vec2(ui.style().indent, 0.0);
        let (title, text_size) = label.layout(text_pos, ui);
        let text_max_x = text_pos.x + text_size.x;
        let desired_width = ui.available_width().max(text_max_x - ui.cursor().x);

        let interact = ui.reserve_space(
            vec2(
                desired_width,
                text_size.y + 2.0 * ui.style().button_padding.y,
            ),
            Some(id),
        );
        let text_pos = pos2(text_pos.x, interact.rect.center().y - text_size.y / 2.0);

        let mut state = {
            let mut memory = ui.memory();
            let mut state = memory.collapsing_headers.entry(id).or_insert(State {
                open: default_open,
                ..Default::default()
            });
            if interact.clicked {
                state.open = !state.open;
                state.toggle_time = ui.input().time;
            }
            *state
        };

        let where_to_put_background = ui.paint_list_len();

        paint_icon(ui, &state, &interact);

        ui.add_text(
            text_pos,
            label.text_style,
            title,
            Some(ui.style().interact_stroke_color(&interact)),
        );

        ui.insert_paint_cmd(
            where_to_put_background,
            PaintCmd::Rect {
                corner_radius: ui.style().interact_corner_radius(&interact),
                fill_color: ui.style().interact_fill_color(&interact),
                outline: ui.style().interact_outline(&interact),
                rect: interact.rect,
            },
        );

        ui.expand_to_include_child(interact.rect); // TODO: remove, just a test

        let animation_time = ui.style().animation_time;
        let time_since_toggle = (ui.input().time - state.toggle_time) as f32;
        let time_since_toggle = time_since_toggle + ui.input().dt; // Instant feedback
        let animate = time_since_toggle < animation_time;
        if animate {
            ui.indent(id, |child_ui| {
                let max_height = if state.open {
                    let full_height = state.open_height.unwrap_or(1000.0);
                    remap(time_since_toggle, 0.0..=animation_time, 0.0..=full_height)
                } else {
                    let full_height = state.open_height.unwrap_or_default();
                    remap_clamp(time_since_toggle, 0.0..=animation_time, full_height..=0.0)
                };

                let mut clip_rect = child_ui.clip_rect();
                clip_rect.max.y = clip_rect.max.y.min(child_ui.cursor().y + max_height);
                child_ui.set_clip_rect(clip_rect);

                let top_left = child_ui.top_left();
                add_contents(child_ui);

                state.open_height = Some(child_ui.bounding_size().y);

                // Pretend children took up less space:
                let mut child_bounds = child_ui.child_bounds();
                child_bounds.max.y = child_bounds.max.y.min(top_left.y + max_height);
                child_ui.force_set_child_bounds(child_bounds);
            });
        } else if state.open {
            let full_size = ui.indent(id, add_contents).rect.size();
            state.open_height = Some(full_size.y);
        }

        ui.memory().collapsing_headers.insert(id, state);
        ui.response(interact)
    }
}

fn paint_icon(ui: &mut Ui, state: &State, interact: &InteractInfo) {
    let stroke_color = ui.style().interact_stroke_color(interact);
    let stroke_width = ui.style().interact_stroke_width(interact);

    let (mut small_icon_rect, _) = ui.style().icon_rectangles(interact.rect);
    small_icon_rect.set_center(pos2(
        interact.rect.left() + ui.style().indent / 2.0,
        interact.rect.center().y,
    ));

    // Draw a minus:
    ui.add_paint_cmd(PaintCmd::Line {
        points: vec![
            pos2(small_icon_rect.left(), small_icon_rect.center().y),
            pos2(small_icon_rect.right(), small_icon_rect.center().y),
        ],
        color: stroke_color,
        width: stroke_width,
    });

    if !state.open {
        // Draw it as a plus:
        ui.add_paint_cmd(PaintCmd::Line {
            points: vec![
                pos2(small_icon_rect.center().x, small_icon_rect.top()),
                pos2(small_icon_rect.center().x, small_icon_rect.bottom()),
            ],
            color: stroke_color,
            width: stroke_width,
        });
    }
}