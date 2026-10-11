//! User-agent constraint-validation feedback. The DOM facade supplies the
//! current standard validation message; this projection owns no validity rules.
use crate::{BaseDocument, NodeId, node::TextBrush};

pub(crate) struct ValidationMessage {
    node: NodeId,
    text: String,
    layout: parley::Layout<TextBrush>,
}

impl BaseDocument {
    pub fn show_validation_message(&mut self, node: NodeId, text: &str) {
        self.validation_message = (!text.is_empty()).then(|| ValidationMessage {
            node,
            text: text.to_owned(),
            layout: Default::default(),
        });
        self.prepare_validation_message();
        self.shell_provider.request_redraw();
    }

    pub fn dismiss_validation_message(&mut self) {
        if self.validation_message.take().is_some() {
            self.shell_provider.request_redraw();
        }
    }

    pub(crate) fn prepare_validation_message(&mut self) {
        let Some(mut message) = self.validation_message.take() else {
            return;
        };
        let Some(node) = self.get_node(message.node) else {
            return;
        };
        if self.focus_node_id != Some(message.node)
            || !node.has_boxes()
            || !self.can_focus(message.node)
        {
            return;
        }
        let Some(style) = node.primary_styles() else {
            return;
        };
        let authored = crate::stylo_to_parley::style(message.node, &style);
        drop(style);
        let text_style = parley::TextStyle {
            font_family: authored.font_family,
            locale: authored.locale,
            font_size: 13.0,
            brush: TextBrush::from_id(message.node),
            ..Default::default()
        };
        let scale = self.viewport.scale();
        let width = (self.viewport.window_size.0 as f32 / scale - 16.0).clamp(1.0, 304.0);
        let mut fonts = self.font_ctx.lock().unwrap();
        let mut builder = self
            .layout_ctx
            .tree_builder(&mut fonts, scale, true, &text_style);
        builder.push_text(&message.text);
        message.layout = builder.build().0;
        message.layout.break_all_lines(Some(width * scale));
        self.validation_message = Some(message);
    }

    /// A viewport layer, independent of author overflow clips and CSS effects.
    pub fn validation_message_popup(&self) -> Option<([f64; 4], &parley::Layout<TextBrush>)> {
        let message = self.validation_message.as_ref()?;
        let anchor = self.get_client_bounding_rect(message.node)?;
        let scale = self.viewport.scale_f64();
        let viewport_width = self.viewport.window_size.0 as f64 / scale;
        let viewport_height = self.viewport.window_size.1 as f64 / scale;
        let width = (message.layout.width() as f64 / scale + 16.0).min(viewport_width);
        let height = (message.layout.height() as f64 / scale + 16.0).min(viewport_height);
        let x = anchor.x.min(viewport_width - width).max(0.0);
        let y = if anchor.y + anchor.height + height <= viewport_height {
            anchor.y + anchor.height
        } else {
            (anchor.y - height).max(0.0)
        };
        Some(([x, y, width, height], &message.layout))
    }
}
