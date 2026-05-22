module zbt::test_diagnostic {
    use sui::tx_context::TxContext;
    use std::option;
    use zbt::zbt_core;

    public entry fun test_emit(ledger: &mut zbt_core::DiagnosticLedger, ctx: &mut TxContext) {
        zbt_core::submit_diagnostic(
            ledger,
            ctx,
            b"OMO-ERR-001",
            0, // Info
            1, // Type
            x"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            x"0000000000000000000000000000000000000000000000000000000000000000",
            0,
            1,
            false,
            b"fix-test",
            1, // Auto
            option::none(),
            std::vector::empty(),
            b"general",
            option::none(),
            option::none(),
            b"global",
        );
    }
}
