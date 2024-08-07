use std::fmt;
use std::sync::{ Arc, Mutex, mpsc };

use crate::pool::worker::{Worker, Job};

#[derive(Debug)]
pub struct PoolCreationError;
impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pool cannot be created with no threads")
    }
}
impl std::error::Error for PoolCreationError {}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>
}

impl ThreadPool {
    pub fn workers(&self) -> &Vec<Worker> {
        &self.workers
    }

    pub fn sender(&self) -> Result<&mpsc::Sender<Job>, &'static str> {
        self.sender.as_ref().ok_or("Unable to get reference to sender.")
    }
    /// Creates a new ThreadPool with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The number of worker threads to create.
    ///
    /// # Returns
    ///
    /// * `Ok(ThreadPool)` if the pool was successfully created.
    /// * `Err(PoolCreationError)` if the capacity is zero.
    pub fn build(capacity: u32) -> Result<ThreadPool, PoolCreationError> {
        if capacity == 0 {
            return Err(PoolCreationError)
        }
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let workers = (0..capacity)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();
        Ok(ThreadPool { workers, sender: Some(sender) })
    }

    /// Executes a job on the thread pool.
    ///
    /// # Arguments
    ///
    /// * `f` - The job to be executed.
    pub fn execute<F>(&self, f: F)
    where 
        F: FnOnce() + Send + 'static
    {
        match self.sender() {
            Ok(sender) => {
                if let Err(e) = sender.send(Box::new(f)) {
                    eprintln!("Sender failed to send job to threadpool: {e}");
                }
            },
            Err(e) => {
                eprintln!("Failed to get sender: {}", e);
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("byebye -love, Worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                if let Err(e) = thread.join() {
                    eprintln!("Worker {} failed to join: {:?}", worker.id, e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use crate::pool::thread_pool::ThreadPool;

    #[test]
    fn test_thread_pool_creation_with_capacity() {
        let pool = ThreadPool::build(4).unwrap();
        assert_eq!(pool.workers().len(), 4);
    }

    #[test]
    fn test_thread_pool_creation_error() {
        let pool_creation_result = ThreadPool::build(0);
        assert!(matches!(pool_creation_result, Err(PoolCreationError)));
    }

    #[test]
    fn test_thread_pool_execution() {
        let pool = ThreadPool::build(4).unwrap();
        let (tx, rx) = mpsc::channel();
        pool.execute(move || { tx.send("hello").unwrap(); });
        assert_eq!(rx.recv().unwrap(), "hello");
    }
}
