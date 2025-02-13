use std::cmp::Ordering;

use glam::Vec3;



struct PointLayout {

    /// The current position
    position: Vec3,

    /// A pointer 
    index: usize,
}

/// A uniform spatial grid subdivision of the underlying set of points
struct GridSubdivision {
    /// The list of points that must be laid out
    points: Vec<PointLayout>,

    cell: Vec<usize>,

}

impl GridSubdivision {

    pub fn update(&mut self) {

        // Sort the points left to right top to bottom.
        self.points.sort_unstable_by(|p0, p1| {
            let cmp = p0.position.cmplt(p1.position);

            if cmp.test(0) && cmp.test(1) {
                Ordering::Less
            } else {
                Ordering::Greater
            }

        });

        // Put each point into the correct grid cell 
        for point in &self.points {

        }
    }

}