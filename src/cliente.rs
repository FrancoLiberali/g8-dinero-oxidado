extern crate csv;

use std::{
    fs::File,
    sync::{Arc, Mutex, atomic::{AtomicU32, Ordering}},
};
use csv::Writer;
use rand::{Rng, prelude::StdRng};
use crate::transaccion::{Transaccion, TipoTransaccion};

pub struct Cliente {
    file: Arc<Mutex<Writer<File>>>,
    id_cliente: u32,
    n_transaccion: Arc<AtomicU32>,
    rng: Arc<Mutex<StdRng>>,
    n_operaciones: u32
}

impl Cliente {
    pub fn new(file: Arc<Mutex<Writer<File>>>, 
               id_cliente: u32, 
               n_transaccion: Arc<AtomicU32>,
               rng: Arc<Mutex<StdRng>>,
               n_operaciones: u32) -> Self {
        Self { 
            file,
            id_cliente,
            n_transaccion,
            rng,
            n_operaciones
        }
    }

    pub fn operar(&self) {
        let mut rng = self.rng.lock().expect("posioned rng");

        for _ in 0..self.n_operaciones {
            let tipo = if rng.gen() {
                TipoTransaccion::CashIn
            } else {
                TipoTransaccion::CashOut
            };

            let monto = rng.gen::<u16>() as f32 / 1000.0;
            let id_cliente: u32 = self.id_cliente;
            let timestamp: u32 = rng.gen();
            
            let mut file = self.file.lock()
                .expect("transactions file poisoned");
            file.serialize(Transaccion {
                id_transaccion: self.n_transaccion.fetch_add(1, Ordering::SeqCst), 
                id_cliente, 
                timestamp, 
                tipo,
                monto
            }).unwrap();
        }
    }
}
