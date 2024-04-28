//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{exit_current_and_run_next, suspend_current_and_run_next, TaskStatus,TASK_MANAGER},
    timer::{get_time_us,get_time_ms},
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}
impl TaskInfo {
    /// 
    pub fn new()->Self{
        let syscall_times=[0u32; MAX_SYSCALL_NUM];
        Self { status: TaskStatus::UnInit, syscall_times: syscall_times, time: 0 }
    }
    ///
    pub fn call(&mut self,syscall:usize){
        self.syscall_times[syscall]+=1;
    }
    /// 
    pub fn flush(&mut self){
        if self.time==0{
            self.time=get_time_ms();
        }
    }
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    unsafe{
        let current_task_info=(*TASK_MANAGER.current_task()).get_info();
        (*_ti).syscall_times=current_task_info.syscall_times;
        (*_ti).status=TaskStatus::Running;
        (*_ti).time=get_time_ms()-current_task_info.time;
        0
    }
}
