use crate::guest_error::set_error_and_halt;
use hyperlight_common::flatbuffer_wrappers::guest_error::ErrorCode;
use core::sync::atomic::{AtomicUsize, Ordering};

// This is a simplified implementation of Control Flow Guard functionality.
// This would normally validate that the target address is a valid function
// but for now we will just ensure the target address is not null.
#[no_mangle]
pub static __guard_dispatch_icall_fptr: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
pub unsafe fn initialize_cfg_dispatch() {
    __guard_dispatch_icall_fptr.store(__guard_dispatch_icall as usize, Ordering::Relaxed);
}

#[no_mangle]
extern "C" fn __guard_dispatch_icall(
    target_function: *const u8,
    _dispatch_target: *const u8,
) -> *const u8 {
    if target_function.is_null() {
        set_error_and_halt(ErrorCode::ControlFlowGuardInvalidFunction, "Control Flow Guard: Null function pointer called");
    }

    // For now just return the target function address.
    // A full implementation here would verify that the target_function
    // is in a table of valid function entry points

    target_function
}
