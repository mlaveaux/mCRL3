use std::sync::Arc;

use cosmic_text::Metrics;
use femtovg::Canvas;
use femtovg::Color;
use femtovg::Paint;
use femtovg::Path;
use femtovg::Renderer;
use glam::Vec2;
use glam::Vec3Swizzles;

use mcrl3_lts::LabelledTransitionSystem;

use crate::Viewer;
use crate::text_cache::TextCache;

pub struct FemtovgRenderer {
    /// A cache used to cache strings and font information.
    text_cache: TextCache,

    /// A buffer for transition labels.
    labels_cache: Vec<cosmic_text::Buffer>,

    /// Reference to the LTS being rendered
    lts: Arc<LabelledTransitionSystem>,
}

impl FemtovgRenderer {
    pub fn new(lts: Arc<LabelledTransitionSystem>) -> Self {
        let mut text_cache = TextCache::new();
        let mut labels_cache = vec![];

        for label in lts.labels() {
            // Create text elements for all labels that we are going to render.
            let buffer = text_cache.create_buffer(label, Metrics::new(12.0, 12.0));

            // Put it in the label cache.
            labels_cache.push(buffer);
        }

        Self {
            text_cache,
            labels_cache,
            lts,
        }
    }

    /// Render the current state of the simulation into the canvas.
    pub fn render<T: Renderer>(
        &mut self,
        canvas: &mut Canvas<T>,
        viewer: &Viewer,
        draw_actions: bool,
        state_radius: f32,
        view_x: f32,
        view_y: f32,
        screen_x: u32,
        screen_y: u32,
        zoom_level: f32,
        label_text_size: f32,
    ) {
        // Clear the canvas with white color
        canvas.clear_rect(0, 0, screen_x, screen_y, Color::white());

        // Save current transformation state
        canvas.save();

        // Compute the view transform
        canvas.translate(screen_x as f32 / 2.0 + view_x, screen_y as f32 / 2.0 + view_y);
        canvas.scale(zoom_level, zoom_level);

        // The color information for states
        let state_inner_paint = Paint::color(Color::white());
        let initial_state_paint = Paint::color(Color::rgb(100, 255, 100));
        let state_outer = Paint::color(Color::black());

        // The color information for edges
        let mut edge_paint = Paint::color(Color::black());
        edge_paint.set_line_width(1.0);

        // Resize the labels if necessary
        for buffer in &mut self.labels_cache {
            self.text_cache
                .resize(buffer, Metrics::new(label_text_size, label_text_size));
        }

        let mut path = Path::new();

        // Draw the edges and the arrows on them
        for state_index in self.lts.iter_states() {
            let state_view = &viewer.state_view()[state_index];

            // For now we only draw 2D graphs properly
            debug_assert!(state_view.position.z.abs() < 0.01);

            for (transition_index, (label, to)) in self.lts.outgoing_transitions(state_index).enumerate() {
                let to_state_view = &viewer.state_view()[*to];
                let transition_view = &state_view.outgoing[transition_index];

                let label_position = if *to != state_index {
                    // Draw the transition line
                    let mut path = Path::new();
                    path.move_to(state_view.position.x, state_view.position.y);
                    path.line_to(to_state_view.position.x, to_state_view.position.y);
                    canvas.stroke_path(&path, &edge_paint);

                    let direction = (state_view.position - to_state_view.position).normalize();
                    let angle = -1.0 * direction.xy().angle_to(Vec2::new(0.0, -1.0)).to_degrees();

                    // Draw the arrow of the transition
                    canvas.save();
                    canvas.translate(to_state_view.position.x, to_state_view.position.y);
                    canvas.rotate(angle);
                    canvas.translate(0.0, -state_radius - 0.5);

                    let mut arrow_path = Path::new();
                    arrow_path.move_to(0.0, 0.0);
                    arrow_path.line_to(2.0, -5.0);
                    arrow_path.line_to(-2.0, -5.0);
                    arrow_path.close();

                    canvas.fill_path(&arrow_path, &edge_paint);
                    canvas.restore();

                    // Draw the edge handle
                    let middle = (to_state_view.position + state_view.position) / 2.0;
                    let handle_x = middle.x + transition_view.handle_offset.x;
                    let handle_y = middle.y + transition_view.handle_offset.y;

                    path.circle(handle_x, handle_y, 1.0);
                    //canvas.fill(&edge_paint);

                    middle
                } else {
                    // This is a self loop
                    let middle = (2.0 * state_view.position + transition_view.handle_offset) / 2.0;
                    let radius = transition_view.handle_offset.length() / 2.0;

                    path.circle(middle.x, middle.y, radius);
                    //path.stroke(&edge_paint);

                    // Draw the edge handle
                    let handle_x = state_view.position.x + transition_view.handle_offset.x;
                    let handle_y = state_view.position.y + transition_view.handle_offset.y;

                    path.circle(handle_x, handle_y, 1.0);
                    //canvas.fill(&edge_paint);

                    state_view.position + transition_view.handle_offset
                };

                // Draw the text label
                if draw_actions {
                    // Save the current canvas state before drawing text
                    canvas.save();

                    // Reset the transform for text rendering
                    canvas.reset_transform();

                    // Calculate the transformed position for text
                    let transformed_x = label_position.x * zoom_level + screen_x as f32 / 2.0 + view_x;
                    let transformed_y = label_position.y * zoom_level + screen_y as f32 / 2.0 + view_y;

                    let buffer = &self.labels_cache[*label];
                    //self.text_cache.draw(buffer, canvas, transformed_x, transformed_y);

                    // Restore the canvas state
                    canvas.restore();
                }
            }
        }

        // Draw the states on top
        for (index, state_view) in viewer.state_view().iter().enumerate() {
            if index != self.lts.initial_state_index() {
                // Regular state
                //canvas.circle(state_view.position.x, state_view.position.y, state_radius);
                //canvas.fill(&state_inner_paint);
                //canvas.stroke(&state_outer);
            } else {
                // Initial state
                //canvas.circle(state_view.position.x, state_view.position.y, state_radius);
                //canvas.fill(&initial_state_paint);
                //canvas.stroke(&state_outer);
            }
        }

        // Restore original transformation
        canvas.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use femtovg::renderer::Void;

    use mcrl3_lts::read_aut;

    #[test]
    fn test_femtovg_viewer() {
        let file = include_str!("../../../../../examples/lts/abp.aut");
        let lts = Arc::new(read_aut(file.as_bytes(), vec![]).unwrap());

        let mut canvas = Canvas::new(Void).unwrap();
        let viewer = Viewer::new(lts.clone());
        let mut renderer = FemtovgRenderer::new(lts);

        renderer.render(&mut canvas, &viewer, true, 10.0, 0.0, 0.0, 800, 600, 1.0, 12.0);
    }
}
