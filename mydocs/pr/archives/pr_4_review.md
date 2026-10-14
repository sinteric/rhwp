# PR4 검토 기록

최종 판정: source 변경 검토 완료. 최신 head 전체 GitHub Actions가 아직 진행 중이며,
실패·대기·취소 상태에서는 머지하지 않는다.

대상: sinteric/rhwp fork의 main. upstream edwardkim/rhwp와 devel은 변경하지 않는다.
base: 260c6df57f7655875be83f6d23acc83499a0a2a4
code candidate: 4f9e2f355b67c6df614ab82f9e2fc87eac5d59b6

collaborator_self_merge, intake_and_review, local_validation 경로를 적용했다.
O3 실행 계약 변경으로 해당 workflow 제어와 CI impact 검증을 수행했다.
Rust·Nextest·Node 실행 동시성1 설정은 제품 thread 동작과 컴파일러 병렬성을 바꾸지 않는다.
A/B/C/D의 source-side lib와 배타적인 integration inventory, count/provenance/승격
집계, native Skia/WASM/browser/package 핵심 gate를 보존했다. 추가 CI 검증의 편입과
로컬 PR 근거 기준은 기존 운영 문서에 반영했다.

181 workflow 제어와118 Node CI impact/policy/controller/evidence 검증이 통과했다.
actionlint/ShellCheck는 기존 SC2016 advisory를 제외하고 통과했다. 최초 실행에서 기존
Node command 문자열 제어가 새 동시성 옵션을 거부했으므로 목적을 유지한 새 command
검증으로 수정하고 전체181개를 재실행했다. diff 검사도 통과했다.

조판 원칙·시각 sweep은 Rust/render/fixture/HWP/PDF 출력 소스 변경이 없어 비해당이다.
실제 문서 바이너리 결과나 로컬 전체 native/renderer 통과를 주장하지 않는다.
테스트 정책·workflow 후보와 source diff를 검토했고 추가 결함은 찾지 못했다.
실행기 동시성 감소 자체의 성능 개선은 측정하지 않았다.

사용자가 이 CI 최적화의 PR 게시와 통과 시 머지를 승인했다. 전체 최신 head CI가
통과하면 SHA를 고정해 머지하고 검증한 변경 파일과 merge tree를 대조한다.
branch push의 검증/배포 트리거를 추가하지 않았다. 기존 태그·배포 정책은 그대로다.
작업 디렉터리와 branch는 사용자 보존 지시에 따라 남긴다. 다른 contributor의 PR,
upstream branch, runner 등록이나 실제 배포에는 이 작업의 승인을 적용하지 않는다.
