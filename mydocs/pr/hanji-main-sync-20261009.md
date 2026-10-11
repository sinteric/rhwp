# Hanji renderer fork main 통합 검토

fork main `6a0746132043deda72a12b939665aa47559cbd8e`과 upstream main
`1a76570e833917d15817415a53c09ad61ab3203f`의 전체 이력을 merge commit
`d06b10b656821ef5f60801408852965ecfe2929b`으로 보존했다. upstream devel로
대체하지 않았으며 `hanji-pinned`는 변경하지 않는다.

## Conflict와 fork 동작

- `document_core/queries/rendering.rs`: upstream이 유지하는 collection에 동등한
  `fill(true)`/`fill(0)` 처리만 유지하고 제거된 table dirty loop를 복원하지 않는다.
- `model/paragraph.rs`: renderer로 이전된 memo 소유권을 따른다. 기존 fork 호환성 검사는
  공개 composition API의 폭 변경·병렬 실행·저장 partition provenance를 검증한다.
- `parser/cfb_reader.rs`: 완전한 UTF-16 두 바이트 chunk 처리와 홀수 끝 byte 제외를 유지한다.
- `issue_598_footnote_marker_nav.rs`: upstream의 marker 소유·쪽·커서 검사를 채택하고
  fork의 구역 첫머리 및 강제 Page 간격 보존 검사를 그대로 유지한다.

자동으로 이식된 기존 TAC fallback은 단 상단에서 문단 위 간격 전체를 추가했다.
새 upstream에서는 Native HWP5와 inline placement가 저장 원점을 이미 소비하므로,
이 조건은 미주 본문 넘침과 issue6812의 그림/표 배치 두 검사를 실패시켰다.
동일 입력의 exact upstream main은 세 검사 모두 통과한다.

Native HWP5는 upstream 처리를 유지한다. XML container의 block TAC fallback에서는
명시적 Page 또는 구역 첫 문단의 유효한 저장 LINE_SEG 원점을 사용한다. 합성 tag와
문단 위 간격보다 큰 누적 vpos는 증거로 쓰지 않는다. 같은 query 결과를
`HostSpacing.before` 수용 높이와 `layout_table_item` paint가 소비한다. 자연 이월은
문단 위 간격을 계속 제거하며, 편집 세션에서는 저장 형상 보정을 사용하지 않는다.

기존 fork 보존 검사는 바꾸지 않았다. 추가 관계 검사는 동일 forced-Page 입력에서
유효한 1500HU/750HU 저장 위치가 로드 후 유지되고 표의 차이가 그10px만 되는지
확인한다. 0-vpos는 loader가 재구성하므로 이 관계의 대조 입력으로 쓰지 않는다.
최종 변경의 Native 개발 프로필 지정 검사33건은 통과했다. 최종 head의 전체
release-test·lint·Native Skia·fresh WASM·시각 검증은 별도 실행 기록으로 확인해야 한다.
골든이나 baseline을 바꾸어 실패를 감추지 않았다.

## 독립 Print가 필요한 정확한 입력

[footnote-forced-page-exact.hwpx](assets/hanji-main-sync-20261009/footnote-forced-page-exact.hwpx)

- 원본: 공개 `samples/footnote-01.hwp`, SHA256
  `5bbc8a8fd23415aad59dd91d1eb261050946d9c64635933fcd49609ec2cc94e5`.
- 생성: 원본을 parse한 뒤 원래 첫 문단을 `ColumnBreakType::Page`로 지정하고,
  `Paragraph::default()`를 문단0에 삽입한 뒤 전체 model을 HWPX로 serialize했다.
  기존 fork 보존 검사와 같은 입력이다.
- 크기13052bytes, SHA256
  `1f57ec6fcd8b82d453fe07011a7721824020ecf7b9127c9b10c4d0a3b7578e38`.
- 저장 metadata: Hancom Office2020 /11.0.0.3524. Native 관측7쪽은 독립 기대값이 아니다.
- 필요 출력: 동일 bytes를 Hancom Office2020 (`hancom2020`)에서 전쪽 Print PDF로
  출력한다. `pdf_print_method=0`, 기본1-up,100%, 입력 전처리·N-up·쪽맞춤 없이 수행한다.
  실제 제품/build, 입력 hash, Print method/job/status, PDF hash와 쪽수를 보존한다.

이 입력의 독립 Print PDF는 아직 없다. 기존 각주6쪽 PDF나 간격0 대조군은 이 입력의
reference가 될 수 없다. 따라서 전체 시각 승인이나 ordinary PR 준비 완료로 판정하지 않는다.
공개 OFL 글꼴을 사용한 geometry 비교와 font ink 잔여 차이도 구분해서 보고한다.
