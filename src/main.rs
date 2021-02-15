use std::{
    sync::Arc,
    sync::Mutex,
    sync::mpsc::channel,
    thread
};


fn main() {
    let (tx, rx) = channel();

    let rx_2 = Arc::new(Mutex::new(rx));
    let rx_3 = rx_2.clone();
    let t1 = thread::spawn(move || {
        loop {
            println!("t1 esperando mutex");
            let rx_t1 = rx_2.lock().expect("poison");
            println!("t1 agarré mutex");
            let rcv = rx_t1.recv().unwrap();
            println!("t1 recibió: {}", rcv);
            if rcv > 90000 { println!("t1 bai"); break; }
        }
    });

    let t2 = thread::spawn(move || {
        loop {
            println!("t2 esperando mutex");
            let rx_t2 = rx_3.lock().expect("poison");
            println!("t2 agarré mutex");
            let rcv = rx_t2.recv().unwrap();
            println!("t2 recibió: {}", rcv);
            if rcv > 90000 { println!("t2 bai"); break; }
        }
    });

    for i in 0..100000 {
        tx.send(i).unwrap();
        // thread::sleep(Duration::from_millis(10));
    }

    t1.join().unwrap();
    t2.join().unwrap();
}
