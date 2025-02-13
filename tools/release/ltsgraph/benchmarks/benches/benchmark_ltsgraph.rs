use std::sync::Arc;

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use io::io_aut::read_aut;
use ltsgraph_lib::GraphLayout;
use ltsgraph_lib::Viewer;
use tiny_skia::Pixmap;
use tiny_skia::PixmapMut;

/// Render the alternating bit protocol with some settings.
pub fn criterion_benchmark_viewer(c: &mut Criterion) {
    let file = include_str!("../../../../examples/lts/abp.aut");
    let lts = Arc::new(read_aut(file.as_bytes(), vec![]).unwrap());

    let mut viewer = Viewer::new(&lts);

    let mut pixel_buffer = Pixmap::new(800, 600).unwrap();

    c.bench_function("ltsgraph viewer", |bencher| {
        bencher.iter(|| {
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
        });
    });

    c.bench_function("ltsgraph viewer (no text)", |bencher| {
        bencher.iter(|| {
            viewer.render(
                &mut PixmapMut::from_bytes(pixel_buffer.data_mut(), 800, 600).unwrap(),
                false,
                5.0,
                0.0,
                0.0,
                800,
                600,
                1.0,
                14.0,
            );
        });
    });
}

/// Perform layouting the alternating bit protocol with some settings.
pub fn criterion_benchmark_layout(c: &mut Criterion) {
    let file = include_str!("../../../../examples/lts/abp.aut");
    let lts = Arc::new(read_aut(file.as_bytes(), vec![]).unwrap());

    let mut layout = GraphLayout::new(&lts);

    c.bench_function("ltsgraph layout", |bencher| {
        bencher.iter(|| {
            layout.update(5.0, 1.0, 0.01);
        });
    });
}

criterion_group!(benches, criterion_benchmark_viewer, criterion_benchmark_layout,);
criterion_main!(benches);
