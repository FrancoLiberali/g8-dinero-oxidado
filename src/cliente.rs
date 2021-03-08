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
    rng: Arc<Mutex<StdRng>>,
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
               rng: Arc<Mutex<StdRng>>) -> Self {
        let saldo_inicial = rng.lock().expect("poisoned").gen_range(SALDO_INICIAL_MINIMO..SALDO_INICIAL_MAXIMO);
        Self {
            id,
            saldo: Mutex::new(saldo_inicial),
            n_transaccion,
            rng
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
            cliente_destino.escribir_transaccion_pendiente(TipoTransaccion::CashIn, monto, archivo);
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
        *saldo += monto;
    }

    pub fn cash_out(&self, monto: f32) {
        let mut saldo = self.saldo.lock().expect("poisoned");
        *saldo -= monto;
    }

    pub fn get_saldo(&self) -> f32 {
        let saldo = self.saldo.lock().expect("poisoned");
        *saldo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::StringRecord;
    use rand::SeedableRng;
    use uuid::Uuid;

    #[test]
    fn saldo_inicial_esta_entre_el_minimo_y_maximo() {
        let cliente = crear_cliente();
        assert!(cliente.get_saldo() >= SALDO_INICIAL_MINIMO);
        assert!(cliente.get_saldo() <= SALDO_INICIAL_MAXIMO);
    }

    #[test]
    fn realizar_transferencia_pasa_saldo_de_un_cliente_a_otro_cuando_no_queda_por_procesar() {
        let ruta_archivo_tests = "archivo.csv";
        let cliente1 = crear_cliente();
        let cliente2 = Arc::new(crear_cliente());
        let saldo1 = cliente1.get_saldo();
        let saldo2 = cliente2.get_saldo();
        let monto = 105.25;
        cliente1.realizar_transferencia(
            &cliente2,
            monto,
            Arc::new(Mutex::new(Writer::from_path(ruta_archivo_tests).unwrap()))
        );
        assert_eq!(cliente1.get_saldo(), saldo1 - monto);
        assert_eq!(cliente2.get_saldo(), saldo2 + monto);
    }

    #[test]
    fn realizar_transferencia_no_pasa_saldo_de_un_cliente_a_otro_si_queda_sin_procesar() {
        let ruta_archivo_tests = "archivo_tests.csv";
        let cliente1 = crear_cliente_con_semilla(21_64);
        let cliente2 = Arc::new(crear_cliente());
        let saldo1 = cliente1.get_saldo();
        let saldo2 = cliente2.get_saldo();
        let monto = 105.25;
        cliente1.realizar_transferencia(
            &cliente2,
            monto,
            Arc::new(Mutex::new(Writer::from_path(ruta_archivo_tests).unwrap()))
        );
        assert_eq!(cliente1.get_saldo(), saldo1);
        assert_eq!(cliente2.get_saldo(), saldo2);
        let mut reader = csv::Reader::from_path(ruta_archivo_tests).unwrap();
        let mut record = StringRecord::new();
        reader.read_record(&mut record).unwrap();
        assert_eq!(record[0], *"1");
        assert_eq!(record[1], cliente1.id.to_hyphenated().to_string());
        assert_eq!(record[3], *"cash_out");
        assert_eq!(record[4], monto.to_string());
        reader.read_record(&mut record).unwrap();
        assert_eq!(record[0], *"1");
        assert_eq!(record[1], cliente2.id.to_hyphenated().to_string());
        assert_eq!(record[3], *"cash_in");
        assert_eq!(record[4], monto.to_string());
    }

    fn crear_cliente() -> Cliente {
        crear_cliente_con_semilla(2_64)
    }

    fn crear_cliente_con_semilla(semilla: u64) -> Cliente {
        Cliente::new(
            Uuid::new_v4(),
               Arc::new(AtomicU32::new(1)),
               Arc::new(Mutex::new(StdRng::seed_from_u64(semilla)))
        )
    }
}
