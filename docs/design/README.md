# PDF Tile Viewer Dioxus + Embedded PDFium Goal-State Design Package

**Date:** 2026-06-07  
**Project:** PDF Tile Viewer  
**Target migration:** Tauri + Svelte + PDF.js + externally located PDFium dynamic library → Dioxus Desktop + Rust-first PDFium rendering + internally managed PDFium distribution  
**Purpose:** Detailed external design before implementation RFC creation

## Documents

| File | Purpose |
|---|---|
| `00_adr_migration_strategy.md` | Strategic decision record for the migration direction and constraints. |
| `01_goal_state_external_design.md` | Main detailed external design: UI, workflows, architecture, data model, lifecycles, packaging, and acceptance criteria. |
| `02_migration_roadmap.md` | Phased migration plan with spike gates, milestones, risks, and rollback points. |
| `03_rfc_roadmap.md` | RFC themes and recommended sequencing derived from the external design. |

## Reading Order

1. Read `00_adr_migration_strategy.md` to confirm the architectural decision.
2. Read `01_goal_state_external_design.md` as the main design contract.
3. Use `02_migration_roadmap.md` to plan development phases.
4. Use `03_rfc_roadmap.md` to create detailed implementation RFCs after the design is accepted.

## Status

This package is intentionally **pre-RFC**. It is detailed enough to serve as a target design, but individual RFCs should still be written before implementation begins.
