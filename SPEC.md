# Reality VM v4 — Formal Execution Semantics

## 1. Canonical Encoding Contract

All consensus-critical data MUST be encoded via `CanonicalEncode`:
- CBOR with `sort_map(true)`
- No floating-point rounding variance
- Deterministic field ordering via `BTreeMap`
- Hash = SHA-256(canonical_bytes)

## 2. State Algebra

State S = (I, M, E, C, V) where:
- I: Identity (immutable)
- M: Memory (CRDT: LWW-Map ⊕ OR-Set ⊕ G-Counter)
- E: Economy (i64 balance, f64 reputation ∈ [0,1])
- C: Constraints (finite set of boolean predicates)
- V: VectorClock (causality tracking)

Merge operator ⊔: S × S → S
- I: equality required (else conflict)
- M: CRDT join (associative, commutative, idempotent)
- E: economic invariants enforced (no negative balance)
- C: intersection (stricter constraints win)
- V: component-wise max

## 3. Deterministic Execution

Execution function:
```
exec : S × IR* → S × Trace
```
Must satisfy:
- **Purity**: exec(s, ops) = exec(s, ops) ∀ s, ops (no side effects)
- **Replayability**: verify_trace(exec(s, ops)) = true
- **Monotonicity**: vector_clock(exec(s, ops)) ≥ vector_clock(s)

## 4. Validator Consensus

Consensus function:
```
consensus : S × IR* × {Orisha} → {Approved, Rejected, Conditional}
```
With temporal weights:
```
weight(o, t) = base(o) × decay(t - t₀) × trust_history(o)
```
Stability constraint:
```
|weight(o, t+1) - weight(o, t)| ≤ δ_max  // damping
```

## 5. Capability Security

Token verification:
```
verify(token, registry) ⇔ 
  sig_valid(token.sig, token.payload_hash, registry[token.signer_key_id]) ∧
  payload_hash = SHA256(canonical_encode(token.fields)) ∧
  now ≤ token.expiry
```

## 6. Branch Merge Semantics

Branch merge: B × B → B ∪ {Conflict}
- If vector_clocks concurrent: invoke Orisha arbitration policy
- If one happens-before: fast-forward
- Conflict resolution: deterministic ordering via (orisha_weight, timestamp, node_id)

## 7. Invariants

Global invariants (must hold ∀ transitions):
1. `economy.balance ≥ 0`
2. `economy.reputation ∈ [0.0, 1.0]`
3. `memory.private_seal ⇒ capability_token.has_access(path)`
4. `vector_clock.monotonic()`
5. `state_hash = SHA256(canonical_encode(state))`

Violation → automatic rollback + Zàngbétò audit event.
