//! Process management syscalls
use core::mem::size_of;

use alloc::boxed::Box;

use crate::{
    mm::{translated_byte_buffer, MapPermission, PageTable, StepByOne, VirtAddr},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_current_task,
        get_task_syscall_time, is_vpn_available, suspend_current_and_run_next, task_alloc_mem,
        task_dealloc_mem,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let buffers =
        translated_byte_buffer(current_user_token(), ts as *const u8, size_of::<TimeVal>());

    let us = get_time_us();
    let ts = Box::into_raw(Box::new(TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    })) as *const u8;

    let mut index = 0;
    for buffer in buffers {
        for byte in buffer {
            *byte = unsafe { *ts.offset(index) };
            index += 1;
        }
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");

    let task_id = get_current_task();

    let page_table = PageTable::from_token(current_user_token());
    let va = VirtAddr::from(id);
    let vpn = va.floor();

    match trace_request {
        0 => {
            if let Some(pte) = page_table.translate(vpn) {
                if !is_vpn_available(task_id, vpn) || !pte.is_valid() || !pte.readable() {
                    -1
                } else {
                    let ppn = pte.ppn();
                    ppn.get_bytes_array()[va.page_offset()] as isize
                }
            } else {
                -1
            }
        }
        1 => {
            if let Some(pte) = page_table.translate(vpn) {
                if !is_vpn_available(task_id, vpn) || !pte.is_valid() || !pte.writable() {
                    -1
                } else {
                    let ppn = pte.ppn();
                    ppn.get_bytes_array()[va.page_offset()] = data as u8;
                    0
                }
            } else {
                -1
            }
        }
        2 => get_task_syscall_time(get_current_task(), id) as isize,
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);

    // `start` must align by page size
    if !start_va.aligned() {
        return -1;
    };

    // `port` must satisfy conditions
    if port & !0x7 != 0 || port & 0x7 == 0 {
        return -1;
    };

    let page_table = PageTable::from_token(current_user_token());
    let mut start_vpn = start_va.floor();
    let end_vpn = end_va.ceil();

    while start_vpn < end_vpn {
        if let Some(pte) = page_table.translate(start_vpn) {
            if pte.is_valid() {
                // page already mapped
                return -1;
            };
        };
        start_vpn.step();
    }

    task_alloc_mem(
        get_current_task(),
        start_va,
        end_va,
        MapPermission::new((port as u8 + 0x8) << 1),
    )
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);

    // `start` must align by page size
    if !start_va.aligned() {
        return -1;
    };

    let page_table = PageTable::from_token(current_user_token());
    let mut start_vpn = start_va.floor();
    let end_vpn = end_va.ceil();

    // fails when includes unmapped virtual memory
    while start_vpn < end_vpn {
        if let Some(pte) = page_table.translate(start_vpn) {
            if !pte.is_valid() {
                return -1;
            };
        } else {
            return -1;
        };

        start_vpn.step();
    }

    task_dealloc_mem(get_current_task(), start_va)
}

pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
