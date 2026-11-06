# FolioRivet — feasibility, qualification and production roadmap

## Public preview preparation — 6 November 2026

User-approved 0.1.0 preparation is IN PROGRESS. Python/CLI only, Windows x64 and
Ubuntu 22.04 x64 with CPython 3.12; eventual public GitHub/PyPI release and shadow
generation on separate production machines. See [current release status](../release/STATUS.md)
and [build/acceptance instructions](../release/BUILD.md). Development candidates
and installed regressions pass on the existing workstations; this is not final
release clearance. Actual target hosts/production source, new Word review,
final notice/resource review and controlled timing investigation remain pending.
Historical evidence is privately archived behind an ignored local junction;
existing evidence paths remain usable locally and are excluded from source exports.
No publication, commit, push or cutover has occurred. Stages 8–12 remain planned.


Status: Stages 1–5 retain their recorded PASS scopes. Stage 6 bounded Lantern functional qualification PASS: Windows/Linux native execution, Word 2019 automated checks, user-confirmed independent Word review and user-reported successful onboarding. Detailed onboarding measurements/logs remain uncollected; full production gate NOT VERIFIED. Stage 6.5 COMPLETE for bounded local prototype qualification and issue disposition (evidence/stage6.5/CLOSURE.md); full Linux CI remains failing and distribution/production remain on hold. Clean-host and independent onboarding studies are deferred. Stage 7 COMPLETE: bounded prototype feasible with limitations; public preview HOLD and production migration not recommended yet. Stages 8–12 PLANNED, not started. Updated 6 November 2026. See [independent results](evidence/stage6/INDEPENDENT_RESULTS.md) and [Word 2019 evidence](evidence/stage6/word2019/README.md).

## Scope adjustment — 6 November 2026

The user has removed clean-host qualification and measured independent onboarding as prerequisites for completing Stage 6.5 and assessing the current bounded prototype in Stage 7. A solo-maintainer project cannot currently supply those environments/participants. Both studies are deferred until broader adoption or contributors make them practical. This explicitly supersedes earlier gate language treating their absence as a blocker for the current prototype; historical evidence and original requirements below remain recorded. It does not claim either study passed, remove independent Word compatibility review, or change later production/public-release acceptance criteria. Reassess deployment and onboarding evidence when broader support or release scope is agreed. All other Stage 6.5 requirements remain in force; this change alone does not complete the stage.

## Source boundary and target users

This plan reconstructs the detailed seven-stage roadmap visible in `Pasted text(2).txt`. The user has now supplied the original-plan explanation, preserved in [the supplied source](ORIGINAL_PLAN_SUPPLIED.md). This revision reconciles it with the earlier detailed roadmap. Every earlier visible stage work item, verification requirement, deliverable and gate remains retained below. The explanation refers to an underlying technical plan that is not separately available; no unseen requirements are claimed verified. See [source and status](SOURCE_AND_STATUS.md).

FolioRivet is a working name for an independent Wordcraft fork for Python developers generating editable DOCX from Word-authored templates. Repository/package availability and trademark clearance are unverified. Differentiation is proposed: demonstrated template fidelity, supported-platform installation and useful diagnostics.

## Roadmap extension — 5 November 2026

The user requested Stage 6.5 for outstanding fixes and qualification, retaining Stage 7 as the feasibility decision and adding Stages 8–12 for production. These additions supplement the original seven-stage plan; they do not weaken or replace its gates. Internal report-pipeline migration is prioritized; public release is a separate later decision. This documentation update records planned work, not execution or gate completion.

| Stage | Work | Completion gate |
|---|---|---|
| 6.5 — Close outstanding fixes and qualification gaps | Reproducible build tooling, missing checks, storage qualification and artifact review; clean-host and independent onboarding studies deferred. | Remaining issues fixed or given explicit evidence-backed limitations; required unverified gates stay pending. |
| 7 — Complete feasibility review and handoff | Audit Stages 1–6 and 6.5, capability matrix, patch/maintenance register and separate preview/production recommendations. | Evidence-backed decision; unmet production gates remain pending. |
| 8 — Define the actual production integration | Inspect real sources, inventory templates/callers and agree contracts and operational requirements. | Complete production coverage and acceptance criteria agreed. |
| 9 — Implement production coverage | Required features, templates and opt-in integration. | All inventoried production scenarios work with the existing default retained. |
| 10 — Qualify the production candidate | Complete pipeline, supported platforms, Word review, failure and rollback rehearsals. | Full Stage 6 production gate passes; Stage 7 production recommendation reassessed. |
| 11 — Controlled rollout | Shadow generation, limited adoption and monitored expansion after separate cutover approval. | Agreed production acceptance and rollback criteria pass. |
| 12 — Stabilize and retire legacy dependencies | Resolve operational issues; retire obsolete rendering paths after the rollback period. | No callers need the old path; maintenance ownership established. |

## Reconciliation with the newly supplied plan

The supplied explanation confirms all seven stages, the two-platform requirement, template fidelity before bindings, native installed-extension execution, real Word review and separate migration authorization. Its explicit acceptance conditions supplement rather than replace the retained gates:

- Stage 3 requires useful error handling as well as correct formatting and valid DOCX. A Python wrapper that launches a CLI subprocess does not satisfy the native-binding gate.
- Stage 4 requires tables and charts to display correctly in Microsoft Word with no repair prompts, missing values, broken relationships or formatting loss.
- Stage 6 requires semantically correct output on both platforms and protection of existing outputs, manual edits and publication behavior.
- Stage 7 remains an evidence-based feasibility decision, never automatic authorization to cut over.

### Proposed bounded template conventions

These conventions are now explicit planning inputs from the supplied explanation. They are not existing Wordcraft API guarantees. Validate target support and scope during the appropriate stage before claiming implementation.

| Tag | Proposed meaning | Stage |
|---|---|---|
| `value:metadata.model_id` | Populate a scalar | 3 |
| `rows:class_distribution` | Repeat a prototype data row | 4 |
| `repeat:class_training_results` | Repeat a logical block | 5 |
| `value:item.class_name` | Resolve a field in the current repeated item | 4â€“5 |
| `if:has_evaluation` | Include/remove the complete optional block | 5 |
| `image:training_results` | Insert an existing image | 4 |

Do not extend these into a general-purpose expression language. Define target boundaries, required/optional status, duplicate resolution and supported scope explicitly and test them through the shared resolver.

### Qualitative risk priorities

The supplied explanation ranks DOCX fidelity critical; repeated blocks and cross-platform native packaging high; Python/Rust data transfer lower. These are qualitative priorities, not measured scores. They support the existing gate sequence without asserting runtime feasibility.

## Architecture and non-negotiable safeguards

Implement Option A: thin synchronous native PyO3 bindings to the existing Rust engine, sharing operations and target resolution with the existing headless CLI command registry. A fresh document session per job; core operations remain usable without Python. Isolate maturin/Python packaging in a binding crate. Do not duplicate the renderer, headless app, mutable model, report schema, metrics or business rules.

Preserve the default Jinja2/Pandoc renderer and existing public API during the spike. Preserve validated facts, units, unavailable wording, charts, provenance, staging, cleanup, previous generations, manual edits and template originals. A required DOCX failure fails the experimental generation; Markdown is not a silent successful substitute. No commits, pushes, publishing or production cutover are authorized by this package.

Read [legal/IP policy](LEGAL_IP_POLICY.md) before implementation and distribution. Template operations are limited to required scalars, rows/blocks, optional content and pictures. No desktop feature expansion, hosted service, plugin framework, AI writing, PDF pagination guarantees or universal DOCX claims.

## Execution sequence

Stages are sequential evidence gates. Do not advance through an unmet required gate. If a platform or Word review is unavailable, record NOT VERIFIED and keep dependent acceptance pending. A bounded engine-repair decision after Stage 2 must identify scope and evidence, not silently waive fidelity.

### Stage 1 â€” Establish and build a pinned baseline

**Purpose:** Confirm that the selected Wordcraft revision can run unattended on both target platforms before adding functionality.

Work:

- Locate the report repository and establish a separate Wordcraft fork workspace.
- Read upstream and nested `AGENTS.md` instructions. Merge applicable guidance without replacing upstream rules.
- Keep the future-fork handoff as documentation until the fork exists, then place its rules at the appropriate repository scope.
- Verify the report package paths and current implementation against the supplied plan. Correct stale names, links or assumptions.
- Select and record an exact upstream revision.
- Record the Rust toolchain, dependency lockfiles, build features, operating-system details and required runtime dependencies.
- Build the existing headless CLI on Windows x64 and Linux x64.
- Run a minimal document job on both platforms without opening a desktop window or requiring Microsoft Word.
- Record exact commands, output locations, fonts and any environmental requirements.

Deliverables:

- Reproducible baseline build instructions.
- A pinned upstream revision and build-input record.
- CLI smoke-test results for both platforms.
- An initial capability and gap register.

**Gate:** The unmodified CLI builds and performs a real headless document operation on both platforms. Any unavailable platform remains explicitly unverified.


**Additional proposed deliverables (existing gate unchanged):** Define FolioRivet's audience and independent upstream relationship. Inspect the headless/native dependency footprint without speculative restructuring. Open a dependency/asset provenance register and identify inherited licensing questions. Record the exact upstream revision, toolchain, lockfiles and runtime requirements. Do not advertise the proposed Windows x64/Linux x64/CPython 3.12 support as achieved.

### Stage 2 â€” Prove Word-template fidelity

**User scope clarification, 5 November 2026:** no report repository is present on this computer; the current report uses Jinja2 templates rather than Word templates. The user authorized a generated Word master demonstrating the listed features. The six-page, three-section [representative master and feature inventory](evidence/stage2/representative-v2/COVERAGE.md) satisfy the Stage 2 inventory for this spike. Actual report/Jinja2 mapping is deferred to Stage 6 and is not a Stage 2 blocker. The user subsequently completed the interactive Word checklist, reported both versions saved and reopened successfully, and both saved copies passed structural checks. See [manual review](evidence/stage2/representative-v2/manual-review.json).

**Purpose:** Determine whether Wordcraft preserves the document features the report needs before investing in bindings.

Work:

- Inventory the factual content and layout requirements of the existing report templates.
- Author one representative DOCX master in Microsoft Word containing:
  - Branding, named paragraph and character styles.
  - Headers, footers and section settings.
  - A table long enough to span pages, including repeated header rows.
  - A prototype per-class heading and table block.
  - A chart placeholder.
  - An optional evaluation section.
  - Editable manual-comment areas.
- Add tagged content controls using the proposed scalar, repetition, condition and picture conventions.
- Preserve the original template and record its hash.
- Open and save a copy through unmodified Wordcraft.
- Compare the original and resulting DOCX structures, including styles, sections, tables, controls, relationships and media.
- Inspect the result in Microsoft Word for repair prompts, layout changes and lost content.
- Save it again in Word and repeat the relevant checks.
- Reduce each discovered problem to a small reproducible example.

Deliverables:

- The representative template and round-trip outputs.
- A feature-by-feature fidelity assessment.
- Reproducible examples for upstream gaps.
- A proposed repair scope for any required features that fail.

**Gate:** Required content and formatting survive the round trip. If they do not, pause binding implementation and decide whether the engine repairs are sufficiently bounded to continue.


**Additional proposed deliverables (existing gate unchanged):** Use synthetic, redistributable fixtures. Record authorship, source, license and redistribution rights for each template, font and image. Start a feature-level compatibility matrix, including named styles, headers/footers, sections, multipage tables, blocks, pictures, Unicode and Word open/save. Separate structural preservation from visual observations.

### Stage 3 â€” Implement native bindings and scalar population

Execution record: [Stage 3 evidence](evidence/STAGE3.md). Shared scalar targeting,
inspection/preflight, an isolated native PyO3 crate and typed convenience API are
implemented locally. Qualification results and remaining gates are recorded there;
implementation alone does not pass this stage. Later-stage operations remain disabled.

Remaining formatting-gate repair: [bounded style repair and blast-radius plan](evidence/stage3/STYLE_REPAIR_PLAN.md).
This planning record covers shared DOCX/model risks, explicit-edit invalidation,
minimal regressions, two-platform native requalification and Word behavior checks.
It does not mark the gate passed or claim a repair has been implemented.

**Purpose:** Prove the complete Python-to-Rust-to-DOCX path using the smallest useful operation.

Work:

- Inspect the existing Rust engine and command registry to identify reusable loading, targeting, editing and saving operations.
- Add an isolated PyO3 binding crate and maturin packaging.
- Keep core document operations independent of Python.
- Start with the proposed synchronous interface:

  ```python
  render_template(template_path, context_json, assets_dir, output_path) -> str
  ```

- Create a fresh document session for each call.
- Define a bounded JSON payload contract using validated report values. Do not introduce another report schema or recalculate metrics in Rust.
- Implement explicit lookup of tagged scalar targets.
- Preserve surrounding template formatting when replacing values.
- Fail clearly for missing or ambiguous required targets.
- Include operation and target information in Python exceptions.
- Expose the same underlying operation through the existing CLI command registry.
- Build wheels and install them into clean CPython 3.12 environments on both target platforms.
- Exercise the installed extension, rather than relying on source-tree imports or mocked conversion.

Verification:

- Correct scalar values, including Unicode, XML-sensitive characters, multiline text and long labels.
- Clear failures for malformed input and invalid targets.
- Template originals remain unchanged.
- Binding and CLI outputs agree semantically.
- No desktop window or Word installation is required for generation.

Deliverables:

- A working native binding and shared CLI operation.
- Installed-wheel test results.
- A documented interface and error contract.
- A scalar-populated DOCX inspected in Word.

**Gate:** An installed native extension produces a valid DOCX with correct values and preserved formatting on both platforms.


**Additional proposed deliverables (existing gate unchanged):** Add a thin typed Python convenience layer for paths/dictionaries, serialization, docstrings and discoverable exceptions. It must not duplicate report logic. Implement tag inspection and scalar preflight using the renderer's shared target resolver. Report operation, target, scope and actionable errors for malformed, missing, ambiguous or unsupported targets and missing required assets. Preflight cannot guarantee later save/layout success. Exercise the quick start against an installed wheel in a clean environment; verify whether users need a Rust compiler.

### Stage 4 â€” Add repeated table rows and a chart

**Purpose:** Produce the first useful report fragment while testing the most important document-structure risks.

Work:

- Verify that imported controls reliably identify a complete prototype table row.
- Reuse existing engine commands where their behavior is sufficient.
- Add a narrow row-expansion operation only where a verified gap exists.
- Populate each repeated row using values from its current item scope.
- Preserve table widths, cell formatting, borders, paragraph styles and header behavior.
- Handle identifiers and relationships correctly when duplicating content.
- Define empty-list behavior as removal of the prototype data row while retaining the intended table structure.
- Insert one existing chart asset into a tagged picture slot.
- Respect the slotâ€™s dimensions and preserve the imageâ€™s aspect ratio.
- Distinguish required missing assets from optional missing charts, using the existing unavailable wording where appropriate.

Verification:

- Zero, one, three and many data rows.
- Long class names, Unicode and multiline values.
- Page continuation and repeated headers.
- Correct row count, order and values.
- Valid embedded-media relationships and unique relevant identifiers.
- Equivalent CLI and binding behavior.
- Word opens the result without repair and displays the chart correctly.

Deliverables:

- A populated class-result table with an embedded chart.
- Focused structural and semantic tests.
- Word compatibility findings.
- Documentation of any unsupported row features.

**Gate:** Repeated rows and chart insertion work without corrupting document structure or losing required formatting.


**Additional proposed deliverables (existing gate unchanged):** Add an invoice/quotation example using only supported public APIs and existing operations. Supply precomputed business totals from Python. Package an editable template, synthetic input, script, generated DOCX, preview and tested compatibility record only after execution. Document provenance and licenses. Do not add invoice-specific Rust branches.

### Stage 5 â€” Add repeated sections and optional evaluation

**Purpose:** Support the reportâ€™s variable-length structure without introducing a general template language.

Work:

- Identify complete repeatable block boundaries, including headings, tables and associated spacing.
- Implement repeated per-class sections using the same shared Rust operations exposed through both interfaces.
- Resolve item fields within the correct repeated scope.
- Preserve identifiers, relationships and formatting across duplicated blocks.
- Implement explicit optional-block inclusion and removal.
- Ensure omitted evaluation content removes its complete section, including headings and associated breaks where appropriate.
- Validate required target uniqueness within its scope.
- Detect unresolved required tags before reporting success.
- Leave manual-comment areas as editable template content.

Verification:

- Empty, single-item and multi-item repeated sections.
- Evaluation present and absent.
- Optional charts present and absent.
- Missing or ambiguous required targets fail clearly.
- No orphan headings, unintended blank sections or broken references.
- Section order and factual content match validated Python input.
- Representative outputs survive Word open/save.

Deliverables:

- A template capable of representing the spikeâ€™s complete report structure.
- Shared repetition and conditional operations.
- Tests covering the supported combinations.
- An explicit list of supported template conventions and limitations.

**Gate:** Repeated and optional sections match the input data, preserve required formatting and leave no unresolved required targets.


**Additional proposed deliverables (existing gate unchanged):** Extend shared inspection/preflight to repetition scopes, optional blocks and unresolved required tags. Add a project-status example with the same complete evidence bundle as the invoice example. Required target uniqueness is evaluated within supported scopes. Keep conventions bounded; do not introduce arbitrary expressions or executable template code.

### Stage 6 â€” Integrate and qualify the experimental report path

**Purpose:** Connect the proven native operations to the real report package while preserving its existing behavior.

Work:

- Trace current report assembly, chart creation, overrides, validation, staging and publication in the actual source.
- Add an explicit experimental entry point without changing the default renderer or existing public return contract.
- Reuse existing data assembly and chart assets.
- Apply overrides and schema validation before constructing renderer input.
- Keep report-specific decisions and unavailable-value semantics in Python.
- Generate the DOCX only inside the existing staging workflow.
- Publish a generation only after required output checks succeed.
- Preserve cleanup behavior, previous generations and manually edited reports.
- Record renderer version, pinned engine revision and template hash alongside existing provenance.
- Treat required DOCX failure as a failed generation.
- Run applicable report tests and affected CLI, training and publication tests.
- Execute real installed-wheel and CLI jobs on Windows x64 and Linux x64.

Qualification should cover:

| Area | Evidence required |
|---|---|
| Report facts | Values, units, metadata and source notes agree with validated report data. |
| Document structure | Expected tables, sections and media exist; relationships remain valid. |
| Failure handling | Failed rendering never publishes an incomplete generation. |
| Preservation | Templates, previous generations and manual edits remain unchanged. |
| Compatibility | Representative outputs open and save successfully in Word. |
| Distribution | Clean installation and unattended execution work on both platforms. |
| Performance | Runtime and peak memory are measured for representative and larger fixtures. |

For performance measurements, record input sizes, environment, commands and measurement method. Avoid setting arbitrary performance claims before collecting results.

Deliverables:

- An operational experimental report entry point.
- Cross-platform native integration results.
- Representative generated reports.
- Failure-path results and performance measurements.
- An updated evidence log with unresolved limitations.

**Gate:** The real report pipeline produces correct DOCX output through installed native bindings on both platforms and preserves publication safety.


**Additional proposed deliverables (existing gate unchanged):** Test onboarding with independent users: install, generate, edit a template, diagnose a missing field and adapt sanitized input. Record assistance and failures. Ten minutes to a first sample is a proposed measured usability target, not a promise. Compare correctness before performance; pin comparator versions and use equivalent fixtures. Record runtime, peak memory, install footprint and setup steps without unmeasured speed claims. Assemble the training-report example bundle and troubleshooting guide.

### Stage 6.5 — Close outstanding fixes and qualification gaps

**Status:** COMPLETE for bounded local prototype qualification and evidence-backed issue disposition. See [closure and limitations](evidence/stage6.5/CLOSURE.md). Full Linux CI is not green; no desktop, release or production qualification is claimed. Remaining findings are explicit support limits and release/production holds with owners and next actions. Clean-host and independent onboarding studies are deferred by user instruction. Stage 7 assessment is now recorded in [the handoff](evidence/stage7/README.md); its preview/production holds remain in force.

**Purpose:** Complete remaining qualification and release preparation without treating the bounded Lantern fixture as the full production pipeline.

Work, in priority order:

1. Make qualification reproducible: remove machine-specific paths, verify the actual language-data revision against `craft-i18n.rev`, use unique run directories, preserve failed logs and include equivalent suite/install checks on both platforms. Document bootstrap prerequisites; existing scripts rely on prepared toolchains and environments.
2. Complete check coverage: run applicable full upstream and binding checks with documented inputs, investigate failures and document the ignored integration test. Earlier focused checks are not full CI. Record unavailable checks explicitly.
3. Record the tested Windows/Linux workstation environments, fresh-venv installation results, compiler/runtime requirements and exact artifacts. Clean-host qualification, including Windows without Office, is deferred to future adoption testing and is not required to pass Stage 6.5. Retain the limitation: current evidence does not establish clean-machine compatibility.
4. Retain onboarding instructions and earlier user-reported success. Measured independent onboarding is deferred until contributors/users are available and is not required to pass Stage 6.5. Keep the prospective exercise for future adoption testing; do not claim independent usability measurements or invent historical timings.
5. Audit distributable wheels and CLI: newly embedded language data, dependencies, notices, source/assets and branding. Resolve missing notices and prohibited branding before distribution; preserve upstream licenses and attribution. Review does not authorize release.
6. Confirm filesystem behavior on intended deployment storage and unsupported mounts. Retain atomic no-replace publication, fail closed where unsupported and protect templates, previous generations and manual edits.
7. Reconcile current status summaries with superseding results while retaining history. Make fixed defects, supported limitations and external evidence gaps explicit.

Deliverables: reproducible scripts/instructions; full-check and available fresh-venv install records; artifact/provenance review; storage failure results; reconciled gap register with evidence, owners and next actions. Clean-host and independent onboarding studies are future adoption work, not required deliverables for this stage.

**Gate:** Remaining in-scope issues are fixed or have explicit evidence-backed limitations. Clean-host and independent onboarding evidence are explicitly removed from this gate by user instruction on 6 November 2026 and deferred to future adoption testing. They remain unverified limitations, not failed Stage 6.5 requirements or claimed passes. Other required missing evidence stays pending. Stage 6.5 completion does not establish full production readiness or authorize distribution/cutover.

### Stage 7 â€” Decide feasibility and prepare the migration handoff

**Purpose:** Make an explicit decision based on demonstrated results and define any subsequent production work.

**Status:** COMPLETE for the bounded prototype feasibility assessment, 6 November 2026. See [Stage 7 decision and handoff](evidence/stage7/README.md). Prototype: proceed with bounded limitations. Public preview: HOLD. Production: do not migrate yet. Stage 8 is not started. The following scope remains the assessment contract: review Stage 6.5 alongside the original stages. Explicitly distinguish prototype feasibility from production-migration readiness: Stage 7 may conclude that the prototype is feasible while recommending **do not migrate yet**. The actual production ingestion, training and publication workflows have not been supplied or qualified. Reassess the production recommendation after Stage 10; do not imply that every original gate passed when recording a bounded feasibility decision.

The handoff must include a gate-by-gate assessment, evidence-linked capability matrix, patch register with regression coverage and upstream maintenance cost, reproducible setup/verification instructions, separate public-preview and production-migration decisions, and unresolved requirements with owners/blockers.


Work:

- Review every acceptance gate against recorded evidence.
- Separate completed capabilities from partial, failed and untested capabilities.
- Produce a capability matrix covering template preservation, population operations, packaging and Word compatibility.
- Summarize each fork change and the upstream gap it addresses.
- Assess the maintenance cost of carrying those changes against future upstream revisions.
- Provide reproducible setup, generation and verification instructions.
- Recommend one of three outcomes:
  - **Proceed:** All required gates pass.
  - **Proceed with bounded limitations:** A narrower supported scope is acceptable and explicitly agreed.
  - **Do not migrate yet:** Required fidelity, reliability or distribution gates remain unmet.
- If feasible, prepare a separate production migration plan covering:
  - Full template coverage.
  - Public API and output-contract changes.
  - CLI, training pipeline and publication changes.
  - Caller and regression-test updates.
  - Rollout and rollback arrangements.
  - Eventual retirement of Jinja2/Pandoc dependencies.

Deliverables:

- A feasibility decision with supporting evidence.
- Reproducible build and usage instructions.
- Prototype templates and generated outputs.
- A gap and maintenance register.
- A proposed production migration plan, if justified.

**Gate:** The decision is supported by actual builds, installed-binding execution, document checks and Word review. Completing the spike does not itself authorize production cutover.


**Additional proposed deliverables (existing gate unchanged):** Make two separate decisions: public-preview readiness and production-migration readiness. Apply the additional criteria below; public-preview approval never authorizes production replacement. Publish evidence-backed limitations, release notes, reproduction guidance, sanitized issue templates and contribution guidance. Track clean installs, independent integrations, repeat usage and bug resolution; stars are secondary and never promised.

Across every stage, the implementation plan should remain the evidence record: exact revision, environment, commands or review procedure, results, artifacts and remaining limitations. All repository changes remain uncommitted for your review.

## Production stages following the Stage 7 handoff

Stages 8–12 are PLANNED, not started. They require the actual production package: the Lantern fixture omits ingestion, training and production publication workflows. Maintain report calculations/business rules in Python and shared document operations in Rust. Keep the old renderer available during qualification. No uv workspace or historical report behavior is assumed.

### Stage 8 — Define the actual production integration

Inspect the real source and tests. Inventory every template, optional section, chart, language/font requirement, report variant and caller. Agree public API, CLI, output-path, return-value and provenance contracts; expected workload sizes, concurrency, runtime/memory requirements and supported filesystems; existing cleanup, publication, manual-edit preservation and recovery behavior. Record required feature gaps, acceptance thresholds, owners and dependencies before implementation.

**Deliverables:** production coverage inventory, agreed contracts, operational requirements and acceptance matrix.

**Gate:** Complete production coverage and acceptance criteria are agreed from inspected sources. Missing source blocks this gate; fixture behavior is not a substitute.

### Stage 9 — Implement production coverage

Implement only additional document features required by the Stage 8 inventory. Migrate templates and connect real callers through an opt-in renderer, retaining the existing default. Update affected CLI/training/publication code and regression tests. Preserve error handling, source templates, previous generations, manual edits and provenance; keep unsupported requirements explicit.

**Deliverables:** opt-in production implementation, migrated templates, caller/test updates and feature-level evidence.

**Gate:** All inventoried production scenarios work through installed native bindings without changing the existing default; required unsupported features block the gate.

### Stage 10 — Qualify the production candidate

Exercise the complete production pipeline on supported Windows/Linux environments through installed wheels and CLI. Verify facts, units, ordering, structure and preservation, then Word open/save/reopen and visual fidelity. Test malformed inputs, missing assets, concurrent destinations, interrupted writes, permission failures, large reports and publication/recovery behavior. Measure runtime/memory against agreed Stage 8 requirements. Any output-affecting repair needs renewed compatibility evidence. Rehearse rollback and record all failures and limitations.

**Deliverables:** production acceptance results, representative reviewed outputs, performance records, failure/rollback results and updated Stage 7 recommendation.

**Gate:** The full original Stage 6 production gate has evidence on both platforms and the Stage 7 production recommendation is reassessed against all original gates. Remaining required failures block production migration. Qualification is not cutover approval.

### Stage 11 — Controlled rollout

After separate user cutover approval, start shadow generation, then a limited cohort and monitored expansion. Agree measurable acceptance/rollback thresholds and a review period before rollout. Restore the previous renderer on incorrect facts, corruption, publication-safety failures or unacceptable operational regressions. Monitor failures, output checks, runtime/memory and rollback effectiveness; retain previous generations and manual edits.

**Deliverables:** rollout observations, incident/rollback records and production acceptance decision.

**Gate:** Agreed production acceptance and rollback criteria pass. Neither Stage 7 completion nor public-preview readiness authorizes this rollout.

### Stage 12 — Stabilize and retire legacy dependencies

Resolve operational issues and establish maintenance ownership, release/update procedures and regression qualification. Retire obsolete Jinja2/Pandoc rendering paths and dependencies only after the agreed rollback period and a caller/dependency audit confirms they are unused. Retain dependencies still used for other outputs or callers.

**Deliverables:** stabilization record, maintenance process, caller audit and verified retirement changes.

**Gate:** No remaining callers require the retired path and maintenance ownership is established.

Public package release remains a separate later decision requiring its own supported-platform matrix, redistributable examples, artifact/license review, release documentation and maintenance process. Internal production success does not authorize publication. All work remains uncommitted for review; no push, hosted fork, publication or production cutover is authorized by this roadmap.

## Public-preview acceptance (separate Stage 7 decision)

- All capabilities advertised in preview have actual fixture and installed-package evidence; unsupported scope is explicit. Applicable stages 1â€“5 pass for that scope.
- Clean installation and unattended sample generation on Windows x64/Linux x64 with CPython 3.12; compiler/runtime requirements accurately documented.
- Shared inspection/preflight diagnostics, typed convenience API and an independently followed quick start.
- Three redistributable examples (training/evaluation, invoice/quotation, project status), each with template, input, script, actual DOCX, preview and compatibility status.
- Feature-level compatibility matrix, known limitations, license/notice review, provenance records, sanitized issue and regression-fixture contribution guidance.
- Reproducible release inputs and maintenance owner/process. No release is authorized merely by this checklist.

## Production-migration acceptance (separate Stage 7 decision)

All seven original gates, including real Stage 6 report execution on both platforms, must have evidence. Agree full template coverage, public API/output contracts, affected CLI/training/publication callers, regression updates, rollout, rollback and eventual Jinja2/Pandoc retirement. Any narrower scope requires explicit agreement and must not be described as passing the original full scope. Production cutover requires a separate user instruction.

## Upstream maintenance

Maintain a patch register: upstream gap, minimal reproduction, affected files, rationale, regression fixture and upstream disposition. For an upstream update, record the old/new revisions, review upstream instructions and licenses, rebuild and rerun affected gates plus compatibility qualification before changing support claims. Keep core improvements separate from binding and report-specific changes.

## Evidence log â€” current local setup

Updated 4 November 2026 after executing initial setup. Canonical working checkout: this repository, pinned to `92ecb40105f56b674c18e3e88557e1456591868b`. The original setup package remains an unchanged historical handoff. See [Stage 1 execution record](evidence/STAGE1.md) and [gap register](evidence/GAPS.md) for environment, logs, commands and limitations.

| Stage | Status | Evidence |
|---|---|---|
| 1 | PASS | Pinned, unmodified CLI built and generated/reopened DOCX on Windows x64 and Ubuntu 22.04 WSL2 x64. Linux isolated runtime trace launched no child executable; extracted document text agrees across platforms. See evidence/STAGE1.md for exact scope and remaining qualification limits. |
| 2 | PASS for agreed representative-template scope | Six-page, three-section generated master passes Windows/Linux structural and 24 page comparisons. User completed the interactive Word checklist and reported successful save/reopen for both outputs; both manual copies pass structural checks. No repair/recovery messages reported. Actual Jinja2 mapping remains Stage 6; subsequent Stage 3 results are recorded below. See [feature coverage](evidence/stage2/representative-v2/COVERAGE.md) and [manual review](evidence/stage2/representative-v2/manual-review.json). |
| 3 | PASS for agreed representative scalar-template scope | Native CPython 3.12.12 wheels installed/executed on Windows x64 and Ubuntu 22.04 WSL2 x64; eight scalar targets, CLI agreement, failure paths, quick start and Linux no-child-process trace pass. Both six-page outputs pass Word automated open/save/reopen and structural checks. Original outputs passed user prompt review; style losses are now repaired and verified on both platforms, and both repaired saved copies now pass structural checks and user-reported opening without repair prompts; the earlier minimal-data proofing failure passes with the required pinned language data (see remediation). See [Stage 3 record](evidence/STAGE3.md). |
| 4 | PASS for agreed representative row/chart scope | Shared bounded rows/chart operations pass installed Windows/Linux wheels and CLI checks, twelve Word automation cases and 31 matching page pairs. Both user-saved representative copies match automated Word-resaved structures, retain chart media and valid relationships, and opened without repair prompts according to the user. The earlier proofing failure is resolved with pinned language data; wider qualification limits remain. See [Stage 4 execution record](evidence/STAGE4.md). |
| 5 | PASS for bounded representative block scope | Shared whole-block repetition and explicit boolean inclusion, scoped preflight, 15 focused tests per platform, 16 installed cases and seven failure cases per platform, scalar regressions, CLI agreement, twelve automated Word open/save/reopen cases and 43 identical page pairs. Project-status example executed. Both user-saved representative copies match automated Word saves and opened without repair prompts according to the user. Nested variable blocks and reference-bearing prototypes remain unsupported. See [Stage 5](evidence/STAGE5.md). |
| 6 | PASS bounded fixture functional scope; full gate NOT VERIFIED | Fresh Windows/Linux wheels pass 15 tests and 48 native combinations each, four fixture jobs and CLI agreement per platform. Native orphan-heading repair has a failing-baseline regression and 57 focused passes per platform. Six Word save/reopen/export and saved-copy audits pass, with 30 identical page pairs and 48 visible chart placements. Eight earlier outputs, template and 32 source members unchanged. New Word 2019 manual saved copies pass structural audits; user confirms all review/onboarding checks successful. Onboarding timing/participant/assistance logs remain uncollected, omitted production workflows remain unverified; the three earlier Windows layout failures now pass with pinned language data (see remediation). See [continuation](evidence/stage6/CONTINUATION.md). |
| 6.5 | COMPLETE — bounded qualification/disposition | [Closure](evidence/stage6.5/CLOSURE.md): native checks, reproducibility, storage probes and artifact findings recorded. Linux full CI FAIL and release/production HOLD retained as explicit limitations; no full readiness pass claimed. |
| 7 | COMPLETE — bounded feasibility decision | [Assessment and handoff](evidence/stage7/README.md): proceed with bounded prototype limitations; public preview HOLD; do not migrate production yet. Full original Stage 6 gate remains NOT VERIFIED. |
| 8–12 | PLANNED | Production definition, implementation, qualification, rollout and stabilization not started. |

For each future result record: date/reviewer; stage and requirement; PASS/FAIL/NOT VERIFIED; exact upstream revision and patch state; OS/architecture; Rust, Python, tools and lockfiles; fonts/runtime dependencies; exact commands or manual review procedure; exit/results; fixture and template hashes; artifact/log paths; known limitations and next action. Record Word version and open/save/repair findings separately from headless generation. Documentation, mocks, screenshots and Wordcraft's own rendering alone do not satisfy native execution or Word compatibility gates.

## Local revision history

- 5 November 2026, roadmap extension requested: added Stage 6.5 as the immediate qualification/release-preparation priority and Stages 8–12 for internal production migration. Stage 7 remains pending and distinguishes prototype feasibility from production readiness. Documentation change only; no new checks, gate completion, rollout or publication claimed.


- 5 November 2026, Stage 6 bounded review completion: user confirms all Word 2019 review and independent onboarding checks successful. Both new manual saved copies pass bounded structural/media/relationship/unique-ID audits, retain mode 15 and leave generated originals unchanged. Bounded functional scope PASS with user-reported onboarding; detailed onboarding measurements/logs not supplied and no ten-minute result claimed. Full production gate NOT VERIFIED; Stage 7 PENDING, not started. See [results](evidence/stage6/INDEPENDENT_RESULTS.md).


- 5 November 2026, Word 2019 template authorized: user confirmed the two converted manual reports look correct. New mode-15 Lantern template is the default; mode-14 original and historical outputs preserved. Rebuilt fixture wheel passes 15 installed tests and four native/CLI jobs per platform; six Word save/reopen/export audits and 30 identical platform page pairs pass. Contact-sheet review completed; 19 pages differ from old mode-14 exports. Independent review of new outputs/onboarding remain pending; Stage 6 PARTIAL, full production gate NOT VERIFIED, Stage 7 PENDING. See [Word 2019 evidence](evidence/stage6/word2019/README.md).

- 5 November 2026, Stage 6 manual copies received: both converted saved copies pass bounded structural/media/relationship/unique-ID checks and originals remain unchanged. User reports Compatibility Mode and conversion; the canonical template and generated originals specify mode 14, converted copies mode 15. Explicit repair-prompt, post-conversion reopen and visual findings remain pending, as does independent onboarding. Stage 6 remains PARTIAL. See [independent findings](evidence/stage6/INDEPENDENT_RESULTS.md).

- 5 November 2026, Stage 6 evidence handoff: added a full evidence index and independent result form. Fresh read-only integrity verification passes for 242 recorded source files, 147 qualified assets, 32 fixture members, 16 generated originals and two template checks. Neither expected manual review copy is present; independent Word review/onboarding remain PENDING, bounded Stage 6 PARTIAL and full production gate NOT VERIFIED. No technical qualification rerun or Stage 7 work claimed. See [full evidence](evidence/stage6/FULL_EVIDENCE.md).

- 5 November 2026, Stage 6 continuation: user selected bounded fixture qualification and approved implementation. Repaired native heading/table lookahead; rebuilt and executed both installed native wheels and CLIs. Corrected Word review harness passes six save/reopen/export cases and 30 page pairs. Direct raster checks withdraw the earlier blank-chart finding. Independent review and onboarding remain pending; full production gate and Stage 7 remain unverified. See evidence/stage6/CONTINUATION.md.

- 5 November 2026, Stage 6 authorized and implemented for the supplied Lantern fixture: equivalent tagged DOCX, Python-side mapping/overrides, opt-in native entry point, provenance and atomic no-replace staging publication. Final installed Windows/Linux wheels pass 15 tests each and four representative/stress jobs with Rust CLI agreement. Word save/export stalls also affect a prior accepted control; Word/visual/onboarding gates remain pending. Native preview limitations and WSL mounted-filesystem publication restriction recorded. Full production integration and Stage 7 remain unverified; no cutover or publication authorized. See evidence/STAGE6.md.

- 5 November 2026, Stage 5 authorized and implemented: bounded shared repeat/if operations verified with Windows/Linux installed native wheels and CLI. Twelve Word automation cases, saved-copy structural audits and 43 pixel-identical page pairs pass after visual fixture corrections. Gate remains pending independent repair-prompt and saved-copy review. Full Windows engine has 1,220 passes and the inherited proofing failure; Stage 6 has not begun. See evidence/STAGE5.md.

- 5 November 2026, Stage 4 manual review complete: both user-saved representative outputs pass text/style/geometry/media/relationship and unique-ID checks, with generated originals unchanged. User reports no repair prompts for either. Stage 4 gate PASS for the agreed row/chart scope. Stage 5 is eligible; this confirmation does not authorize starting it.

- 5 November 2026, Stage 4 qualification: shared row/chart operations and invoice example executed through installed Windows/Linux CPython 3.12 wheels and CLI. Nine focused Rust tests per platform, hostile-input checks, scalar installed regressions, Word automation and 31 page comparisons pass. Full Windows engine has 1,214 passes and the recorded proofing failure. Gate remains pending independent Word repair-prompt review; no Stage 5/6 implementation began.

- 5 November 2026, Stage 4 start: user authorized row/chart work after Stage 3 passed. Row-wrapper boundary regression passes on Windows for single-row and multi-row controls across repeated saves. Shared resolver and row-insertion gaps identified; row/chart implementation, cross-platform qualification, Word review and invoice example remain pending. See evidence/STAGE4.md.

- 5 November 2026: Stage 2 bounded Rust preservation repairs and two-platform retry recorded; full gate remains pending. See the retry evidence for test-suite failures, baseline comparisons and qualification limits.

- 5 November 2026, user scope clarification: generated Word template accepted as the Stage 2 feature inventory; six-page multi-section fixture verified on both platforms. Actual Jinja2 integration deferred to Stage 6. Only independent repair-dialog review remains as a Stage 2 blocker.

- 5 November 2026, manual review completion: user reported successful interactive save/reopen for Windows and Linux outputs. Saved copies structurally verified and hashed. Stage 2 passes for the agreed representative-template scope; Stage 3 remains unimplemented.

- 5 November 2026, Stage 3 execution: user authorized bindings; native scalar operations and installed Windows/Linux wheels verified. Word automated review and six identical cross-platform page images recorded. Gate remains pending independent Word review and explicit recording/disposition of inherited style normalization; no Stage 4 work began.

- 5 November 2026, saved manual copies: both user-saved `manual-review-native-output.docx` files pass scalar/structure/relationship checks and match the automated Word-resaved structures. Hashes recorded in `evidence/stage3/manual-review-structural-audit.json`. Interactive reopen/prompt findings still await confirmation; inherited style-limit disposition remains pending.

- 5 November 2026, manual prompt confirmation: user reports both saved Stage 3 copies opened with no repair prompts. The independent repair-prompt check passes; the gate remains pending inherited style-limit disposition. No Stage 4 work authorized or begun.

- 5 November 2026, Option 1 repair: four style losses repaired with source-attribute and edit/undo regressions, clean installed Windows/Linux wheels and Word theme/numbering/gallery verification. 24 Word page comparisons pass. FIDELITY-03 closes for the inventoried scope; Stage 3 gate awaits independent prompt review of the new outputs. Existing proofing/layout test failures remain recorded. No Stage 4 work began.

- 5 November 2026, repaired-output review complete: the user supplied both new manual-review-native-output.docx files and reported no repair prompts for either. Both saved copies match the automated Word-resaved structures and retain eight scalar controls, three tables, three sections and unique control IDs. Original outputs are unchanged. Stage 3 gate PASS for the agreed representative scalar-template scope; existing proofing/layout failures and wider qualification limits remain recorded. Stage 4 is eligible, not started or authorized by this confirmation.

- 5 November 2026, Stage 5 manual review complete: both supplied saved copies pass text/style/geometry/media/relationship and unique-ID audits; generated originals are unchanged. User reports no repair prompts for either. Stage 5 gate PASS for the bounded representative block scope. Existing qualification limits remain; Stage 6 is eligible but is not started or authorized by this confirmation.

## Stage 1–6 remediation — 5 November 2026

Approved prototype remediation is recorded in
[evidence/remediation/README.md](evidence/remediation/README.md). Required pinned
language data resolves the four recorded Windows proofing/layout failures;
Linux additionally needs a CJK font for its existing indent fixture. No engine
behavior, public API or report schema change was needed. The repaired build
configuration passes installed-wheel regression qualification on Windows and Linux. Original
bounded PASS scopes stand; clean Windows without Office, measured independent
onboarding and full production qualification remain unverified. Stage 7 has not
started and neither publication nor migration is authorized.

- 5 November 2026, Stage 6.5 execution authorized: added portable offline qualification, strict language-data revision checks, fresh installed-wheel regressions, explicit ignored measurement execution and embedded-data notice repair. Full workspace checks encounter missing inputs; clean deployment, measured onboarding and release clearance remain NOT VERIFIED. Stage 7 not started.

- 5 November 2026, Stage 6.5 final execution record: Windows workspace 5,120 passes and all individual check components pass; Linux workspace 5,056 passes with three desktop UI failures and a separate intermittent aggregate timing failure. Both final installed wheels pass native regressions, 15 Lantern tests and four actual jobs per platform; all 16 DOCX member comparisons match prior outputs. Full Linux CI and external gates remain open. See evidence/stage6.5/README.md and execution-summary.json. No Stage 7 work, release or cutover.

- 6 November 2026, user-authorized scope adjustment: removed clean-host and independent onboarding evidence from Stage 6.5/current bounded-prototype decision gates; deferred both to future adoption testing. Existing results/limitations preserved, other gates unchanged, Stage 6.5 remains IN PROGRESS. Documentation only; no new execution results or release authorization.

- 6 November 2026, Stage 6.5 closed for agreed bounded local prototype qualification and issue disposition under its fixes-or-evidence-backed-limitations gate. Added actual Windows/Linux staging-interruption checks, native Linux permission-denial evidence, dependency source-notice and brand inventories. Full Linux CI failures remain unresolved; distribution and production remain held. See evidence/stage6.5/CLOSURE.md. Stage 7 may begin; not yet executed.

- 6 November 2026, Stage 7 completed by evidence review: bounded prototype feasible with limitations; public preview HOLD and production migration not recommended yet. Added gate/capability assessment, patch and maintenance register, reproducible usage guidance and proposed Stages 8–12 handoff. No new native builds/tests or Word review; no Stage 8 implementation, distribution or cutover.

## Experimental public preview preparation — 6 November 2026

Approved plan: prepare 0.1.0a1 for manual TestPyPI/PyPI publication and a GitHub
prerelease, CPython 3.12 Windows x64 and Linux glibc 2.35+ x64 only.
Production-source access and shadow integration move to adoption testing.
Redistribution review, final installed artifacts, clean-host installation and
independent Word review remain preview gates. No publication or commit performed.
See [preview checklist](../release/PREVIEW.md) for current work and remaining gates.
Historical bounded passes retain their original scope.

Preview preparation results: both optimized native candidates and installed
regressions pass; artifact assembly and strict Twine checks pass. No full CI,
clean-host or independent Word review rerun. Publication remains HOLD.
See [validation](../release/PREVIEW-VALIDATION.md).

6 November 2026: user explicitly authorized review and a local preparation commit.
Windows full cargo xtask ci passes all nine steps after CLI identity test repairs.
Release HOLD, excluded example/handoff files, existing fixture dependencies and
remaining distribution gates are recorded in ../release/PREVIEW-VALIDATION.md.
No push, tag or publication authorized or performed.
