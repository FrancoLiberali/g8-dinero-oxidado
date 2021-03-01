extern crate csv;

use std::{
    fs::File,
    sync::{Arc, Mutex, atomic::{AtomicU32, Ordering}},
};
use csv::Writer;
use rand::{Rng, prelude::StdRng};
use crate::transaccion::{Transaccion, TipoTransaccion};

pub struct Cliente {
    // TODO dejar solo id y saldo acá
    pub id: u32,
    n_transaccion: Arc<AtomicU32>,
    rng: Arc<Mutex<StdRng>>,
    saldo: Mutex<f32>
}

impl Cliente {
    pub fn new(
               id: u32, 
               n_transaccion: Arc<AtomicU32>,
               rng: Arc<Mutex<StdRng>>) -> Self {
        Self { 
            id,
            n_transaccion,
            rng,
            saldo: Mutex::new(0.0) // TODO poner un saldo inicial random
        }
    }

    pub fn operar(&self, file: Arc<Mutex<Writer<File>>>, n_operaciones: u32) {
        let mut rng = self.rng.lock().expect("posioned rng");

        for _ in 0..n_operaciones {
            let tipo = if rng.gen() {
                TipoTransaccion::CashIn
            } else {
                TipoTransaccion::CashOut
            };

            let monto = rng.gen::<u16>() as f32 / 1000.0;
            let id_cliente: u32 = self.id;
            let timestamp: u32 = rng.gen();
            
            let mut file = file.lock()
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

    pub fn cash_in(&self, monto: f32) {
        let mut saldo = self.saldo.lock().expect("poisoned");
        *saldo = *saldo + monto;
    }

    pub fn cash_out(&self, monto: f32) {
        let mut saldo = self.saldo.lock().expect("poisoned");
        *saldo = *saldo - monto;
    }
}
