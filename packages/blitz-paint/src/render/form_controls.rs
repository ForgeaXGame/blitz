use super::ElementCx;
use crate::color::Color;
use anyrender::PaintScene;
use blitz_dom::local_name;
use kurbo::{Affine, BezPath, Cap, Circle, Join, Point, RoundedRect, Stroke, Vec2};
use peniko::Fill;
use style::dom::TElement as _;

impl ElementCx<'_, '_> {
    pub(super) fn draw_input(&self, scene: &mut impl PaintScene) {
        if let Some(select) = self.element.select_data() {
            let frame = self.frame.content_box;
            let index = self.context.dom.select_index(self.node.id).unwrap_or(-1);
            if let Some((_, layout)) = select.choices.get(index as usize) {
                let position = Affine::translate((
                    frame.x0,
                    frame.y0 + (frame.height() - f64::from(layout.height())) / 2.0,
                ));
                crate::text::stroke_text(
                    scene,
                    layout.lines(),
                    self.context.dom,
                    self.transform * position,
                    self.scale,
                    self.node.id,
                    &mut self.context.draw_text_context.borrow_mut(),
                );
            }
            let center = Point::new(frame.x1 - 7.0 * self.scale, frame.center().y);
            let mut arrow = BezPath::new();
            arrow.move_to((center.x - 3.0 * self.scale, center.y - 1.5 * self.scale));
            arrow.line_to((center.x, center.y + 1.5 * self.scale));
            arrow.line_to((center.x + 3.0 * self.scale, center.y - 1.5 * self.scale));
            scene.stroke(
                &Stroke::new(self.scale),
                self.transform,
                Color::BLACK,
                None,
                &arrow,
            );
            return;
        }
        if self.node.local_name() != "input" {
            return;
        }
        if let Some(value) = self.element.range_value() {
            let constraints = self.element.range_constraints().unwrap();
            let (vertical, reversed) = self.node.range_axis();
            let ratio = if constraints.max > constraints.min {
                (value - constraints.min) / (constraints.max - constraints.min)
            } else {
                0.0
            };
            let ratio = if reversed { 1.0 - ratio } else { ratio };
            let frame = self.frame.border_box;
            let thumb = (blitz_dom::node::RANGE_THUMB_SIZE * self.scale)
                .min(frame.width())
                .min(frame.height());
            let radius = thumb / 2.0;
            let (start, end) = if vertical {
                (
                    Point::new(frame.center().x, frame.y0 + radius),
                    Point::new(frame.center().x, frame.y1 - radius),
                )
            } else {
                (
                    Point::new(frame.x0 + radius, frame.center().y),
                    Point::new(frame.x1 - radius, frame.center().y),
                )
            };
            let center = start.lerp(end, ratio);
            let accent = if self.node.attr(local_name!("disabled")).is_some() {
                Color::from_rgba8(170, 170, 170, 255)
            } else {
                Color::from_rgba8(0, 117, 255, 255)
            };
            let track = Stroke::new(4.0 * self.scale).with_caps(Cap::Round);
            scene.stroke(
                &track,
                self.transform,
                Color::from_rgba8(210, 210, 210, 255),
                None,
                &kurbo::Line::new(start, end),
            );
            scene.stroke(
                &track,
                self.transform,
                accent,
                None,
                &kurbo::Line::new(if reversed { end } else { start }, center),
            );
            scene.fill(
                Fill::NonZero,
                self.transform,
                accent,
                None,
                &Circle::new(center, radius),
            );
            return;
        }
        let Some(checked) = self.element.checkbox_input_checked() else {
            return;
        };

        let type_attr = self.node.attr(local_name!("type"));
        let disabled = self.node.attr(local_name!("disabled")).is_some();

        let accent_color = if disabled {
            Color::from_rgba8(209, 209, 209, 255)
        } else {
            Color::from_rgba8(0, 117, 255, 255)
        };

        let width = self.frame.border_box.width();
        let height = self.frame.border_box.height();
        let min_dimension = width.min(height);
        let scale = (min_dimension - 4.0).max(0.0) / 16.0;

        let frame = self.frame.border_box.to_rounded_rect(scale * 2.0);

        match type_attr {
            Some("checkbox") => {
                draw_checkbox(scene, checked, frame, self.transform, accent_color, scale);
            }
            Some("radio") => {
                let center = frame.center();
                draw_radio_button(scene, checked, center, self.transform, accent_color, scale);
            }
            _ => {}
        }
    }
}

impl super::BlitzDomPainter<'_, '_> {
    pub(super) fn draw_validation_message(&self, scene: &mut impl PaintScene) {
        let Some((rect, layout)) = self.dom.validation_message_popup() else {
            return;
        };
        let transform = Affine::translate((
            self.initial_x + rect[0] * self.scale,
            self.initial_y + rect[1] * self.scale,
        ));
        let bounds = RoundedRect::from_rect(
            kurbo::Rect::new(0.0, 0.0, rect[2] * self.scale, rect[3] * self.scale),
            4.0 * self.scale,
        );
        scene.fill(Fill::NonZero, transform, Color::WHITE, None, &bounds);
        scene.stroke(&Stroke::new(self.scale), transform, Color::BLACK, None, &bounds);
        scene.push_clip_layer(Fill::NonZero, transform, &bounds);
        let text_transform = transform * Affine::translate((8.0 * self.scale, 8.0 * self.scale));
        for line in layout.lines() {
            for item in line.items() {
                let parley::PositionedLayoutItem::GlyphRun(glyphs) = item else {
                    continue;
                };
                let run = glyphs.run();
                scene.draw_glyphs(
                    run.font(), run.font_size(), true,
                    bytemuck::cast_slice(run.normalized_coords()), Vec2::ZERO,
                    Fill::NonZero, &anyrender::Paint::from(Color::BLACK), 1.0,
                    text_transform, None,
                    glyphs.positioned_glyphs().map(|glyph| anyrender::Glyph {
                        id: glyph.id as _, x: glyph.x, y: glyph.y,
                    }),
                );
            }
        }
        scene.pop_layer();
    }

    pub(super) fn draw_select_popups(&self, scene: &mut impl PaintScene) {
        for (id, node) in self.dom.tree().iter() {
            let Some(rect) = self.dom.select_popup_rect(id) else {
                continue;
            };
            let select = node.element_data().unwrap().select_data().unwrap();
            let transform = Affine::translate((
                self.initial_x + rect[0] * self.scale,
                self.initial_y + rect[1] * self.scale,
            ));
            let bounds = kurbo::Rect::new(0.0, 0.0, rect[2] * self.scale, rect[3] * self.scale);
            scene.fill(Fill::NonZero, transform, Color::WHITE, None, &bounds);
            scene.stroke(
                &Stroke::new(self.scale),
                transform,
                Color::from_rgba8(100, 100, 100, 255),
                None,
                &bounds,
            );
            for (index, (option, layout)) in select.choices.iter().enumerate() {
                let top = index as f64 * select.row_height * self.scale;
                if top + select.row_height * self.scale > bounds.height() {
                    break;
                }
                if select.highlighted == Some(*option) {
                    scene.fill(
                        Fill::NonZero,
                        transform,
                        Color::from_rgba8(180, 213, 255, 255),
                        None,
                        &kurbo::Rect::new(
                            0.0,
                            top,
                            bounds.width(),
                            top + select.row_height * self.scale,
                        ),
                    );
                }
                crate::text::stroke_text(
                    scene,
                    layout.lines(),
                    self.dom,
                    transform
                        * Affine::translate((
                            4.0 * self.scale,
                            top + (select.row_height * self.scale - f64::from(layout.height()))
                                / 2.0,
                        )),
                    self.scale,
                    id,
                    &mut self.draw_text_context.borrow_mut(),
                );
            }
        }
    }
}

fn draw_checkbox(
    scene: &mut impl PaintScene,
    checked: bool,
    frame: RoundedRect,
    transform: Affine,
    accent_color: Color,
    scale: f64,
) {
    if checked {
        scene.fill(Fill::NonZero, transform, accent_color, None, &frame);
        //Tick code derived from masonry
        let mut path = BezPath::new();
        path.move_to((2.0, 9.0));
        path.line_to((6.0, 13.0));
        path.line_to((14.0, 2.0));

        path.apply_affine(Affine::translate(Vec2 { x: 2.0, y: 1.0 }).then_scale(scale));
        path.apply_affine(Affine::translate((frame.rect().x0, frame.rect().y0)));

        let style = Stroke {
            width: 2.0 * scale,
            join: Join::Round,
            miter_limit: 10.0,
            start_cap: Cap::Round,
            end_cap: Cap::Round,
            dash_pattern: Default::default(),
            dash_offset: 0.0,
        };

        scene.stroke(&style, transform, Color::WHITE, None, &path);
    } else {
        scene.fill(Fill::NonZero, transform, Color::WHITE, None, &frame);
        scene.stroke(&Stroke::default(), transform, accent_color, None, &frame);
    }
}

fn draw_radio_button(
    scene: &mut impl PaintScene,
    checked: bool,
    center: Point,
    transform: Affine,
    accent_color: Color,
    scale: f64,
) {
    let outer_ring = Circle::new(center, 8.0 * scale);
    let gap = Circle::new(center, 6.0 * scale);
    let inner_circle = Circle::new(center, 4.0 * scale);
    if checked {
        scene.fill(Fill::NonZero, transform, accent_color, None, &outer_ring);
        scene.fill(Fill::NonZero, transform, Color::WHITE, None, &gap);
        scene.fill(Fill::NonZero, transform, accent_color, None, &inner_circle);
    } else {
        const GRAY: Color = color::palette::css::GRAY;
        scene.fill(Fill::NonZero, transform, GRAY, None, &outer_ring);
        scene.fill(Fill::NonZero, transform, Color::WHITE, None, &gap);
    }
}
