## 编程作业

1.死锁检测

> 当线程尝试获取Mutex或Semaphore时判断是否有依赖环，如果有环，就会死锁。
>
> 通过dfs判断环。Semaphore环检测会有多条路径，需要全部路径都成环才会死锁(一个线程一次只能获取一个Semaphore)。

## 简答作业

1. 在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源。 

   - 需要回收的资源有哪些？ 

   > tid,trap_cx,ustack

   - 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？

   > TaskManager: 需要移除引用，因为线程不再运行，不再参与调度。
   >
   > Mutex，Semaphore，Condver: 不需要移除引用，因为它们都属于进程，进程结束就会自动回收。

2. 对比以下两种 `Mutex.unlock` 的实现，二者有什么区别？这些区别可能会导致什么问题？

```rust
 1impl Mutex for Mutex1 {
 2    fn unlock(&self) {
 3        let mut mutex_inner = self.inner.exclusive_access();
 4        assert!(mutex_inner.locked);
 5        mutex_inner.locked = false;
 6        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
 7            add_task(waking_task);
 8        }
 9    }
10}
11
12impl Mutex for Mutex2 {
13    fn unlock(&self) {
14        let mut mutex_inner = self.inner.exclusive_access();
15        assert!(mutex_inner.locked);
16        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
17            add_task(waking_task);
18        } else {
19            mutex_inner.locked = false;
20        }
21    }
22}
```

> 两种方式的区别在mutex中有等待线程时体现：
>
> 第一种方式：解锁，唤醒
>
> 第二种方式：不解锁，唤醒，相当于直接将锁移交
>
> 当一个线程调用mutex.lock尝试加锁时，如果已经上锁会加入等待队列，并阻塞；如果是第一种方式，当线程从等待队列被唤醒时，会直接进行mutex.lock的下一条指令，这时mutex_inner.locked=false，其他线程调用lock还能获取锁，就会有两个线程都获取到同一个mutex，这是错误的。
>
> 如果是第二种方式，线程被唤醒时mutex_inner.locked=true，自动获取锁，这是正确的。

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > 独立完成，无交流对象。

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > 仅参考rCore-Tutorial-Guide-2024S官方文档。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
