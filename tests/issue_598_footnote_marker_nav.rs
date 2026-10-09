//! 각주 탐색은 현재 마커의 문단·제어 소속과 커서 단위를 검증한다.
//! 실제 배치는 한컴 PDF 대비 Native/fresh WASM 전쪽 Visual Sweep으로 확인하며,
//! 이 검사는 절대 픽셀 좌표를 새 기대값으로 고정하지 않는다.

use std::path::Path;

use rhwp::renderer::render_tree::RenderNodeType;
use rhwp::wasm_api::HwpDocument;

#[test]
fn tac_tables_keep_stored_top_spacing_at_section_start_and_forced_page_break() {
    use rhwp::document_core::DocumentCore;
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    fn first_table(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(node.node_type, RenderNodeType::Table(_)) {
            return Some(node);
        }
        node.children.iter().find_map(first_table)
    }

    // The source's first LINE_SEG vertpos is 1500 HWPUNIT, or 20 px at 96 DPI.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/footnote-01.hwp");
    let original = std::fs::read(path).expect("read sample");
    let mut forced = rhwp::parser::parse_document(&original).expect("parse source");
    forced.sections[0].paragraphs[0].column_type = rhwp::model::paragraph::ColumnBreakType::Page;
    forced.sections[0]
        .paragraphs
        .insert(0, rhwp::model::paragraph::Paragraph::default());
    let forced_bytes = rhwp::serializer::hwpx::serialize_hwpx(&forced).expect("serialize probe");
    for (bytes, page) in [(&original, 0), (&forced_bytes, 1)] {
        let core = DocumentCore::from_bytes(bytes).expect("parse sample");
        let tree = core.build_page_render_tree(page).expect("render page");
        let body = tree
            .root
            .children
            .iter()
            .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
            .expect("body");
        let table = first_table(body).expect("first TAC table");
        assert!(
            table.bbox.y - body.bbox.y >= 19.9,
            "page {page}: table={} body={}, expected at least 20 px of source spacing",
            table.bbox.y,
            body.bbox.y
        );
    }
}

fn json_number(json: &str, key: &str) -> f64 {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern).expect("json key not found") + pattern.len();
    let rest = &json[start..];
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(rest.len());
    rest[..end].parse::<f64>().expect("json number parse")
}

fn body_marker_center(doc: &HwpDocument, para_index: usize, number: u16) -> (f64, f64) {
    let tree = doc.build_page_render_tree(0).expect("본문 마커의 첫 쪽");
    let mut pending = vec![&tree.root];
    let mut markers = Vec::new();
    while let Some(node) = pending.pop() {
        if let RenderNodeType::FootnoteMarker(marker) = &node.node_type {
            if marker.section_index == 0
                && marker.para_index == para_index
                && marker.control_index == 0
                && marker.number == number
            {
                markers.push(node.bbox);
            }
        }
        pending.extend(node.children.iter());
    }
    assert_eq!(
        markers.len(),
        1,
        "마커의 쪽·문단·제어·번호 소속과 누락/중복"
    );
    let rect = markers[0];
    assert!(rect.width > 0.0 && rect.height > 0.0, "가시 마커 상자");
    (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

fn body_paragraph_text_on_page(doc: &HwpDocument, page: u32, para_index: usize) -> String {
    let tree = doc.build_page_render_tree(page).expect("본문 문단의 쪽");
    let mut pending = vec![&tree.root];
    let mut text = String::new();
    while let Some(node) = pending.pop() {
        if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            continue;
        }
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.section_index == Some(0) && run.para_index == Some(para_index) {
                text.push_str(&run.text);
            }
        }
        pending.extend(node.children.iter().rev());
    }
    text
}

#[test]
fn issue_598_body_footnote_marker_has_hit_and_cursor_unit() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/footnote-01.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse footnote-01.hwp");

    assert_eq!(doc.get_control_text_positions(0, 3), "[7]");
    assert_eq!(doc.page_count(), 6, "한컴 PDF와 같은 쪽 소속");
    assert_eq!(
        body_paragraph_text_on_page(&doc, 2, 23),
        "설계도의 데이터를 다운로드하고 자체제작으로 조립",
        "3쪽 첫 줄 소속"
    );
    assert!(
        !body_paragraph_text_on_page(&doc, 1, 23).contains("설계도의"),
        "3쪽 첫 줄을 2쪽에도 중복 배치하지 않는다"
    );

    let (x, y) = body_marker_center(&doc, 3, 1);
    let hit = doc
        .hit_test_body_footnote_marker_native(0, x, y)
        .expect("hit body footnote marker");
    assert!(hit.contains("\"hit\":true"), "hit json: {hit}");
    assert!(hit.contains("\"sectionIndex\":0"), "hit json: {hit}");
    assert!(hit.contains("\"paragraphIndex\":3"), "hit json: {hit}");
    assert!(hit.contains("\"controlIndex\":0"), "hit json: {hit}");
    assert!(hit.contains("\"footnoteIndex\":0"), "hit json: {hit}");

    let marker_left = doc
        .get_cursor_rect_native(0, 3, 7)
        .expect("marker left rect");
    let marker_right = doc
        .get_cursor_rect_native(0, 3, 8)
        .expect("marker right rect");
    let left_x = json_number(&marker_left, "x");
    let right_x = json_number(&marker_right, "x");
    assert!(
        right_x > left_x,
        "marker right caret should be after left caret: left={marker_left}, right={marker_right}"
    );

    let next = doc.navigate_next_editable_wasm(0, 3, 7, 1, "[]");
    assert!(next.contains("\"charOffset\":8"), "next json: {next}");
    let prev = doc.navigate_next_editable_wasm(0, 3, 8, -1, "[]");
    assert!(prev.contains("\"charOffset\":7"), "prev json: {prev}");
}

#[test]
fn issue_598_second_body_footnote_marker_has_same_cursor_unit() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/footnote-01.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse footnote-01.hwp");

    assert_eq!(doc.get_control_text_positions(0, 7), "[6]");

    let (x, y) = body_marker_center(&doc, 7, 2);
    let hit = doc
        .hit_test_body_footnote_marker_native(0, x, y)
        .expect("hit second body footnote marker");
    assert!(hit.contains("\"hit\":true"), "hit json: {hit}");
    assert!(hit.contains("\"paragraphIndex\":7"), "hit json: {hit}");
    assert!(hit.contains("\"controlIndex\":0"), "hit json: {hit}");
    assert!(hit.contains("\"footnoteIndex\":1"), "hit json: {hit}");

    let next = doc.navigate_next_editable_wasm(0, 7, 6, 1, "[]");
    assert!(next.contains("\"charOffset\":7"), "next json: {next}");
    let prev = doc.navigate_next_editable_wasm(0, 7, 7, -1, "[]");
    assert!(prev.contains("\"charOffset\":6"), "prev json: {prev}");

    let marker_left = doc
        .get_cursor_rect_native(0, 7, 6)
        .expect("second marker left rect");
    let marker_right = doc
        .get_cursor_rect_native(0, 7, 7)
        .expect("second marker right rect");
    let left_x = json_number(&marker_left, "x");
    let right_x = json_number(&marker_right, "x");
    assert!(
        right_x > left_x,
        "second marker right caret should be after left caret: left={marker_left}, right={marker_right}"
    );
}

#[test]
fn endnote_marker_can_be_found_and_deleted_like_footnote() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/endnote-01.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse endnote-01.hwp");

    // 구역0·문단3·제어0의 미주 마커는 문자 위치7에 있다.
    let forward = doc
        .get_footnote_at_cursor_native(0, 3, 7, "forward")
        .expect("find endnote after cursor");
    assert!(forward.contains("\"hit\":true"), "forward json: {forward}");
    assert!(
        forward.contains("\"controlIndex\":0"),
        "forward json: {forward}"
    );

    let deleted = doc
        .delete_footnote_native(0, 3, 0)
        .expect("delete endnote control");
    assert!(deleted.contains("\"ok\":true"), "deleted json: {deleted}");
}

#[test]
fn issue_598_body_footnote_marker_can_be_found_and_deleted_from_cursor() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/footnote-01.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse footnote-01.hwp");

    let (old_x, old_y) = body_marker_center(&doc, 3, 1);

    let backward = doc
        .get_footnote_at_cursor_native(0, 3, 8, "backward")
        .expect("find footnote before cursor");
    assert!(
        backward.contains("\"hit\":true"),
        "backward json: {backward}"
    );
    assert!(
        backward.contains("\"controlIndex\":0"),
        "backward json: {backward}"
    );
    assert!(
        backward.contains("\"charOffset\":7"),
        "backward json: {backward}"
    );
    assert!(
        backward.contains("\"footnoteNumber\":1"),
        "backward json: {backward}"
    );

    let forward = doc
        .get_footnote_at_cursor_native(0, 3, 7, "forward")
        .expect("find footnote after cursor");
    assert!(forward.contains("\"hit\":true"), "forward json: {forward}");
    assert!(
        forward.contains("\"controlIndex\":0"),
        "forward json: {forward}"
    );
    assert!(
        forward.contains("\"charOffset\":7"),
        "forward json: {forward}"
    );

    let deleted = doc
        .delete_footnote_native(0, 3, 0)
        .expect("delete first body footnote");
    assert!(deleted.contains("\"ok\":true"), "deleted json: {deleted}");
    assert!(
        deleted.contains("\"charOffset\":7"),
        "deleted json: {deleted}"
    );
    assert!(
        deleted.contains("\"deletedNumber\":1"),
        "deleted json: {deleted}"
    );

    assert_eq!(doc.get_control_text_positions(0, 3), "[]");

    let missed = doc
        .get_footnote_at_cursor_native(0, 3, 8, "backward")
        .expect("deleted footnote should not be found");
    assert_eq!(missed, "{\"hit\":false}");

    let old_marker_hit = doc
        .hit_test_body_footnote_marker_native(0, old_x, old_y)
        .expect("hit old marker position after delete");
    assert_eq!(old_marker_hit, "{\"hit\":false}");

    let second_info = doc
        .get_footnote_info_native(0, 7, 0)
        .expect("remaining footnote info");
    assert!(
        second_info.contains("\"number\":1"),
        "second info json: {second_info}"
    );
}

#[test]
fn issue_598_backspace_before_marker_keeps_marker_anchor_and_undo_restores_it() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/footnote-01.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse footnote-01.hwp");

    assert_eq!(doc.get_control_text_positions(0, 3), "[7]");
    assert_eq!(
        doc.get_text_range_native(0, 3, 6, 2)
            .expect("text around marker"),
        "체와"
    );

    doc.delete_text_native(0, 3, 6, 1)
        .expect("delete text before footnote marker");
    assert_eq!(
        doc.get_control_text_positions(0, 3),
        "[6]",
        "footnote marker should stay between remaining previous text and following text"
    );
    assert_eq!(
        doc.get_text_range_native(0, 3, 5, 2)
            .expect("text around marker after delete"),
        "액와"
    );

    doc.insert_text_native(0, 3, 6, "체")
        .expect("undo-like insert before footnote marker");
    assert_eq!(
        doc.get_control_text_positions(0, 3),
        "[7]",
        "undo-like insert should restore original footnote marker anchor"
    );
    assert_eq!(
        doc.get_text_range_native(0, 3, 6, 2)
            .expect("text around marker after restore"),
        "체와"
    );
}
