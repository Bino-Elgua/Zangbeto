module zbt::zbt_diagnostics {
    use std::vector;
    use std::option::{Self, Option};
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::hash::blake2b256;
    use zbt::zbt_errors;
    use zbt::zbt_guard;

    // ─────────────────────────────────────────────────────
    // Canonical Constants
    // ─────────────────────────────────────────────────────
    const CODE_PREFIX: vector<u8> = b"OMO-ERR-";
    const MAX_CODE_LEN: u64 = 16;
    const SHA256_LEN: u64 = 32;
    
    // Severity
    const SEV_INFO: u8 = 0;
    const SEV_WARNING: u8 = 1;
    const SEV_ERROR: u8 = 2;
    
    // Category bitmask flags
    const CAT_TYPE: u8 = 1;
    const CAT_LOGIC: u8 = 2;
    const CAT_SECURITY: u8 = 4;
    const CAT_RECEIPT: u8 = 8;
    const CAT_IDENTITY: u8 = 16;
    const CAT_RHYTHM: u8 = 32;
    const VALID_CATEGORIES: u8 = 1 | 2 | 4 | 8 | 16 | 32;
    
    // Status lifecycle
    const STATUS_PENDING: u8 = 0;
    const STATUS_VERIFIED: u8 = 1;
    const STATUS_DISPUTED: u8 = 2;
    const STATUS_FIXED: u8 = 3;
    const STATUS_RISK_ACCEPTED: u8 = 4;

    // ─────────────────────────────────────────────────────
    // Core Receipt Struct
    // ─────────────────────────────────────────────────────
    struct DiagnosticReceipt has key, store {
        id: UID,
        code: vector<u8>,
        severity: u8,
        category: u8,
        message_hash: vector<u8>,
        agent_id: vector<u8>,
        birth_epoch: u64,
        tier: u8,
        sabbath_active: bool,
        repair_id: vector<u8>,
        repair_strategy: u8,
        status: u8,
        fixed_at: Option<u64>,
        zangbeto_sig: vector<u8>,
        submitted_at: u64,
        red_team_round: Option<u64>,
        arweave_tx: Option<vector<u8>>,
        ots_proof: Option<vector<u8>>,

        // Constitutional Receipt Fields
        witness_quorum: vector<vector<u8>>,
        constitutional_class: vector<u8>,
        economic_impact: Option<u64>,
        seal_policy: Option<vector<u8>>,
        sovereign_scope: vector<u8>,
    }

    // ─────────────────────────────────────────────────────
    // Schema Validation
    // ─────────────────────────────────────────────────────
    public fun validate_schema(
        code: &vector<u8>,
        severity: u8,
        category: u8,
        message_hash: &vector<u8>,
        agent_id: &vector<u8>,
    ): bool {
        if (!vector::starts_with(code, &CODE_PREFIX)) { return false };
        if (vector::length(code) > MAX_CODE_LEN) { return false };
        if (severity > SEV_ERROR) { return false };
        if ((category | VALID_CATEGORIES) != VALID_CATEGORIES) { return false };
        if (vector::length(message_hash) != SHA256_LEN) { return false };
        if (vector::length(agent_id) < 32) { return false };
        true
    }

    public fun has_category(category: u8, flag: u8): bool {
        (category & flag) == flag
    }

    // ─────────────────────────────────────────────────────
    // Emit Diagnostic Receipt
    // ─────────────────────────────────────────────────────
    public fun emit_diagnostic(
        ctx: &mut TxContext,
        code: vector<u8>,
        severity: u8,
        category: u8,
        message_hash: vector<u8>,
        agent_id: vector<u8>,
        birth_epoch: u64,
        tier: u8,
        sabbath_active: bool,
        repair_id: vector<u8>,
        repair_strategy: u8,
        red_team_round: Option<u64>,
        witness_quorum: vector<vector<u8>>,
        constitutional_class: vector<u8>,
        economic_impact: Option<u64>,
        seal_policy: Option<vector<u8>>,
        sovereign_scope: vector<u8>,
    ): DiagnosticReceipt {
        assert!(validate_schema(&code, severity, category, &message_hash, &agent_id), zbt_errors::E_SCHEMA_INVALID);
        assert!(repair_strategy >= 1 && repair_strategy <= 3, zbt_errors::E_REPAIR_STRATEGY_INVALID);
        
        let mut payload = vector::empty<u8>();
        vector::append(&mut payload, code);
        vector::append(&mut payload, message_hash);
        vector::append(&mut payload, agent_id);
        let digest = blake2b256(&payload);
        let mock_sig = vector::slice(&digest, 0, 64);

        let receipt = DiagnosticReceipt {
            id: object::new(ctx),
            code, severity, category, message_hash, agent_id, birth_epoch,
            tier, sabbath_active, repair_id, repair_strategy,
            status: STATUS_PENDING,
            fixed_at: option::none(),
            zangbeto_sig: mock_sig,
            arweave_tx: option::none(),
            ots_proof: option::none(),
            submitted_at: tx_context::epoch(ctx),
            red_team_round,
            witness_quorum,
            constitutional_class,
            economic_impact,
            seal_policy,
            sovereign_scope,
        };

        zbt_guard::emit_diagnostic_event(
            object::uid_to_inner(&receipt.id),
            receipt.code,
            receipt.severity,
            receipt.category,
            receipt.agent_id,
            receipt.repair_id,
        );

        receipt
    }

    // ─────────────────────────────────────────────────────
    // Lifecycle Mutations
    // ─────────────────────────────────────────────────────
    public entry fun mark_fixed(
        receipt: &mut DiagnosticReceipt,
        ctx: &mut TxContext,
    ) {
        assert!(receipt.status == STATUS_PENDING || receipt.status == STATUS_VERIFIED, zbt_errors::E_INVALID_TRANSITION);
        receipt.status = STATUS_FIXED;
        receipt.fixed_at = option::some(tx_context::epoch(ctx));
        zbt_guard::emit_fix_event(object::uid_to_inner(&receipt.id), tx_context::epoch(ctx));
    }

    public entry fun mark_disputed(
        receipt: &mut DiagnosticReceipt,
        reason: vector<u8>,
        ctx: &mut TxContext,
    ) {
        assert!(receipt.status == STATUS_PENDING, zbt_errors::E_INVALID_TRANSITION);
        receipt.status = STATUS_DISPUTED;
        zbt_guard::emit_dispute_event(object::uid_to_inner(&receipt.id), reason);
    }

    // ─────────────────────────────────────────────────────
    // Query Helpers
    // ─────────────────────────────────────────────────────
    public fun get_code(r: &DiagnosticReceipt): &vector<u8> { &r.code }
    public fun get_severity(r: &DiagnosticReceipt): u8 { r.severity }
    public fun get_category(r: &DiagnosticReceipt): u8 { r.category }
    public fun get_agent_id(r: &DiagnosticReceipt): &vector<u8> { &r.agent_id }
    public fun get_repair_id(r: &DiagnosticReceipt): &vector<u8> { &r.repair_id }
    public fun get_status(r: &DiagnosticReceipt): u8 { r.status }
    pub fun get_signature(r: &DiagnosticReceipt): &vector<u8> { &r.zangbeto_sig }
    pub fun get_red_team_round(r: &DiagnosticReceipt): &Option<u64> { &r.red_team_round }
    
    // Constants for status (needed by core)
    public fun status_pending(): u8 { STATUS_PENDING }
    public fun status_verified(): u8 { STATUS_VERIFIED }
    public fun status_disputed(): u8 { STATUS_DISPUTED }
    public fun status_fixed(): u8 { STATUS_FIXED }
    public fun status_risk_accepted(): u8 { STATUS_RISK_ACCEPTED }
}
