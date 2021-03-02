extern crate csv;

use std::{fs::File, sync::{Arc, Mutex, atomic::{AtomicU32, Ordering}}, time::SystemTime};
use csv::Writer;
use rand::{Rng, prelude::StdRng};
use uuid::Uuid;
use crate::transaccion::{Transaccion, TipoTransaccion};

pub struct Cliente {
    pub id: Uuid,
    saldo: Mutex<f32>,
    n_transaccion: Arc<AtomicU32>,
    rng: Mutex<StdRng>,
}

const SALDO_INICIAL_MINIMO: f32 = 100.0;
const SALDO_INICIAL_MAXIMO: f32 = 10000.0;
const CANTIDAD_MINIMA_OPERACIONES: u32 = 10;
const CANTIDAD_MAXIMA_OPERACIONES: u32 = 100;
const MONTO_MINIMO_TRANSFERENCIA: f32 = 10.0;
const MONTO_MAXIMO_TRANSFERENCIA: f32 = 1000.0;
const PROBABILIDAD_TRANSACCION_NO_PROCESADA: f64 = 0.1; // 10%

impl Cliente {
    pub fn new(
               id: Uuid,
               n_transaccion: Arc<AtomicU32>,
               mut rng: StdRng) -> Self {
        Self {
            id,
            saldo: Mutex::new(rng.gen_range(SALDO_INICIAL_MINIMO..SALDO_INICIAL_MAXIMO)),
            n_transaccion,
            rng: Mutex::new(rng)
        }
    }

    pub fn operar(&self, clientes: Arc<Vec<Arc<Cliente>>>, archivo: Arc<Mutex<Writer<File>>>) {
        let cantidad_operaciones = self.rng.lock().expect("poisoned").gen_range(CANTIDAD_MINIMA_OPERACIONES..CANTIDAD_MAXIMA_OPERACIONES);
        for _ in 0..cantidad_operaciones {
            let indice_cliente_destino = self.rng.lock().expect("poisoned").gen_range(0..clientes.len() - 1);
            let cliente_destino = &clientes[indice_cliente_destino];

            let monto_transferencia = self.rng.lock().expect("poisoned").gen_range(MONTO_MINIMO_TRANSFERENCIA..MONTO_MAXIMO_TRANSFERENCIA);
            self.realizar_transferencia(cliente_destino, monto_transferencia, archivo.clone());
        }

    }

    fn realizar_transferencia(&self, cliente_destino: &Arc<Cliente>, monto: f32, archivo: Arc<Mutex<Writer<File>>>) {
        let procesar_ahora: f64 = self.rng.lock().expect("poisoned").gen();
        if procesar_ahora < PROBABILIDAD_TRANSACCION_NO_PROCESADA {
            self.escribir_transaccion_pendiente(TipoTransaccion::CashOut, monto, archivo.clone());
            cliente_destino.escribir_transaccion_pendiente(TipoTransaccion::CashIn, monto, archivo.clone());
        } else {
            cliente_destino.cash_in(monto);
            self.cash_out(monto);
        }

    }

    fn escribir_transaccion_pendiente(&self, tipo: TipoTransaccion, monto: f32, archivo: Arc<Mutex<Writer<File>>>) {
        let mut archivo_w = archivo.lock().expect("transactions file poisoned");
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("SystemTime before UNIX EPOCH!").as_millis();
        archivo_w.serialize(Transaccion {
            id: self.n_transaccion.fetch_add(1, Ordering::SeqCst),
            id_cliente: self.id,
            timestamp,
            tipo,
            monto
        }).unwrap();
    }

    pub fn cash_in(&self, monto: f32) {
        let mut saldo = self.saldo.lock().expect("poisoned");
        *saldo = *saldo + monto;
    }

    pub fn cash_out(&self, monto: f32) {
        let mut saldo = self.saldo.lock().expect("poisoned");
        *saldo = *saldo - monto;
    }

    pub fn get_saldo(&self) -> f32 {
        let mut saldo = self.saldo.lock().expect("poisoned");
        *saldo
    }
}
