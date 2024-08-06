use std::fmt;
use std::{
    sync::{Arc, Mutex, mpsc},
    thread
};

#[derive(Debug)]
pub struct PoolCreationError;
impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pool cannot be created with no threads")
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct Worker {
    id: u32,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new Worker
    /// 
    /// # Arguments
    /// 
    /// * `id` - unique identifier for the worker
    /// * `receiver` - shared receiver for receiving jobs
    fn new(id: u32, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let payload = receiver.lock().unwrap().recv();
            match payload {
                Ok(job) => {
                    println!("Worker {id} is on it!!");
                    job();
                }
                Err(msg) => {
                    println!("Worker {id} disconnecting: {msg}");
                    break;
                }
            }
        });
        Worker { id, thread: Some(thread) }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>
}

impl ThreadPool {
    pub fn workers(&self) -> &Vec<Worker> {
        &self.workers
    }

    pub fn sender(&self) -> &mpsc::Sender<Job> {
        self.sender.as_ref().unwrap()
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
        self.sender().send(Box::new(f)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("byebye -love, Worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
