//! Process management syscalls
use crate::config::PAGE_SIZE;
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, task_memory_map, task_memory_unmap};
use crate::timer::get_time_us;

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


use crate::mm::translated_byte_buffer;
use crate::task::current_user_token;

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let buffers = translated_byte_buffer(current_user_token(), _ts as *const u8, core::mem::size_of::<TimeVal>());
    let timeval = TimeVal { 
        sec: us / 1_000_000,
        usec: us % 1_000_000
    };
    let timeval_bytes = unsafe {
        core::slice::from_raw_parts(
            &timeval as *const TimeVal as *const u8,
            core::mem::size_of::<TimeVal>(),
        )
    };
    let mut offset = 0;
    for buffer in buffers {
        let len = buffer.len();
        buffer.copy_from_slice(&timeval_bytes[offset..offset + len]);
        offset += len;
    }
    0
}

use crate::mm::PageTable;
use crate::mm::VirtAddr;

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        0 => {
            // read
            let page_table = PageTable::from_token(current_user_token());
            let vpn = VirtAddr::from(_id).floor();
            let pte = page_table.translate(vpn);
            if pte.is_none() || !pte.unwrap().is_user() || !pte.unwrap().is_valid() || !pte.unwrap().readable() {
                -1
            } else {
                let ppn = pte.unwrap().ppn();
                let val = ppn.get_bytes_array()[VirtAddr::from(_id).page_offset()];
                val as isize
            }
        }
        1 => {
            // write
            let page_table = PageTable::from_token(current_user_token());
            let vpn = VirtAddr::from(_id).floor();
            let pte = page_table.translate(vpn);
            if pte.is_none() || !pte.unwrap().is_user() || !pte.unwrap().is_valid() || !pte.unwrap().writable() {
                -1
            } else {
                let ppn = pte.unwrap().ppn();
                ppn.get_bytes_array()[VirtAddr::from(_id).page_offset()] = _data as u8;
                0
            }
        }
        2 => {
            // count
            let count = crate::task::count_syscall(_id);
            count as isize
        }
        _ => -1
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    if prot & !0x7 != 0 || prot & 0x7 == 0 {
        return -1;
    }
    task_memory_map(start, len, prot)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    task_memory_unmap(start, len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
