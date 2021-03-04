use crate::Message;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Worker {
    pub id: usize,
    pub handle: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        Worker {
            id,
            handle: Some(std::thread::spawn(move || loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job executing.", id);
                        job.call_box();
                    }
                    Message::Terminate => {
                        println!("Worker {} terminating!.", id);
                        break;
                    }
                }
                ()
            })),
        }
    }
}
