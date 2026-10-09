//! Preserve source partition provenance across derived overflow composition.
//! Upstream replaced the atomic paragraph memo with a renderer-owned cache;
//! the Rust 1.99 CAS workaround is no longer needed in the source model.

use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::renderer::composer::{compose_paragraph, recompose_stored_single_line_if_overflowing};
use rhwp::renderer::style_resolver::ResolvedStyleSet;

fn invalidated_paragraph() -> Paragraph {
    let mut para = Paragraph {
        text: "가".repeat(60),
        line_segs: vec![LineSeg {
            line_height: 800,
            baseline_distance: 640,
            ..Default::default()
        }],
        ..Default::default()
    };
    para.invalidate_layout_inputs();
    para
}

#[test]
fn width_memo_updates_preserve_stored_partition_invalidation() {
    let para = invalidated_paragraph();
    let stored = para.line_segs.clone();
    let styles = ResolvedStyleSet::default();
    for (width, overflowed) in [(50.0, true), (5000.0, false)] {
        let mut composed = compose_paragraph(&para);
        recompose_stored_single_line_if_overflowing(&mut composed, &para, width, &styles, 96.0);
        assert_eq!(composed.lines.len() > 1, overflowed);
        assert_eq!(para.line_segs, stored);
        assert!(para.stored_text_partition_is_dirty());
    }
}

#[test]
fn concurrent_width_memo_updates_keep_partition_provenance() {
    let para = invalidated_paragraph();
    std::thread::scope(|scope| {
        for width in [50.0, 5000.0, 50.0, 5000.0] {
            let para = &para;
            scope.spawn(move || {
                let mut composed = compose_paragraph(para);
                recompose_stored_single_line_if_overflowing(
                    &mut composed,
                    para,
                    width,
                    &ResolvedStyleSet::default(),
                    96.0,
                );
                assert_eq!(composed.lines.len() > 1, width == 50.0);
                assert!(para.stored_text_partition_is_dirty());
            });
        }
    });
    assert!(para.stored_text_partition_is_dirty());
}
