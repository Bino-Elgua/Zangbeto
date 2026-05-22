# 🌀 Zàngbétò Master Blueprint: The Self-Healing Loop

This document serves as the definitive technical guide and actionable roadmap for the Zàngbétò project. It synthesizes the requirements from `read first` and `todo` into a single, executable blueprint for AI agents.

## 🏛️ 1. Architecture Vision
Zàngbétò is a **ritual-driven immune system** for smart contracts. It operates as a feedback loop between "Red Team" (anomaly detection) and "Blue Team" (automated repair) agents, with the Sui blockchain acting as the **Immutable Witness (The Shrine)**.

### Core Pillars
1.  **Structured Diagnostics (`OMO-ERR`):** Every failure, regardless of language (Rust, Move, Python, Julia), is emitted as machine-readable JSON.
2.  **The Heartbeat:** A continuous cycle of detection, proposal, and auto-repair.
3.  **Guardian Enforcement:** On-chain validation of diagnostic schemas and agent reputation.
4.  **Ritual Audit:** Weekly "Sabbath" cycles for anchoring data to Arweave and sealing the ledger with OpenTimestamps.

---

## 🛠️ 2. Component Breakdown

### A. The Shrine (Sui Move)
- **`zbt_diagnostics`**: Schema enforcement, signature verification, and lifecycle management of `DiagnosticReceipt`.
- **`zbt_core`**: The typed ledger, agent reputation tracking, and indexing for efficient querying.
- **`zbt_guard`**: Event emission for off-chain listeners (Night Patrol).

### B. The Steward (Rust Core)
- **`omo-diagnostic`**: The canonical crate defining the JSON schema used by all components.
- **`DiagnosticHandler`**: The logic gate that decides if a bug can be auto-merged or requires human intervention.
- **`Heartbeat Engine`**: The orchestrator of Red/Blue agent rounds.

### C. The Veils (Emission Helpers)
- **Python/Julia/Move**: Utilities that allow any part of the system to scream "I'm broken" in a way the Steward understands.

---

## 📋 3. Actionable Checklist

### Phase 1: Foundation & Schema (Week 1)
- [x] **Align Move Modules:** Update `shrine/sources/` with the final reconciled code from `read first`.
    - [x] Update `zbt_diagnostics.move` (Schema validation + Bitmasks).
    - [x] Update `zbt_core.move` (Typed ledger + Agent indexing).
    - [x] Update `zbt_errors.move` (New 9xxx error codes).
- [x] **Rust Crate Setup:** Initialize `crates/omo-diagnostic`.
    - [x] Define Enums: `Severity`, `Category`, `RepairStrategy`, `Status`.
    - [x] Define Structs: `Diagnostic`, `RepairPlan`, `AuditTrail`.
    - [x] Implement `to_zangbeto_payload()` for Move compatibility.

### Phase 2: Cross-Language Veils (Week 1-2)
- [x] **Python Veil:** Finalize `omo_diagnostic.py` and add `requirements.txt`.
- [x] **Julia Veil:** Finalize `Diagnostic.jl`.
- [x] **Move Test-Emission:** Create a test Move module that emits a structured diagnostic.
- [x] **Validation:** Run `make test-emission` to verify all helpers output identical JSON structures.

### Phase 3: The Heartbeat Engine (Week 2-3)
- [x] **Steward Heartbeat:** Implement the `steward-heartbeat.rs` binary.
    - [x] Red Agent: Random/Scenario-based anomaly generation.
    - [x] Blue Agent: Patch generation and validation.
    - [x] Safety Gate: Implement `should_auto_merge()` logic.
- [x] **Zangbeto Client:** Implement the Rust client for Sui interaction (signing + submission).
- [x] **Sandbox:** Ensure repairs run in isolated environments (Git branches/containers).

### Phase 4: Integration & Rituals (Week 3-4)
- [x] **Native Event Listener:** Implement a Rust-based Sui event listener within the Steward.
    - [x] Subscribe to `DiagnosticEmitted` and `DiagnosticSubmittedEvent`.
    - [x] Trigger the Heartbeat Engine automatically on new diagnostics.
- [x] **Sabbath Ritual:** Automate the checklist in `ops/sabbath_checklist.md`.
    - [x] Local collection and deduplication of the week's receipts.
    - [x] Summary generation and local archival.
- [x] **Verification:** Run `make test-full-flow` to verify the streamlined loop.

---

## 🔐 4. Safety Mandates for Agents
1.  **Never Auto-Merge Security:** Any diagnostic with `Category::Security` or `Category::Identity` MUST have `RepairStrategy::Manual`.
2.  **Reputation Check:** Before acting on a repair, verify the emitting agent's `tier` and `reputation_ledger` status.
3.  **Signature Required:** In production, never trust a diagnostic that isn't signed by a Zàngbétò Guardian.
4.  **Immutable Logs:** Always anchor the final state of a repair (including diffs) to Arweave.

---

## 🕯️ 5. Ritual Notes
*   **The Sabbath** is not just for humans; it is when the system clears its caches, rotates its keys, and "seals" its history.
*   **The Orishas** (Èṣù, Ògún, etc.) represent different types of monitoring (Network, Logic, Security). Ensure your diagnostics tag the correct Orisha for better routing.

*Àṣẹ.*
