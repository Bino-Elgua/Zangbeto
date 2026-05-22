# Formal Enforcement Semantics

## 1. Anomaly Classification
Anomalies are detected by comparing reality transitions against deterministic models.
- **Replay Mismatch**: Execution trace differs from replayed trace.
- **CRDT Conflict**: Merge lattice reveals concurrent or divergent state.
- **Policy Violation**: Pluggable Orisha policies (WASM) reject a transition.
- **Invariants**: Economic or temporal laws are broken.

## 2. Action Escalation Ladder
Responses to anomalies follow a deterministic ladder:
1. **Observe**: Log and retain evidence.
2. **Flag**: Notify Orisha for review.
3. **Quarantine**: Isolate affected state subtrees to prevent corruption.
4. **Rollback**: Revert state to the last known valid hash.
5. **Punish**: Slashes, revocations, or suspensions for malicious agents.
6. **Halt**: Emergency stop of the execution engine.

## 3. Quarantine Semantics
Quarantined state is isolated from the main merge lattice.
- **Merge Blocklist**: Prevents reintegration of divergent hashes.
- **Release Conditions**: Quarantines are only released via time, consensus, or resolution proofs.

## 4. Arbitration Rules
Conflict resolution between Orisha is handled via:
- **Weighted Vote**: Influence based on Orisha reputation.
- **Hierarchical**: Superior Orisha (e.g. Ṣàngó) overrides subordinates.
- **Consensus**: Multi-party quorum required for critical actions.
