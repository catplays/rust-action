use std::{sync::mpsc::channel, thread};


fn main() {
    thread_get()
}


fn thread_get() {
    // 通过channel向线程中传递参数
    let (tx, rx) = channel();
    let arr = vec![1];
    tx.send(arr.to_owned()).expect("Unable to send on channel");

    let receiver = thread::spawn(move || {
        let value = rx.recv().expect("Unable to receive from channel");
        println!("{:?}", value);
    });

    receiver.join().expect("The receiver thread has panicked");
}