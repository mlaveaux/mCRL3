use std::sync::Arc;

use cosmic_text::Metrics;
use glam::Mat3;
use glam::Vec2;
use glam::Vec3;
use glam::Vec3Swizzles;
use tiny_skia::Shader;
use tiny_skia::Stroke;
use tiny_skia::Transform;

use mcrl3_lts::LabelledTransitionSystem;

use crate::graph_layout::GraphLayout;
use crate::text_cache::TextCache;

pub struct Viewer {
    /// A cache used to cache strings and font information.
    text_cache: TextCache,

    /// A buffer for transition labels.
    labels_cache: Vec<cosmic_text::Buffer>,

    /// The underlying LTS being displayed.
    lts: Arc<LabelledTransitionSystem>,

    /// Stores a local copy of the state positions.
    view_states: Vec<StateView>,
}

#[derive(Clone, Default)]
struct StateView {
    pub position: Vec3,
    pub outgoing: Vec<TransitionView>,
}

#[derive(Clone, Default)]
pub struct TransitionView {
    /// The offset of the handle w.r.t. the 'from' state.
    pub handle_offset: Vec3,
}

impl Viewer {
    pub fn new(lts: &Arc<LabelledTransitionSystem>) -> Viewer {
        let mut text_cache = TextCache::new();
        let mut labels_cache = vec![];

        for label in lts.labels() {
            // Create text elements for all labels that we are going to render.
            let buffer = text_cache.create_buffer(label, Metrics::new(12.0, 12.0));

            // Put it in the label cache.
            labels_cache.push(buffer);
        }

        // Initialize the view information for the states.
        let mut view_states = vec![StateView::default(); lts.num_of_states()];

        // Add the transition view information
        for (state_index, state_view) in view_states.iter_mut().enumerate() {
            state_view.outgoing = vec![TransitionView::default(); lts.outgoing_transitions(state_index).count()];

            // Compute the offsets for self-loops, put them at equal distance around the state.
            let num_selfloops = lts
                .outgoing_transitions(state_index)
                .filter(|(_, to)| *to == state_index)
                .count();

            // Keep track of the current self loop index.
            let mut index_selfloop = 0;

            // Keep track of the current transition index.
            let mut index_transition = 0;

            for (transition_index, (_, to)) in lts.outgoing_transitions(state_index).enumerate() {
                let transition_view = &mut state_view.outgoing[transition_index];

                if state_index == *to {
                    // This is a self loop so compute a rotation around the state for its handle.
                    let rotation_mat = Mat3::from_euler(
                        glam::EulerRot::XYZ,
                        0.0,
                        0.0,
                        (index_selfloop as f32 / num_selfloops as f32) * 2.0 * std::f32::consts::PI,
                    );
                    transition_view.handle_offset = rotation_mat.mul_vec3(Vec3::new(0.0, -40.0, 0.0));

                    index_selfloop += 1;
                } else {
                    // Determine whether any of the outgoing edges from the reached state point back.
                    let has_backtransition = lts
                        .outgoing_transitions(*to)
                        .filter(|(_, other_to)| *other_to == state_index)
                        .count()
                        > 0;

                    // Compute the number of transitions going to the same state.
                    let num_transitions = lts
                        .outgoing_transitions(state_index)
                        .filter(|(_, to)| *to == state_index)
                        .count();

                    if has_backtransition {
                        // Offset the outgoing transitions towards that state to the right.
                        transition_view.handle_offset =
                            Vec3::new(0.0, index_transition as f32 / num_transitions as f32, 0.0);
                    } else {
                        // Balance transitions around the midpoint.
                    }

                    index_transition += 1;
                }
            }
        }

        Viewer {
            text_cache,
            labels_cache,
            lts: lts.clone(),
            view_states,
        }
    }

    /// Update the state of the viewer with the given graph layout.
    pub fn update(&mut self, layout: &GraphLayout) {
        for (index, layout_state) in self.view_states.iter_mut().enumerate() {
            layout_state.position = layout.layout_states[index].position;
        }
    }

    /// Returns the center of the graph.
    pub fn center(&self) -> Vec3 {
        self.view_states.iter().map(|x| x.position).sum::<Vec3>() / self.view_states.len() as f32
    }

    /// Render the current state of the simulation into the pixmap.
    pub fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        draw_actions: bool,
        state_radius: f32,
        view_x: f32,
        view_y: f32,
        screen_x: u32,
        screen_y: u32,
        zoom_level: f32,
        label_text_size: f32,
    ) {
        pixmap.fill(tiny_skia::Color::WHITE);

        // Compute the view transform
        let view_transform = Transform::from_translate(view_x, view_y)
            .post_scale(zoom_level, zoom_level)
            .post_translate(screen_x as f32 / 2.0, screen_y as f32 / 2.0);

        // The color information for states.
        let state_inner_paint = tiny_skia::Paint {
            shader: Shader::SolidColor(tiny_skia::Color::from_rgba8(255, 255, 255, 255)),
            ..Default::default()
        };
        let initial_state_paint = tiny_skia::Paint {
            shader: Shader::SolidColor(tiny_skia::Color::from_rgba8(100, 255, 100, 255)),
            ..Default::default()
        };
        let state_outer = tiny_skia::Paint {
            shader: Shader::SolidColor(tiny_skia::Color::from_rgba8(0, 0, 0, 255)),
            ..Default::default()
        };

        // The color information for edges
        let edge_paint = tiny_skia::Paint::default();

        // The arrow to indicate the direction of the edge, this unwrap should never fail.
        let arrow = {
            let mut builder = tiny_skia::PathBuilder::new();
            builder.line_to(2.0, -5.0);
            builder.line_to(-2.0, -5.0);
            builder.close();
            builder.finish().unwrap()
        };

        // A single circle that is used to render colored states.
        let circle = {
            let mut builder = tiny_skia::PathBuilder::new();
            builder.push_circle(0.0, 0.0, state_radius);
            builder.finish().unwrap()
        };

        // Resize the labels if necessary.
        for buffer in &mut self.labels_cache {
            self.text_cache
                .resize(buffer, Metrics::new(label_text_size, label_text_size));
        }

        // Draw the edges and the arrows on them
        let mut edge_builder = tiny_skia::PathBuilder::new();
        let mut arrow_builder = tiny_skia::PathBuilder::new();

        for state_index in self.lts.iter_states() {
            let state_view = &self.view_states[state_index];

            // For now we only draw 2D graphs properly.
            debug_assert!(state_view.position.z.abs() < 0.01);

            for (transition_index, (label, to)) in self.lts.outgoing_transitions(state_index).enumerate() {
                let to_state_view = &self.view_states[*to];
                let transition_view = &state_view.outgoing[transition_index];

                let label_position = if *to != state_index {
                    // Draw the transition
                    edge_builder.move_to(state_view.position.x, state_view.position.y);
                    edge_builder.line_to(to_state_view.position.x, to_state_view.position.y);

                    let direction = (state_view.position - to_state_view.position).normalize();
                    let angle = -1.0 * direction.xy().angle_to(Vec2::new(0.0, -1.0)).to_degrees();

                    // Draw the arrow of the transition
                    if let Some(path) = arrow.clone().transform(
                        Transform::from_translate(0.0, -state_radius - 0.5)
                            .post_rotate(angle)
                            .post_translate(to_state_view.position.x, to_state_view.position.y),
                    ) {
                        arrow_builder.push_path(&path);
                    };

                    // Draw the edge handle
                    let middle = (to_state_view.position + state_view.position) / 2.0;
                    edge_builder.push_circle(
                        middle.x + transition_view.handle_offset.x,
                        middle.y + transition_view.handle_offset.y,
                        1.0,
                    );

                    middle
                } else {
                    // This is a self loop so draw a circle around the middle of the position and the handle.
                    let middle = (2.0 * state_view.position + transition_view.handle_offset) / 2.0;
                    edge_builder.push_circle(middle.x, middle.y, transition_view.handle_offset.length() / 2.0);

                    // Draw the edge handle
                    edge_builder.push_circle(
                        state_view.position.x + transition_view.handle_offset.x,
                        state_view.position.y + transition_view.handle_offset.y,
                        1.0,
                    );
                    state_view.position + transition_view.handle_offset
                };

                // Draw the text label
                if draw_actions {
                    let buffer = &self.labels_cache[*label];
                    self.text_cache.draw(
                        buffer,
                        pixmap,
                        Transform::from_translate(label_position.x, label_position.y).post_concat(view_transform),
                    );
                }
            }
        }

        if let Some(path) = arrow_builder.finish() {
            pixmap.fill_path(&path, &edge_paint, tiny_skia::FillRule::Winding, view_transform, None);
        }

        // Draw the path for edges.
        if let Some(path) = edge_builder.finish() {
            pixmap.stroke_path(&path, &edge_paint, &Stroke::default(), view_transform, None);
        }

        // Draw the states on top.
        let mut state_path_builder = tiny_skia::PathBuilder::new();

        for (index, state_view) in self.view_states.iter().enumerate() {
            if index != self.lts.initial_state_index() {
                state_path_builder.push_circle(state_view.position.x, state_view.position.y, state_radius);
            } else {
                // Draw the colored states individually
                let transform =
                    Transform::from_translate(state_view.position.x, state_view.position.y).post_concat(view_transform);

                pixmap.fill_path(
                    &circle,
                    &initial_state_paint,
                    tiny_skia::FillRule::Winding,
                    transform,
                    None,
                );

                pixmap.stroke_path(&circle, &state_outer, &Stroke::default(), transform, None);
            }
        }

        // Draw the states with an outline.
        if let Some(path) = state_path_builder.finish() {
            pixmap.fill_path(
                &path,
                &state_inner_paint,
                tiny_skia::FillRule::Winding,
                view_transform,
                None,
            );

            pixmap.stroke_path(&path, &state_outer, &Stroke::default(), view_transform, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use mcrl3_lts::read_aut;
    use tiny_skia::Pixmap;
    use tiny_skia::PixmapMut;

    use super::*;

    #[test]
    fn test_viewer() {
        // Render a single from the alternating bit protocol with some settings.
        let file = include_str!("../../../../../examples/lts/abp.aut");
        let lts = Arc::new(read_aut(file.as_bytes(), vec![]).unwrap());

        let mut viewer = Viewer::new(&lts);

        let mut pixel_buffer = Pixmap::new(800, 600).unwrap();
        viewer.render(
            &mut PixmapMut::from_bytes(pixel_buffer.data_mut(), 800, 600).unwrap(),
            true,
            5.0,
            0.0,
            0.0,
            800,
            600,
            1.0,
            14.0,
        );
    }
}
