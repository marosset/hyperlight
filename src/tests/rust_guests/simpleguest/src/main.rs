/*
Copyright 2025  The Hyperlight Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

#![no_std]
#![no_main]
const DEFAULT_GUEST_STACK_SIZE: i32 = 65536; // default stack size
const MAX_BUFFER_SIZE: usize = 1024;
// ^^^ arbitrary value for max buffer size
// to support allocations when we'd get a
// stack overflow. This can be removed once
// we have proper stack guards in place.

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{format, vec};
use core::ffi::c_char;
use core::hint::black_box;
use core::ptr::write_volatile;

use hyperlight_common::flatbuffer_wrappers::function_call::{FunctionCall, FunctionCallType};
use hyperlight_common::flatbuffer_wrappers::function_types::{
    ParameterType, ParameterValue, ReturnType,
};
use hyperlight_common::flatbuffer_wrappers::guest_error::ErrorCode;
use hyperlight_common::flatbuffer_wrappers::guest_log_level::LogLevel;
use hyperlight_common::flatbuffer_wrappers::util::get_flatbuffer_result;
use hyperlight_common::mem::PAGE_SIZE;
use hyperlight_guest::error::{HyperlightGuestError, Result};
use hyperlight_guest::exit::{abort_with_code, abort_with_code_and_message};
use hyperlight_guest_bin::guest_function::definition::GuestFunctionDefinition;
use hyperlight_guest_bin::guest_function::register::register_function;
use hyperlight_guest_bin::host_comm::{
    call_host_function, call_host_function_without_returning_result, read_n_bytes_from_user_memory,
};
use hyperlight_guest_bin::memory::malloc;
use hyperlight_guest_bin::{MIN_STACK_ADDRESS, guest_logger};
use log::{LevelFilter, error};

extern crate hyperlight_guest;

static mut BIGARRAY: [i32; 1024 * 1024] = [0; 1024 * 1024];

fn set_static(_: &FunctionCall) -> Result<Vec<u8>> {
    unsafe {
        #[allow(static_mut_refs)]
        for val in BIGARRAY.iter_mut() {
            *val = 1;
        }
        #[allow(static_mut_refs)]
        Ok(get_flatbuffer_result(BIGARRAY.len() as i32))
    }
}

fn echo_double(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Double(value) = function_call.parameters.clone().unwrap()[0].clone() {
        Ok(get_flatbuffer_result(value))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to echo_double".to_string(),
        ))
    }
}

fn echo_float(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Float(value) = function_call.parameters.clone().unwrap()[0].clone() {
        Ok(get_flatbuffer_result(value))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to echo_float".to_string(),
        ))
    }
}

fn print_output(message: &str) -> Result<Vec<u8>> {
    let res = call_host_function::<i32>(
        "HostPrint",
        Some(Vec::from(&[ParameterValue::String(message.to_string())])),
        ReturnType::Int,
    )?;

    Ok(get_flatbuffer_result(res))
}

fn simple_print_output(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::String(message) = function_call.parameters.clone().unwrap()[0].clone() {
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to simple_print_output".to_string(),
        ))
    }
}

fn set_byte_array_to_zero(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::VecBytes(mut vec) = function_call.parameters.clone().unwrap()[0].clone()
    {
        vec.fill(0);
        Ok(get_flatbuffer_result(&*vec))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to set_byte_array_to_zero".to_string(),
        ))
    }
}

fn print_two_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (ParameterValue::String(arg1), ParameterValue::Int(arg2)) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
    ) {
        let message = format!("Message: arg1:{} arg2:{}.", arg1, arg2);
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_two_args".to_string(),
        ))
    }
}

fn print_three_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (ParameterValue::String(arg1), ParameterValue::Int(arg2), ParameterValue::Long(arg3)) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
    ) {
        let message = format!("Message: arg1:{} arg2:{} arg3:{}.", arg1, arg2, arg3);
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_three_args".to_string(),
        ))
    }
}

fn print_four_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
    ) {
        let message = format!(
            "Message: arg1:{} arg2:{} arg3:{} arg4:{}.",
            arg1, arg2, arg3, arg4
        );
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_four_args".to_string(),
        ))
    }
}

fn print_five_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
        ParameterValue::String(arg5),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
        function_call.parameters.clone().unwrap()[4].clone(),
    ) {
        let message = format!(
            "Message: arg1:{} arg2:{} arg3:{} arg4:{} arg5:{}.",
            arg1, arg2, arg3, arg4, arg5
        );
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_five_args".to_string(),
        ))
    }
}

fn print_six_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
        ParameterValue::String(arg5),
        ParameterValue::Bool(arg6),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
        function_call.parameters.clone().unwrap()[4].clone(),
        function_call.parameters.clone().unwrap()[5].clone(),
    ) {
        let message = format!(
            "Message: arg1:{} arg2:{} arg3:{} arg4:{} arg5:{} arg6:{}.",
            arg1, arg2, arg3, arg4, arg5, arg6
        );
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_six_args".to_string(),
        ))
    }
}

fn print_seven_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
        ParameterValue::String(arg5),
        ParameterValue::Bool(arg6),
        ParameterValue::Bool(arg7),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
        function_call.parameters.clone().unwrap()[4].clone(),
        function_call.parameters.clone().unwrap()[5].clone(),
        function_call.parameters.clone().unwrap()[6].clone(),
    ) {
        let message = format!(
            "Message: arg1:{} arg2:{} arg3:{} arg4:{} arg5:{} arg6:{} arg7:{}.",
            arg1, arg2, arg3, arg4, arg5, arg6, arg7
        );
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_seven_args".to_string(),
        ))
    }
}

fn print_eight_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
        ParameterValue::String(arg5),
        ParameterValue::Bool(arg6),
        ParameterValue::Bool(arg7),
        ParameterValue::UInt(arg8),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
        function_call.parameters.clone().unwrap()[4].clone(),
        function_call.parameters.clone().unwrap()[5].clone(),
        function_call.parameters.clone().unwrap()[6].clone(),
        function_call.parameters.clone().unwrap()[7].clone(),
    ) {
        let message = format!(
            "Message: arg1:{} arg2:{} arg3:{} arg4:{} arg5:{} arg6:{} arg7:{} arg8:{}.",
            arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8
        );
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_eight_args".to_string(),
        ))
    }
}

fn print_nine_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
        ParameterValue::String(arg5),
        ParameterValue::Bool(arg6),
        ParameterValue::Bool(arg7),
        ParameterValue::UInt(arg8),
        ParameterValue::ULong(arg9),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
        function_call.parameters.clone().unwrap()[4].clone(),
        function_call.parameters.clone().unwrap()[5].clone(),
        function_call.parameters.clone().unwrap()[6].clone(),
        function_call.parameters.clone().unwrap()[7].clone(),
        function_call.parameters.clone().unwrap()[8].clone(),
    ) {
        let message = format!(
            "Message: arg1:{} arg2:{} arg3:{} arg4:{} arg5:{} arg6:{} arg7:{} arg8:{} arg9:{}.",
            arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9
        );
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_nine_args".to_string(),
        ))
    }
}

fn print_ten_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
        ParameterValue::String(arg5),
        ParameterValue::Bool(arg6),
        ParameterValue::Bool(arg7),
        ParameterValue::UInt(arg8),
        ParameterValue::ULong(arg9),
        ParameterValue::Int(arg10),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
        function_call.parameters.clone().unwrap()[4].clone(),
        function_call.parameters.clone().unwrap()[5].clone(),
        function_call.parameters.clone().unwrap()[6].clone(),
        function_call.parameters.clone().unwrap()[7].clone(),
        function_call.parameters.clone().unwrap()[8].clone(),
        function_call.parameters.clone().unwrap()[9].clone(),
    ) {
        let message = format!(
            "Message: arg1:{} arg2:{} arg3:{} arg4:{} arg5:{} arg6:{} arg7:{} arg8:{} arg9:{} arg10:{}.",
            arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10
        );
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_ten_args".to_string(),
        ))
    }
}

fn print_eleven_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
        ParameterValue::String(arg5),
        ParameterValue::Bool(arg6),
        ParameterValue::Bool(arg7),
        ParameterValue::UInt(arg8),
        ParameterValue::ULong(arg9),
        ParameterValue::Int(arg10),
        ParameterValue::Float(arg11),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
        function_call.parameters.clone().unwrap()[4].clone(),
        function_call.parameters.clone().unwrap()[5].clone(),
        function_call.parameters.clone().unwrap()[6].clone(),
        function_call.parameters.clone().unwrap()[7].clone(),
        function_call.parameters.clone().unwrap()[8].clone(),
        function_call.parameters.clone().unwrap()[9].clone(),
        function_call.parameters.clone().unwrap()[10].clone(),
    ) {
        let message = format!(
            "Message: arg1:{} arg2:{} arg3:{} arg4:{} arg5:{} arg6:{} arg7:{} arg8:{} arg9:{} arg10:{} arg11:{:.3}.",
            arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11
        );
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_eleven_args".to_string(),
        ))
    }
}

fn buffer_overrun(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::String(value) = function_call.parameters.clone().unwrap()[0].clone() {
        let c_str = value.as_str();

        let mut buffer: [u8; 17] = [0; 17];
        let length = c_str.len();

        let copy_length = length.min(buffer.len());
        buffer[..copy_length].copy_from_slice(&c_str.as_bytes()[..copy_length]);

        let result = (17i32).saturating_sub(length as i32);

        Ok(get_flatbuffer_result(result))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to buffer_overrun".to_string(),
        ))
    }
}

#[allow(unconditional_recursion)]
fn infinite_recursion(_a: &FunctionCall) -> Result<Vec<u8>> {
    // blackbox is needed so something
    //is written to the stack in release mode,
    //to trigger guard page violation
    let param = black_box(5);
    black_box(param);
    infinite_recursion(_a)
}

fn stack_overflow(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Int(i) = function_call.parameters.clone().unwrap()[0].clone() {
        loop_stack_overflow(i);
        Ok(get_flatbuffer_result(i))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to stack_overflow".to_string(),
        ))
    }
}
// This function will allocate i * (8KiB + 1B) on the stack
fn loop_stack_overflow(i: i32) {
    if i > 0 {
        let _nums = black_box([0u8; 0x2000 + 1]); // chkstk guaranteed to be called for > 8KiB
        loop_stack_overflow(i - 1);
    }
}

fn large_var(_: &FunctionCall) -> Result<Vec<u8>> {
    let _buffer = black_box([0u8; (DEFAULT_GUEST_STACK_SIZE + 1) as usize]);
    Ok(get_flatbuffer_result(DEFAULT_GUEST_STACK_SIZE + 1))
}

fn small_var(_: &FunctionCall) -> Result<Vec<u8>> {
    let _buffer = black_box([0u8; 1024]);
    Ok(get_flatbuffer_result(1024))
}

fn call_malloc(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Int(size) = function_call.parameters.clone().unwrap()[0].clone() {
        // will panic if OOM, and we need blackbox to avoid optimizing away this test
        let buffer = Vec::<u8>::with_capacity(size as usize);
        black_box(buffer);
        Ok(get_flatbuffer_result(size))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to call_malloc".to_string(),
        ))
    }
}

fn malloc_and_free(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Int(size) = function_call.parameters.clone().unwrap()[0].clone() {
        let alloc_length = if size < DEFAULT_GUEST_STACK_SIZE {
            size
        } else {
            size.min(MAX_BUFFER_SIZE as i32)
        };
        let allocated_buffer = vec![0; alloc_length as usize];
        drop(allocated_buffer);

        Ok(get_flatbuffer_result(size))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to malloc_and_free".to_string(),
        ))
    }
}

fn echo(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::String(value) = function_call.parameters.clone().unwrap()[0].clone() {
        Ok(get_flatbuffer_result(&*value))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to echo".to_string(),
        ))
    }
}

fn get_size_prefixed_buffer(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::VecBytes(data) = function_call.parameters.clone().unwrap()[0].clone() {
        Ok(get_flatbuffer_result(&*data))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to get_size_prefixed_buffer".to_string(),
        ))
    }
}

#[expect(
    clippy::empty_loop,
    reason = "This function is used to keep the CPU busy"
)]
fn spin(_: &FunctionCall) -> Result<Vec<u8>> {
    loop {
        // Keep the CPU 100% busy forever
    }

    #[allow(unreachable_code)]
    Ok(get_flatbuffer_result(()))
}

fn test_abort(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Int(code) = function_call.parameters.clone().unwrap()[0].clone() {
        abort_with_code(&[code as u8]);
    }
    Ok(get_flatbuffer_result(()))
}

fn test_abort_with_code_and_message(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (ParameterValue::Int(code), ParameterValue::String(message)) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
    ) {
        unsafe {
            abort_with_code_and_message(&[code as u8], message.as_ptr() as *const c_char);
        }
    }
    Ok(get_flatbuffer_result(()))
}

fn test_guest_panic(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::String(message) = function_call.parameters.clone().unwrap()[0].clone() {
        panic!("{}", message);
    }
    Ok(get_flatbuffer_result(()))
}

fn test_write_raw_ptr(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Long(offset) = function_call.parameters.clone().unwrap()[0].clone() {
        let min_stack_addr = unsafe { MIN_STACK_ADDRESS };
        let page_guard_start = min_stack_addr - PAGE_SIZE;
        let addr = {
            let abs = u64::try_from(offset.abs())
                .map_err(|_| error!("Invalid offset"))
                .unwrap();
            if offset.is_negative() {
                page_guard_start - abs
            } else {
                page_guard_start + abs
            }
        };
        unsafe {
            // print_output(format!("writing to {:#x}\n", addr).as_str()).unwrap();
            write_volatile(addr as *mut u8, 0u8);
        }
        return Ok(get_flatbuffer_result("success"));
    }
    Ok(get_flatbuffer_result("fail"))
}

fn execute_on_stack(_function_call: &FunctionCall) -> Result<Vec<u8>> {
    unsafe {
        let mut noop: u8 = 0x90;
        let stack_fn: fn() = core::mem::transmute(&mut noop as *mut u8);
        stack_fn();
    };
    Ok(get_flatbuffer_result("fail"))
}

fn execute_on_heap(_function_call: &FunctionCall) -> Result<Vec<u8>> {
    unsafe {
        // NO-OP followed by RET
        let heap_memory = Box::new([0x90u8, 0xC3]);
        let heap_fn: fn() = core::mem::transmute(Box::into_raw(heap_memory));
        heap_fn();
        black_box(heap_fn); // avoid optimization when running in release mode
    }
    // will only reach this point if heap is executable
    Ok(get_flatbuffer_result("fail"))
}

fn test_rust_malloc(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Int(code) = function_call.parameters.clone().unwrap()[0].clone() {
        let ptr = unsafe { malloc(code as usize) };
        Ok(get_flatbuffer_result(ptr as i32))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to test_rust_malloc".to_string(),
        ))
    }
}

fn log_message(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (ParameterValue::String(message), ParameterValue::Int(level)) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
    ) {
        let level = LevelFilter::iter().nth(level as usize).unwrap().to_level();

        match level {
            Some(level) => log::log!(level, "{}", &message),
            None => {
                // was passed LevelFilter::Off, do nothing
            }
        }
        Ok(get_flatbuffer_result(()))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to log_message".to_string(),
        ))
    }
}

fn trigger_exception(_: &FunctionCall) -> Result<Vec<u8>> {
    unsafe {
        core::arch::asm!("ud2");
    } // trigger an undefined instruction exception
    Ok(get_flatbuffer_result(()))
}

static mut COUNTER: i32 = 0;

fn add_to_static(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Int(i) = function_call.parameters.clone().unwrap()[0].clone() {
        let res = unsafe {
            COUNTER += i;
            COUNTER
        };
        Ok(get_flatbuffer_result(res))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to add_to_static".to_string(),
        ))
    }
}

fn get_static(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if function_call.parameters.is_none() {
        Ok(get_flatbuffer_result(unsafe { COUNTER }))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to get_static".to_string(),
        ))
    }
}

fn add_to_static_and_fail(_: &FunctionCall) -> Result<Vec<u8>> {
    unsafe {
        COUNTER += 10;
    };
    Err(HyperlightGuestError::new(
        ErrorCode::GuestError,
        "Crash on purpose".to_string(),
    ))
}

fn violate_seccomp_filters(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if function_call.parameters.is_none() {
        let res = call_host_function::<u64>("MakeGetpidSyscall", None, ReturnType::ULong)?;

        Ok(get_flatbuffer_result(res))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to violate_seccomp_filters".to_string(),
        ))
    }
}

fn call_given_paramless_hostfunc_that_returns_i64(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::String(hostfuncname) =
        function_call.parameters.clone().unwrap()[0].clone()
    {
        let res = call_host_function::<i64>(&hostfuncname, None, ReturnType::Long)?;

        Ok(get_flatbuffer_result(res))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to test_rust_malloc".to_string(),
        ))
    }
}

fn add(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (ParameterValue::Int(a), ParameterValue::Int(b)) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
    ) {
        let res = call_host_function::<i32>(
            "HostAdd",
            Some(Vec::from(&[ParameterValue::Int(a), ParameterValue::Int(b)])),
            ReturnType::Int,
        )?;
        Ok(get_flatbuffer_result(res))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to add".to_string(),
        ))
    }
}

// Does nothing, but used for testing large parameters
fn large_parameters(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (ParameterValue::VecBytes(v), ParameterValue::String(s)) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
    ) {
        black_box((v, s));
        Ok(get_flatbuffer_result(()))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to large_parameters".to_string(),
        ))
    }
}

fn read_from_user_memory(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (ParameterValue::ULong(num), ParameterValue::VecBytes(expected)) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
    ) {
        let bytes = read_n_bytes_from_user_memory(num).expect("Failed to read from user memory");

        // verify that the user memory contains the expected data
        if bytes != expected {
            error!("User memory does not contain the expected data");
            return Err(HyperlightGuestError::new(
                ErrorCode::GuestError,
                "User memory does not contain the expected data".to_string(),
            ));
        }

        Ok(get_flatbuffer_result(&*bytes))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to read_from_user_memory".to_string(),
        ))
    }
}

fn read_mapped_buffer(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (ParameterValue::ULong(base), ParameterValue::ULong(len)) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
    ) {
        let base = base as usize as *const u8;
        let len = len as usize;

        unsafe {
            hyperlight_guest_bin::paging::map_region(base as _, base as _, len as u64 + 4096)
        };

        let data = unsafe { core::slice::from_raw_parts(base, len) };

        Ok(get_flatbuffer_result(data))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to read_mapped_buffer".to_string(),
        ))
    }
}

#[no_mangle]
pub extern "C" fn hyperlight_main() {
    let read_from_user_memory_def = GuestFunctionDefinition::new(
        "ReadFromUserMemory".to_string(),
        Vec::from(&[ParameterType::ULong, ParameterType::VecBytes]),
        ReturnType::VecBytes,
        read_from_user_memory as usize,
    );

    register_function(read_from_user_memory_def);

    let read_mapped_buffer_def = GuestFunctionDefinition::new(
        "ReadMappedBuffer".to_string(),
        Vec::from(&[ParameterType::ULong, ParameterType::ULong]),
        ReturnType::VecBytes,
        read_mapped_buffer as usize,
    );

    register_function(read_mapped_buffer_def);

    let set_static_def = GuestFunctionDefinition::new(
        "SetStatic".to_string(),
        Vec::new(),
        ReturnType::Int,
        set_static as usize,
    );

    register_function(set_static_def);

    let simple_print_output_def = GuestFunctionDefinition::new(
        "PrintOutput".to_string(),
        Vec::from(&[ParameterType::String]),
        ReturnType::Int,
        simple_print_output as usize,
    );
    register_function(simple_print_output_def);

    let print_using_printf_def = GuestFunctionDefinition::new(
        "PrintUsingPrintf".to_string(),
        Vec::from(&[ParameterType::String]),
        ReturnType::Int,
        simple_print_output as usize, // alias to simple_print_output for now
    );
    register_function(print_using_printf_def);

    let stack_overflow_def = GuestFunctionDefinition::new(
        "StackOverflow".to_string(),
        Vec::from(&[ParameterType::Int]),
        ReturnType::Int,
        stack_overflow as usize,
    );
    register_function(stack_overflow_def);

    let buffer_overrun_def = GuestFunctionDefinition::new(
        "BufferOverrun".to_string(),
        Vec::from(&[ParameterType::String]),
        ReturnType::Int,
        buffer_overrun as usize,
    );
    register_function(buffer_overrun_def);

    let large_var_def = GuestFunctionDefinition::new(
        "LargeVar".to_string(),
        Vec::new(),
        ReturnType::Int,
        large_var as usize,
    );
    register_function(large_var_def);

    let small_var_def = GuestFunctionDefinition::new(
        "SmallVar".to_string(),
        Vec::new(),
        ReturnType::Int,
        small_var as usize,
    );
    register_function(small_var_def);

    let call_malloc_def = GuestFunctionDefinition::new(
        "CallMalloc".to_string(),
        Vec::from(&[ParameterType::Int]),
        ReturnType::Int,
        call_malloc as usize,
    );
    register_function(call_malloc_def);

    let malloc_and_free_def = GuestFunctionDefinition::new(
        "MallocAndFree".to_string(),
        Vec::from(&[ParameterType::Int]),
        ReturnType::Int,
        malloc_and_free as usize,
    );
    register_function(malloc_and_free_def);

    let print_two_args_def = GuestFunctionDefinition::new(
        "PrintTwoArgs".to_string(),
        Vec::from(&[ParameterType::String, ParameterType::Int]),
        ReturnType::Int,
        print_two_args as usize,
    );
    register_function(print_two_args_def);

    let print_three_args_def = GuestFunctionDefinition::new(
        "PrintThreeArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
        ]),
        ReturnType::Int,
        print_three_args as usize,
    );
    register_function(print_three_args_def);

    let print_four_args_def = GuestFunctionDefinition::new(
        "PrintFourArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
        ]),
        ReturnType::Int,
        print_four_args as usize,
    );
    register_function(print_four_args_def);

    let print_five_args_def = GuestFunctionDefinition::new(
        "PrintFiveArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
            ParameterType::String,
        ]),
        ReturnType::Int,
        print_five_args as usize,
    );
    register_function(print_five_args_def);

    let print_six_args_def = GuestFunctionDefinition::new(
        "PrintSixArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
            ParameterType::String,
            ParameterType::Bool,
        ]),
        ReturnType::Int,
        print_six_args as usize,
    );
    register_function(print_six_args_def);

    let print_seven_args_def = GuestFunctionDefinition::new(
        "PrintSevenArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
            ParameterType::String,
            ParameterType::Bool,
            ParameterType::Bool,
        ]),
        ReturnType::Int,
        print_seven_args as usize,
    );
    register_function(print_seven_args_def);

    let print_eight_args_def = GuestFunctionDefinition::new(
        "PrintEightArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
            ParameterType::String,
            ParameterType::Bool,
            ParameterType::Bool,
            ParameterType::UInt,
        ]),
        ReturnType::Int,
        print_eight_args as usize,
    );
    register_function(print_eight_args_def);

    let print_nine_args_def = GuestFunctionDefinition::new(
        "PrintNineArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
            ParameterType::String,
            ParameterType::Bool,
            ParameterType::Bool,
            ParameterType::UInt,
            ParameterType::ULong,
        ]),
        ReturnType::Int,
        print_nine_args as usize,
    );
    register_function(print_nine_args_def);

    let print_ten_args_def = GuestFunctionDefinition::new(
        "PrintTenArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
            ParameterType::String,
            ParameterType::Bool,
            ParameterType::Bool,
            ParameterType::UInt,
            ParameterType::ULong,
            ParameterType::Int,
        ]),
        ReturnType::Int,
        print_ten_args as usize,
    );
    register_function(print_ten_args_def);

    let print_eleven_args_def = GuestFunctionDefinition::new(
        "PrintElevenArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
            ParameterType::String,
            ParameterType::Bool,
            ParameterType::Bool,
            ParameterType::UInt,
            ParameterType::ULong,
            ParameterType::Int,
            ParameterType::Float,
        ]),
        ReturnType::Int,
        print_eleven_args as usize,
    );
    register_function(print_eleven_args_def);

    let set_byte_array_to_zero_def = GuestFunctionDefinition::new(
        "SetByteArrayToZero".to_string(),
        Vec::from(&[ParameterType::VecBytes]),
        ReturnType::VecBytes,
        set_byte_array_to_zero as usize,
    );
    register_function(set_byte_array_to_zero_def);

    let echo_def = GuestFunctionDefinition::new(
        "Echo".to_string(),
        Vec::from(&[ParameterType::String]),
        ReturnType::String,
        echo as usize,
    );
    register_function(echo_def);

    let get_size_prefixed_buffer_def = GuestFunctionDefinition::new(
        "GetSizePrefixedBuffer".to_string(),
        Vec::from(&[ParameterType::VecBytes]),
        ReturnType::Int,
        get_size_prefixed_buffer as usize,
    );
    register_function(get_size_prefixed_buffer_def);

    let spin_def = GuestFunctionDefinition::new(
        "Spin".to_string(),
        Vec::new(),
        ReturnType::Int,
        spin as usize,
    );
    register_function(spin_def);

    let abort_def = GuestFunctionDefinition::new(
        "GuestAbortWithCode".to_string(),
        Vec::from(&[ParameterType::Int]),
        ReturnType::Void,
        test_abort as usize,
    );
    register_function(abort_def);

    let abort_with_code_message_def = GuestFunctionDefinition::new(
        "GuestAbortWithMessage".to_string(),
        Vec::from(&[ParameterType::Int, ParameterType::String]),
        ReturnType::Void,
        test_abort_with_code_and_message as usize,
    );
    register_function(abort_with_code_message_def);

    let guest_panic_def = GuestFunctionDefinition::new(
        "guest_panic".to_string(),
        Vec::from(&[ParameterType::String]),
        ReturnType::Void,
        test_guest_panic as usize,
    );
    register_function(guest_panic_def);

    let rust_malloc_def = GuestFunctionDefinition::new(
        "TestMalloc".to_string(),
        Vec::from(&[ParameterType::Int]),
        ReturnType::Int,
        test_rust_malloc as usize,
    );
    register_function(rust_malloc_def);

    let log_message_def = GuestFunctionDefinition::new(
        "LogMessage".to_string(),
        Vec::from(&[ParameterType::String, ParameterType::Int]),
        ReturnType::Void,
        log_message as usize,
    );
    register_function(log_message_def);

    let infinite_recursion_def = GuestFunctionDefinition::new(
        "InfiniteRecursion".to_string(),
        Vec::new(),
        ReturnType::Void,
        infinite_recursion as usize,
    );
    register_function(infinite_recursion_def);

    let test_write_raw_ptr_def = GuestFunctionDefinition::new(
        "test_write_raw_ptr".to_string(),
        Vec::from(&[ParameterType::Long]),
        ReturnType::String,
        test_write_raw_ptr as usize,
    );
    register_function(test_write_raw_ptr_def);

    let execute_on_stack_def = GuestFunctionDefinition::new(
        "ExecuteOnStack".to_string(),
        Vec::new(),
        ReturnType::String,
        execute_on_stack as usize,
    );
    register_function(execute_on_stack_def);

    let execute_on_heap_def = GuestFunctionDefinition::new(
        "ExecuteOnHeap".to_string(),
        Vec::new(),
        ReturnType::String,
        execute_on_heap as usize,
    );
    register_function(execute_on_heap_def);

    let add_to_static_def = GuestFunctionDefinition::new(
        "AddToStatic".to_string(),
        Vec::from(&[ParameterType::Int]),
        ReturnType::Int,
        add_to_static as usize,
    );
    register_function(add_to_static_def);

    let get_static_def = GuestFunctionDefinition::new(
        "GetStatic".to_string(),
        Vec::new(),
        ReturnType::Int,
        get_static as usize,
    );
    register_function(get_static_def);

    let add_to_static_and_fail_def = GuestFunctionDefinition::new(
        "AddToStaticAndFail".to_string(),
        Vec::new(),
        ReturnType::Int,
        add_to_static_and_fail as usize,
    );
    register_function(add_to_static_and_fail_def);

    let violate_seccomp_filters_def = GuestFunctionDefinition::new(
        "ViolateSeccompFilters".to_string(),
        Vec::new(),
        ReturnType::ULong,
        violate_seccomp_filters as usize,
    );
    register_function(violate_seccomp_filters_def);

    let echo_float_def = GuestFunctionDefinition::new(
        "EchoFloat".to_string(),
        Vec::from(&[ParameterType::Float]),
        ReturnType::Float,
        echo_float as usize,
    );
    register_function(echo_float_def);

    let echo_double_def = GuestFunctionDefinition::new(
        "EchoDouble".to_string(),
        Vec::from(&[ParameterType::Double]),
        ReturnType::Double,
        echo_double as usize,
    );
    register_function(echo_double_def);

    let add_def = GuestFunctionDefinition::new(
        "Add".to_string(),
        Vec::from(&[ParameterType::Int, ParameterType::Int]),
        ReturnType::Int,
        add as usize,
    );
    register_function(add_def);

    let trigger_exception_def = GuestFunctionDefinition::new(
        "TriggerException".to_string(),
        Vec::new(),
        ReturnType::Void,
        trigger_exception as usize,
    );
    register_function(trigger_exception_def);

    let large_parameters_def = GuestFunctionDefinition::new(
        "LargeParameters".to_string(),
        Vec::from(&[ParameterType::VecBytes, ParameterType::String]),
        ReturnType::Void,
        large_parameters as usize,
    );
    register_function(large_parameters_def);

    let call_given_hostfunc_def = GuestFunctionDefinition::new(
        "CallGivenParamlessHostFuncThatReturnsI64".to_string(),
        Vec::from(&[ParameterType::String]),
        ReturnType::Long,
        call_given_paramless_hostfunc_that_returns_i64 as usize,
    );
    register_function(call_given_hostfunc_def);
}

#[no_mangle]
pub fn guest_dispatch_function(function_call: FunctionCall) -> Result<Vec<u8>> {
    // This test checks the stack behavior of the input/output buffer
    // by calling the host before serializing the function call.
    // If the stack is not working correctly, the input or output buffer will be
    // overwritten before the function call is serialized, and we will not be able
    // to verify that the function call name is "ThisIsNotARealFunctionButTheNameIsImportant"
    if function_call.function_name == "FuzzHostFunc" {
        return fuzz_host_function(function_call);
    }

    let message = "Hi this is a log message that will overwrite the shared buffer if the stack is not working correctly";

    guest_logger::log_message(
        LogLevel::Information,
        message,
        "source",
        "caller",
        "file",
        1,
    );

    let result = call_host_function::<i32>(
        "HostPrint",
        Some(Vec::from(&[ParameterValue::String(message.to_string())])),
        ReturnType::Int,
    )?;
    let function_name = function_call.function_name.clone();
    let param_len = function_call.parameters.clone().unwrap_or_default().len();
    let call_type = function_call.function_call_type().clone();

    if function_name != "ThisIsNotARealFunctionButTheNameIsImportant"
        || param_len != 0
        || call_type != FunctionCallType::Guest
        || result != 100
    {
        return Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionNotFound,
            function_name,
        ));
    }

    Ok(get_flatbuffer_result(99))
}

// Interprets the given guest function call as a host function call and dispatches it to the host.
fn fuzz_host_function(func: FunctionCall) -> Result<Vec<u8>> {
    let mut params = func.parameters.unwrap();
    // first parameter must be string (the name of the host function to call)
    let host_func_name = match params.remove(0) {
        // TODO use `swap_remove` instead of `remove` if performance is an issue, but left out
        // to avoid confusion for replicating failure cases
        ParameterValue::String(name) => name,
        _ => {
            return Err(HyperlightGuestError::new(
                ErrorCode::GuestFunctionParameterTypeMismatch,
                "Invalid parameters passed to fuzz_host_function".to_string(),
            ));
        }
    };

    // Because we do not know at compile time the actual return type of the host function to be called
    // we cannot use the `call_host_function<T>` generic function.
    // We need to use the `call_host_function_without_returning_result` function that does not retrieve the return
    // value
    call_host_function_without_returning_result(
        &host_func_name,
        Some(params),
        func.expected_return_type,
    )
    .expect("failed to call host function");
    Ok(get_flatbuffer_result(()))
}
