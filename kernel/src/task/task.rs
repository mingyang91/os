#[derive(Debug)]
pub struct TaskControlBlock {
    pub task_ctx_ptr: usize,
    pub task_status: TaskStatus,
}

impl TaskControlBlock {
    pub fn get_task_ctx_ptr2(&self) -> *const usize {
        &self.task_ctx_ptr as *const usize
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}