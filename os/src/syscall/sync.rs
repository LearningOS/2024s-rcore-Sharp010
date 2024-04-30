use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task,suspend_current_and_run_next};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );

    let tid=current_task().unwrap().inner_exclusive_access().get_tid();
    let process = current_task().unwrap().process.upgrade().unwrap();
    let process_inner = process.inner_exclusive_access();
    let detect=process_inner.deadlock_detect;
    let sem_enable=process_inner.semaphore_list.len()>=1;
    if detect && sem_enable{
        println!("sleep !!! {}  {}",tid,ms);
    }
    drop(process_inner);
    drop(process);
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        println!("??????????");
        process_inner.mutex_list[id] = mutex;
        id as isize
    } else {
        // println!("create mutex!");
        process_inner.mutex_list.push(mutex);
        if process_inner.deadlock_detect{
            process_inner.mutex_available.push(1);
            let mutex_len=process_inner.mutex_list.len();
            assert_eq!( mutex_len,process_inner.mutex_available.len());
            // println!("current thread_id!  tid{}",current_task().unwrap().inner_exclusive_access().get_tid());
            println!("mutex create!  len{}",process_inner.mutex_list.len());
            for mutexs in process_inner.mutex_need.iter_mut(){
                mutexs.push(0);
                println!("mutex add! ");
                assert_eq!( mutex_len,mutexs.len());
            }
            // println!("mutex_need! len {}",process_inner.mutex_need.len());
            for mutexs in process_inner.mutex_allocation.iter_mut(){
                mutexs.push(0);
                assert_eq!( mutex_len,mutexs.len());
            }
        }
        // println!("mutex_allocation! len {}",process_inner.mutex_allocation.len());
        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let mut process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    let deadlock_detect=process_inner.deadlock_detect;
    // drop(process_inner);
    // drop(process);
    if  deadlock_detect  {
        let mut result=process_inner.detect_mutex_deadlock(mutex_id);
        println!("mutex lock!!!!111 detect {}",result);
        while result==-1 {
            println!("mutex lock!!!!333 mutex len {} mutex_id {}",process_inner.mutex_list.len(),mutex_id);
            drop(process_inner);
            drop(process);
            suspend_current_and_run_next();
            process = current_process();
            process_inner = process.inner_exclusive_access();
            result=process_inner.detect_mutex_deadlock(mutex_id);
        }
        match result{
            // error
            -0xdead => {
                return -0xdead;
            }
            // correct
            _ =>{}
        }
        println!("mutex lock!!!!222");
    }
    drop(process_inner);
    drop(process);
    mutex.lock();
    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    // println!("mutex unlock!!!!111");
    let current_tid=current_task().unwrap().inner_exclusive_access().get_tid();
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    if process_inner.deadlock_detect{
        if process_inner.mutex_allocation[current_tid][mutex_id]<=0{
            return -0xdead;
        }
        println!("mutex unlock!!!!222{}",current_tid);
        // process_inner.mutex_allocation[0].push(true);
        process_inner.mutex_allocation[current_tid][mutex_id]=0;
        process_inner.mutex_available[mutex_id]+=1;
        // process_inner.mutex_need[current_tid][mutex_id]=true;
    }
    drop(process_inner);
    drop(process);
    // println!("mutex unlock!!!!333");
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        if process_inner.deadlock_detect{
            process_inner.sem_available.push(res_count as u32);
            assert_eq!( process_inner.semaphore_list.len(),process_inner.sem_available.len());
            println!("sem create!!!!111 count {} id {}",res_count,process_inner.semaphore_list.len() - 1);
            for sems in process_inner.sem_need.iter_mut(){
                sems.push(0);
            }
            for sems in process_inner.sem_allocation.iter_mut(){
                sems.push(0);
            }
        }
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let current_tid=current_task().unwrap().inner_exclusive_access().get_tid();
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    if process_inner.deadlock_detect{
        println!("detect {} ",process_inner.deadlock_detect);
        println!("sem up! sem{} tid{} ",sem_id,current_tid);
        if process_inner.sem_allocation[current_tid][sem_id]<=0{
            println!("unexpect sem up!");
            return -0xdead;
        }
        println!("tid[{}] realease{}",current_tid,sem_id);
        process_inner.sem_allocation[current_tid][sem_id]-=1;
        process_inner.sem_available[sem_id]+=1;
        // process_inner.sem_need[current_tid][sem_id]=0;
    }
    drop(process_inner);
    drop(process);
    sem.up();
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let current_tid=current_task().unwrap().inner_exclusive_access().get_tid();
    let  mut process = current_process();
    let  mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    // // drop(process_inner);
    // // drop(process);
    let deadlock_detect=process_inner.deadlock_detect;
    if  deadlock_detect  {
        print!("available sem:");
        for sem in process_inner.sem_available.iter(){
            print!("{}",sem);
        }
        println!("");
        let mut result=process_inner.detect_semaphore_deadlock(sem_id);
        println!("sem down!!!! detect_lock {}  tid{} => semid {}",result,current_tid,sem_id);
        while result==-1 {
            // println!("sem down!!!!333 sem len {}",process_inner.semaphore_list.len());
            drop(process_inner);
            drop(process);
            suspend_current_and_run_next();
            process = current_process();
            process_inner = process.inner_exclusive_access();
            result=process_inner.detect_semaphore_deadlock(sem_id);
        }
        match result{
            // error
            -0xdead => {
                return -0xdead;
            }
            // correct
            _ =>{}
        }
        println!("sem down ok!");
        
    }else{
        drop(process_inner);
        drop(process);
    }
    sem.down();
    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(_enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    current_process().inner_exclusive_access().deadlock_detect=if _enabled==1{true}else{false};
    0
}
