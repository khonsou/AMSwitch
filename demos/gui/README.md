# GUI Interactive Mock

**English** | [简体中文](README.zh-CN.md)

A single-file H5 demo — open `index.html` in any browser and every button works (all data is simulated).

The UI is bilingual: toggle `EN/中` in the title bar, persisted across reloads. Dynamic logs and mock data (profile names, game titles) are bilingual too — overseas reviewers can walk the full flow in English.

Interactions covered: the two-layer rule model (game → each device switches to its assigned onboard profile), fall back to default on game exit, offline devices silently skipped and re-switched when plugged in, adding a game (three-layer sources + exe dedupe), live re-switch when editing the active rule, deleting rules (always-visible ✕ on the row head, two-step inline confirm; deleting the active rule falls back to default immediately), one-click diagnostics copy.

A four-step guide sits at the top of the page; the annotation column on the right maps each design decision to its PRD clause.
