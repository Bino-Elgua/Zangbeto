module app::payments {
    use sui::tx_context::TxContext;
    use sui::object::UID;
    use zbt::zbt_guard;

    struct Treasury has key {
        id: UID,
        balance: u64,
    }

    public fun settle_payment(t: &mut Treasury, amount: u64, _ctx: &mut TxContext) {
        if (t.balance < amount) {
             zbt_guard::invariant_true(false, 1001);
        };
        t.balance = t.balance - amount;
    }
}
