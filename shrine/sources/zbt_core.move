module zbt::zbt_core {
    use std::vector;
    use std::option::{Self, Option};
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::table::{Self, Table};
    use sui::transfer;
    use zbt::zbt_diagnostics::{Self as diag, DiagnosticReceipt};
    use zbt::zbt_errors;
    use zbt::zbt_guard;

    // ─────────────────────────────────────────────────────
    // Typed Diagnostic Ledger with Indexing
    // ─────────────────────────────────────────────────────
    struct DiagnosticLedger has key {
        id: UID,
        // receipt_id -> DiagnosticReceipt
        table: Table<sui::object::ID, DiagnosticReceipt>,
        // agent_id -> list of receipt_ids
        agent_index: Table<vector<u8>, vector<sui::object::ID>>,
        // category_bit -> list of receipt_ids (for analytics)
        category_index: Table<u8, vector<sui::object::ID>>,
        // red_team_round -> list of receipt_ids (for heartbeat tracking)
        round_index: Table<u64, vector<sui::object::ID>>,
    }

    // ─────────────────────────────────────────────────────
    // Reputation & Bounty Tracking
    // ─────────────────────────────────────────────────────
    struct AgentReputation has key, store {
        id: UID,
        agent_id: vector<u8>,
        birth_epoch: u64,
        total_receipts: u64,
        fixed_count: u64,
        disputed_count: u64,
        bounty_balance: u64,
        last_active: u64,
    }

    struct ReputationLedger has key {
        id: UID,
        table: Table<vector<u8>, AgentReputation>,
    }

    // ─────────────────────────────────────────────────────
    // Initialization (One-Time Setup)
    // ─────────────────────────────────────────────────────
    public entry fun init(ctx: &mut TxContext) {
        let diag_ledger = DiagnosticLedger {
            id: object::new(ctx),
            table: table::new(ctx),
            agent_index: table::new(ctx),
            category_index: table::new(ctx),
            round_index: table::new(ctx),
        };
        transfer::share_object(diag_ledger);

        let rep_ledger = ReputationLedger {
            id: object::new(ctx),
            table: table::new(ctx),
        };
        transfer::share_object(rep_ledger);
    }

    // ─────────────────────────────────────────────────────
    // Submit Diagnostic to Typed Ledger (Canonical Entry)
    // ─────────────────────────────────────────────────────
    public entry fun submit_diagnostic(
        ledger: &mut DiagnosticLedger,
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
    ) {
        assert!(diag::validate_schema(&code, severity, category, &message_hash, &agent_id), zbt_errors::E_SCHEMA_INVALID);
        
        let receipt = diag::emit_diagnostic(
            ctx, code, severity, category, message_hash, agent_id,
            birth_epoch, tier, sabbath_active, repair_id, repair_strategy, red_team_round,
            witness_quorum, constitutional_class, economic_impact, seal_policy, sovereign_scope
        );
        let receipt_id = object::id(&receipt);

        // Index by agent
        if (!table::contains(&ledger.agent_index, agent_id)) {
            table::add(&mut ledger.agent_index, agent_id, vector::empty());
        };
        let agent_receipts = table::borrow_mut(&mut ledger.agent_index, agent_id);
        vector::push_back(agent_receipts, receipt_id);

        // Index by category bitmask
        let bit = 1;
        while (bit <= 32) {
            if ((category & bit) == bit) {
                if (!table::contains(&ledger.category_index, bit)) {
                    table::add(&mut ledger.category_index, bit, vector::empty());
                };
                let cat_receipts = table::borrow_mut(&mut ledger.category_index, bit);
                vector::push_back(cat_receipts, receipt_id);
            };
            bit = bit << 1;
        };

        // Index by red_team_round
        if (option::is_some(&red_team_round)) {
            let round = *option::borrow(&red_team_round);
            if (!table::contains(&ledger.round_index, round)) {
                table::add(&mut ledger.round_index, round, vector::empty());
            };
            let round_receipts = table::borrow_mut(&mut ledger.round_index, round);
            vector::push_back(round_receipts, receipt_id);
        };

        zbt_guard::emit_diagnostic_submitted_event(
            receipt_id,
            *diag::get_code(&receipt),
            diag::get_severity(&receipt),
            diag::get_category(&receipt),
            *diag::get_agent_id(&receipt),
            *diag::get_repair_id(&receipt),
            *diag::get_red_team_round(&receipt),
        );

        table::add(&mut ledger.table, receipt_id, receipt);
    }

    // ─────────────────────────────────────────────────────
    // Query Helpers
    // ─────────────────────────────────────────────────────
    public fun query_by_agent(
        ledger: &DiagnosticLedger,
        agent_id: vector<u8>,
        _limit: u64,
    ): vector<sui::object::ID> {
        if (!table::contains(&ledger.agent_index, agent_id)) { return vector::empty() };
        *table::borrow(&ledger.agent_index, agent_id)
    }

    // ─────────────────────────────────────────────────────
    // Reputation Management
    // ─────────────────────────────────────────────────────
    public entry fun register_agent(
        rep_ledger: &mut ReputationLedger,
        agent_id: vector<u8>,
        birth_epoch: u64,
        ctx: &mut TxContext,
    ) {
        if (!table::contains(&rep_ledger.table, agent_id)) {
            let rep = AgentReputation {
                id: object::new(ctx),
                agent_id,
                birth_epoch,
                total_receipts: 0,
                fixed_count: 0,
                disputed_count: 0,
                bounty_balance: 0,
                last_active: tx_context::epoch(ctx),
            };
            table::add(&mut rep_ledger.table, agent_id, rep);
        };
    }

    public entry fun mark_fixed_and_reward(
        diag_ledger: &mut DiagnosticLedger,
        rep_ledger: &mut ReputationLedger,
        receipt_id: sui::object::ID,
        fixer_agent: vector<u8>,
        bounty_amount: u64,
        ctx: &mut TxContext,
    ) {
        assert!(table::contains(&diag_ledger.table, receipt_id), zbt_errors::E_RECEIPT_NOT_FOUND);
        
        let receipt = table::borrow_mut(&mut diag_ledger.table, receipt_id);
        diag::mark_fixed(receipt, ctx);
        
        if (table::contains(&rep_ledger.table, fixer_agent)) {
            let rep = table::borrow_mut(&mut rep_ledger.table, fixer_agent);
            rep.fixed_count = rep.fixed_count + 1;
            rep.bounty_balance = rep.bounty_balance + bounty_amount;
            rep.last_active = tx_context::epoch(ctx);
        };
        
        zbt_guard::emit_fixed_event(receipt_id, fixer_agent, bounty_amount);
    }
}
