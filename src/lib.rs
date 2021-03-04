mod worker;

use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

pub type Job = Box<dyn FnBox + Send + 'static>;

pub enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    threads: Vec<worker::Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let mut threads = Vec::with_capacity(size);
        let (sender, recevier) = mpsc::channel();
        let recevier = Arc::new(Mutex::new(recevier));

        for i in 0..size {
            threads.push(worker::Worker::new(i, Arc::clone(&recevier)));
        }
        ThreadPool { threads, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.threads {
            self.sender.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.threads {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.handle.take() {
                thread.join().unwrap();
            }
        }
    }
}
