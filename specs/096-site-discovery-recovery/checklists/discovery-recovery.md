# Discovery And Recovery Requirements Checklist: Site Discovery And Recovery

**Purpose**: Validate that S096 defines complete, precise, and measurable requirements for current-first search and exported not-found recovery

**Created**: 2026-08-28

**Feature**: [spec.md](../spec.md)

**Note**: This checklist evaluates the requirements themselves, not the implementation.

## Requirement Completeness

- [x] CHK001 Are both audited retired-command queries named explicitly? [Completeness, Spec §FR-001 to §FR-002]
- [x] CHK002 Is the required current guidance defined as replacement instruction rather than merely a non-changelog result? [Completeness, Spec §FR-003]
- [x] CHK003 Is preservation of matching historical results required for both queries? [Completeness, Spec §FR-004]
- [x] CHK004 Are the established current-guidance queries included as regression subjects? [Completeness, Spec §FR-005]
- [x] CHK005 Are not-found identity, explanation, status, landmark, links, responsiveness, and error-free hydration all specified? [Completeness, Spec §FR-007 to §FR-012]
- [x] CHK006 Is the durable audit update required for both remaining findings without rewriting prior evidence? [Completeness, Spec §FR-014]

## Requirement Clarity

- [x] CHK007 Is successful search ordering defined as the first result rather than an unspecified prominent position? [Clarity, Spec §FR-001 to §FR-002]
- [x] CHK008 Are historical results required to follow current guidance without being removed from search? [Clarity, Spec §FR-004]
- [x] CHK009 Is production-equivalent search verification distinguished from source-text inspection? [Clarity, Spec §FR-006]
- [x] CHK010 Is the not-found response status fixed at HTTP 404 while the page content is replaced? [Clarity, Spec §FR-010]
- [x] CHK011 Are both recovery URLs and their labelling requirement explicit? [Clarity, Spec §FR-009]
- [x] CHK012 Is the primary landmark count and identity unambiguous? [Clarity, Spec §FR-008]

## Requirement Consistency

- [x] CHK013 Do current-first ranking and preserved release history coexist without contradictory suppression requirements? [Consistency, Spec §FR-001 to §FR-005]
- [x] CHK014 Does the not-found landmark requirement align with the existing shared bypass destination? [Consistency, Spec §FR-008, Edge Cases]
- [x] CHK015 Does the custom not-found page remain outside the fixed 54-route public inventory? [Consistency, Spec §SC-007, Assumptions]
- [x] CHK016 Do correction requirements preserve current documentation and both product modes? [Consistency, Spec §FR-013]

## Acceptance Criteria Quality

- [x] CHK017 Can search ordering and historical-result retention be measured for both exact queries? [Measurability, Spec §SC-001 to §SC-002]
- [x] CHK018 Can current-query relevance be evaluated against the four named baseline queries? [Measurability, Spec §SC-003]
- [x] CHK019 Can the not-found status, landmark, destinations, overflow, and keyboard reachability be measured at named viewports? [Measurability, Spec §SC-004 to §SC-005]
- [x] CHK020 Is regression sensitivity required for each corrected failure class? [Acceptance Criteria, Spec §SC-006]

## Scenario And Edge-Case Coverage

- [x] CHK021 Are repeated historical matches, case and whitespace variants, preserved history, and overlapping current queries addressed? [Coverage, Edge Cases]
- [x] CHK022 Are nested absent paths, generated-route lookalikes, shared skip targets, and hydration failures addressed? [Coverage, Edge Cases]
- [x] CHK023 Are primary, alternate, exception, recovery, and non-functional scenarios all represented? [Coverage, User Stories, Edge Cases]

## Scope And Evidence Boundaries

- [x] CHK024 Is visual redesign excluded in favor of existing brand and theme assets? [Scope, Assumptions]
- [x] CHK025 Are changelog content, public routes, navigation, and runtime product behavior protected from incidental changes? [Scope, Spec §FR-013]
- [x] CHK026 Are native assistive-technology claims excluded from the automated evidence boundary? [Evidence, Assumptions]
