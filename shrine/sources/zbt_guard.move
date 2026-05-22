module zbt::zbt_guard {
    use sui::event;
    use sui::tx_context::TxContext;
    use sui::object::{ID};

    // ─────────────────────────────────────────────────────
    // Event Structs
    // ─────────────────────────────────────────────────────
    struct InvariantBreach has drop, store { rule: vector<u8>, code: u64 }
    
    struct DiagnosticEmitted has drop, store {
        receipt_id: ID,
        code: vector<u8>,
        severity: u8,
        category: u8,
        agent_id: vector<u8>,
        repair_id: vector<u8>,
    }

    struct AutoRepairExecuted has drop, store {
        receipt_id: ID,
        repair_id: vector<u8>,
        result: bool,
        gas_used: u64,
    }

    struct FixEvent has drop, store {
        receipt_id: ID,
        fixer: vector<u8>,
        epoch: u64
    }

    struct DisputeEvent has drop, store {
        receipt_id: ID,
        reason: vector<u8>
    }

    struct DiagnosticSubmittedEvent has drop, store {
        receipt_id: ID,
        code: vector<u8>,
        severity: u8,
        category: u8,
        agent_id: vector<u8>,
        repair_id: vector<u8>,
        red_team_round: sui::option::Option<u64>,
    }

    struct FixedEvent has drop, store {
        receipt_id: ID,
        fixer_agent: vector<u8>,
        bounty_amount: u64
    }

    // ─────────────────────────────────────────────────────
    // Emission Helpers
    // ─────────────────────────────────────────────────────
    public fun invariant_true(cond: bool, code: u64) {
        if (!cond) {
            event::emit(InvariantBreach{ rule: b"runtime", code });
            abort code;
        }
    }

    public fun emit_diagnostic_event(
        receipt_id: ID,
        code: vector<u8>,
        severity: u8,
        category: u8,
        agent_id: vector<u8>,
        repair_id: vector<u8>,
    ) {
        event::emit(DiagnosticEmitted {
            receipt_id,
            code,
            severity,
            category,
            agent_id,
            repair_id,
        });
    }

    public fun emit_auto_repair_event(
        receipt_id: ID,
        repair_id: vector<u8>,
        result: bool,
        gas_used: u64,
    ) {
        event::emit(AutoRepairExecuted {
            receipt_id,
            repair_id,
            result,
            gas_used,
        });
    }

    public fun emit_fix_event(receipt_id: ID, epoch: u64) {
        event::emit(FixEvent { receipt_id, fixer: b"guardian", epoch });
    }

    public fun emit_dispute_event(receipt_id: ID, reason: vector<u8>) {
        event::emit(DisputeEvent { receipt_id, reason });
    }

    public fun emit_diagnostic_submitted_event(
        receipt_id: ID,
        code: vector<u8>,
        severity: u8,
        category: u8,
        agent_id: vector<u8>,
        repair_id: vector<u8>,
        red_team_round: sui::option::Option<u64>,
    ) {
        event::emit(DiagnosticSubmittedEvent {
            receipt_id,
            code,
            severity,
            category,
            agent_id,
            repair_id,
            red_team_round,
        });
    }

    public fun emit_fixed_event(
        receipt_id: ID,
        fixer_agent: vector<u8>,
        bounty_amount: u64,
    ) {
        event::emit(FixedEvent {
            receipt_id,
            fixer_agent,
            bounty_amount,
        });
    }
}
