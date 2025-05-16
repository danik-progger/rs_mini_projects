use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    task::Context,
};

use std::pin::Pin;

use futures::task::{ArcWake, waker_ref};

fn main() {
    let mut mini_tokio = MiniTokio::new();
    mini_tokio.spawn(async { hey().await });
    mini_tokio.run();
}
async fn hey() {
    println!("Hey!");
}
struct MiniTokio {
    queue: Receiver<Arc<Task>>,
    sender: Sender<Arc<Task>>,
}
struct Task {
    task_future: Mutex<TaskFuture>,
    sender: Sender<Arc<Task>>,
}
struct TaskFuture {
    future: Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>,
}
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.sender.send(arc_self.clone()).unwrap();
    }
}
impl MiniTokio {
    fn new() -> MiniTokio {
        let (sender, receiver) = channel();
        MiniTokio {
            queue: receiver,
            sender,
        }
    }

    fn spawn<F>(&mut self, f: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task_future = TaskFuture {
            future: Some(Box::pin(f)),
        };
        let task = Arc::new(Task {
            task_future: Mutex::new(task_future),
            sender: self.sender.clone(),
        });
        self.sender.send(task).unwrap();
    }

    fn run(&mut self) {
        while let Ok(task) = self.queue.recv() {
            let mut task_future = task.task_future.lock().unwrap();
            if let Some(mut future) = task_future.future.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                if future.as_mut().poll(context).is_pending() {
                    (*task_future).future = Some(future)
                }
            }
        }
    }
}
