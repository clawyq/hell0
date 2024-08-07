use std::sync::{ Arc, Mutex, mpsc };
use std::thread;

pub struct Worker {
    pub(crate) id: u32,
    pub(crate) thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new Worker
    /// 
    /// # Arguments
    /// 
    /// * `id` - unique identifier for the worker
    /// * `receiver` - shared receiver for receiving jobs
    pub fn new(id: u32, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
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

pub type Job = Box<dyn FnOnce() + Send + 'static>;
