//! Process management syscalls
use crate::{
    config::{MAX_SYSCALL_NUM,PAGE_SIZE},
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, current_user_token,TASK_MANAGER
    }, timer::{ get_time_us,get_time_ms}, mm::{translated_byte_buffer,MapPermission},
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
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
    pub fn init_time(&mut self)->&mut Self{
        if self.time==0{
            self.time=get_time_ms();
        }
        self
    }
    /// 
    pub fn set_time(&mut self,time:usize)->&mut Self{
        self.time=time;
        self
    }
    ///
    pub fn get_time(&self)->usize{
        self.time
    }
    /// 
    pub fn flush_status(&mut self,task_status:TaskStatus)->&mut Self{
        self.status=task_status;
        self
    }
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
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us=get_time_us();
    let time_val=TimeVal{
        sec:us/1_000_000,
        usec:us%1_000_000
    };
    // maybe uncontinuous  
    let user_time_val_buffer=translated_byte_buffer(current_user_token(), _ts as *const u8, core::mem::size_of::<TimeVal>());
    unsafe{
        // get timeval bytes 
        let time_val_bytes=core::slice::from_raw_parts((&time_val as *const TimeVal)as *const u8 , core::mem::size_of::<TimeVal>());    
        // copy timeval to user space
        let mut offset = 0;
        for bytes in user_time_val_buffer{
            bytes.copy_from_slice(&time_val_bytes[offset..offset+bytes.len()]);
            offset+=bytes.len();
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    // taskinfo maybe uncontinuous 
    let user_task_info_buffer=translated_byte_buffer(current_user_token(), _ti as *const u8, core::mem::size_of::<TaskInfo>());
    unsafe{
        // get taskinfo bytes
        let current_task_info=(*TASK_MANAGER.current_task()).get_info();
        let origin_time=current_task_info.get_time();
        current_task_info.set_time(get_time_ms()-origin_time);
        let task_info_bytes=core::slice::from_raw_parts((current_task_info as *const TaskInfo)as * const u8 , core::mem::size_of::<TaskInfo>());
        // copy to user space
        let mut offset=0;
        for bytes in user_task_info_buffer{
            bytes.copy_from_slice(&task_info_bytes[offset..offset+bytes.len()]);
            offset+=bytes.len();
        }
        current_task_info.set_time(origin_time);
    }
    0   
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _port & 0x7 ==0 || _port & !0x7 != 0 || _start % PAGE_SIZE!=0 {
        return -1
    }
    let mut permission=MapPermission::U;
    if _port & 0x1 ==1 {
        permission|=MapPermission::R;
    }
    if _port & 0x2 ==0x2 {
        permission|=MapPermission::W;
    }
    if _port & 0x4 ==0x4{
        permission|=MapPermission::X;
    }
    unsafe {
        // ceil(4096)=4096 
        (*TASK_MANAGER.current_task()).memory_set.mmap(_start.into(), (_start+_len).into(), permission)
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if _start % PAGE_SIZE!=0 || _len % PAGE_SIZE!=0{
        return -1
    }
    unsafe{
        (*TASK_MANAGER.current_task()).memory_set.unmap(_start.into(), (_start+_len).into());
    }
    0
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
