# lab3

## 简单总结你实现的功能（200字以内，不要贴代码）

迁移 `sys_get_time` `sys_mmap` `sys_munmap`，与上一章的实现方法一致。

实现 `sys_spawn`，通过文件路径加载 elf_data，并以此创建新的子进程 `TaskControlBlock`，将其 parent 设置为当前进程，并在当前进程的子进程列表中加入新进程。通过 `add_task` 将子进程加入 task manager 并开始执行。

实现 `sys_set_priority`，在 task inner 中加入新的字段 `prio`，并在系统调用中直接赋值。

实现 stride scheduling。

- 在 `TaskControlBlock` 中添加 `stride` 和 `prio` 字段
- 在 task manager `fetch` 下一个需要执行的进程时，使用 `min_by_key` 挑选出 `stride` 最小的那个
- 执行进程时，将其 `stride` 加上 `BIG_STRIDE / prio` 即 `pass`

## 完成问答题

并非 p1。主要原因是 $250 + 10 \equiv 4 \pmod{256}$， p2 执行一个时间片后其 stride 变为 4, 小于 `p1.stride`，故调度器会再次选择 p2。

$prio \ge 2$ 则每次执行完后增加的步长小于等于 `BIG_STRIDE / 2`，然后调度器会去执行其他 stride 更小的进程，他们的 stride 会逐渐与第一个缩小差距。故 stride 之间的差距永远不会大于 `BIG_STRIDE / 2`。

```rs
impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 - other.0 > BIG_STRIDE / 2 {
            Some(Ordering::Less)
        } else if other.0 - self.0 > BIG_STRIDE / 2 {
            Some(Ordering::Greater)
        } else {
            self.0.partial_cmp(other.0)
        }
    }
}
```

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无。

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

无。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

大部分代码复制自本人之前的作答，见 [LearningOS/2024a-rcore-huizm](https://github.com/LearningOS/2024a-rcore-huizm/tree/ch5)。
