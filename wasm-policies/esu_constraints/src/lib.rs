#[no_mangle]
pub extern "C" fn validate(ptr: u32, len: u32) -> i32 {
    // In reality, would deserialize state/ops/context and return result code
    1 // Approved
}
