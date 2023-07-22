use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

trait FnBox {
    fn call_box(self: Box<Self>);
    // Self: 이 트레잇을 구현한 타입
}

impl<F: FnOnce()> FnBox for F {
    //F에 트레잇을 구현
    fn call_box(self: Box<F>) {
        self()
    }
}

type Job = Box<dyn FnBox + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        // 메세지를 송수신할 채널 생성

        let receiver = Arc::new(Mutex::new(receiver));
        // Arc: 공유 소유권을 제공
		// Mutex: receiver에 한 번에 하나의 worker만 접근하도록 보장

        let mut workers = Vec::with_capacity(size);
        //with_capacity: 벡터 공간을 미리 할당, 
		// 갯수를 알고 있으면 new보다 효율적

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
            //스레드를 생성하고 벡터 내에 보관
        }

        ThreadPool {
            workers,
            sender,
        }
    }

    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let job = receiver.lock().unwrap().recv().unwrap();

                println!("Worker {} got a job; executing.", id);

                job.call_box();
            }
        });

        Worker {
            id,
            thread,
        }
    }
}