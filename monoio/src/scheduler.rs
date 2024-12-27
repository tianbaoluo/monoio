use std::{cell::UnsafeCell, collections::VecDeque, marker::PhantomData};

use crate::task::{Schedule, Task};

pub(crate) struct LocalScheduler;

impl Schedule for LocalScheduler {
    fn schedule(&self, task: Task<Self>) {
        crate::runtime::CURRENT.with(|cx| cx.tasks.push(task));
    }

    fn yield_now(&self, task: Task<Self>) {
        crate::runtime::CURRENT.with(|cx| cx.tasks.push_front(task));
    }
}

pub(crate) struct SubLocalScheduler;
impl Schedule for SubLocalScheduler {
    fn schedule(&self, task: Task<Self>) {
        crate::runtime::CURRENT.with(|cx| cx.sub_tasks.push(task));
    }

    fn yield_now(&self, task: Task<Self>) {
        crate::runtime::CURRENT.with(|cx| cx.sub_tasks.push_front(task));
    }
}

pub(crate) struct TaskQueue<S: Schedule> {
    // Local queue.
    queue: UnsafeCell<VecDeque<Task<S>>>,
    // Make sure the type is `!Send` and `!Sync`.
    _marker: PhantomData<*const ()>,
}

impl <S: Schedule> Default for TaskQueue<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl <S: Schedule> Drop for TaskQueue<S> {
    fn drop(&mut self) {
        unsafe {
            let queue = &mut *self.queue.get();
            while let Some(_task) = queue.pop_front() {}
        }
    }
}

impl <S: Schedule> TaskQueue<S> {
    pub(crate) fn new() -> Self {
        const DEFAULT_TASK_QUEUE_SIZE: usize = 4096;
        Self::new_with_capacity(DEFAULT_TASK_QUEUE_SIZE)
    }
    pub(crate) fn new_with_capacity(capacity: usize) -> Self {
        Self {
            queue: UnsafeCell::new(VecDeque::with_capacity(capacity)),
            _marker: PhantomData,
        }
    }

    pub(crate) fn len(&self) -> usize {
        unsafe { (*self.queue.get()).len() }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn push(&self, runnable: Task<S>) {
        unsafe {
            (*self.queue.get()).push_back(runnable);
        }
    }

    pub(crate) fn push_front(&self, runnable: Task<S>) {
        unsafe {
            (*self.queue.get()).push_front(runnable);
        }
    }

    pub(crate) fn pop(&self) -> Option<Task<S>> {
        unsafe { (*self.queue.get()).pop_front() }
    }
}
