extern crate rand;
extern crate csv;
extern crate serde;

use std::{
    sync::Arc,
    sync::Mutex,
    sync::mpsc::channel,
    thread
};

use serde::{Serialize, Serializer};

enum TipoTransaccion {
    CashIn,
    CashOut
}

impl Serialize for TipoTransaccion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            TipoTransaccion::CashIn => "cash_in",
            TipoTransaccion::CashOut => "cash_out"
        })
    }
}

#[derive(Serialize)]
struct Transaccion {
    #[serde(rename = "Transaction")]
    id_transaccion: u32,
    #[serde(rename = "User_id")]
    id_cliente: u32,
    #[serde(rename = "Timestamp")]
    timestamp: u32,
    #[serde(rename = "Type")]
    tipo: TipoTransaccion,
    #[serde(rename = "Amount")]
    monto: f32
}

fn generar_transacciones(n_clientes: u32, n_transacciones: u32) {
    use rand::prelude::*;
    let mut rng = rand::thread_rng();
    let mut file = csv::Writer::from_path("transacciones.csv").unwrap();

    for id_transaccion in 1..(n_transacciones+1) {
        let id_cliente: u32 = rng.gen_range(0..n_clientes);
        let timestamp: u32 = rng.gen();
        let tipo = if rng.gen() {
            TipoTransaccion::CashIn
        } else {
            TipoTransaccion::CashOut
        };
        let monto = rng.gen::<u32>() as f32 / 1000.0;

        file.serialize(Transaccion {
            id_transaccion, id_cliente, timestamp, tipo, monto
        }).unwrap();
    }
    file.flush().unwrap();
}

fn main() {
    generar_transacciones(10, 1000);
    /*let (tx, rx) = channel();

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
    t2.join().unwrap();*/
}
