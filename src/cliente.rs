extern crate csv;

use std::{
    fs::File,
    sync::{Arc, Mutex},
    thread::JoinHandle, 
    thread,
    time::Duration,
};
use rand::{Rng, SeedableRng, prelude::StdRng};

use csv::Writer;

use serde::{Serialize, Serializer};

const PROBABILIDAD_DE_CHECK_IN: f64 = 0.5; // 50%
const CANTIDAD_DE_OPERACIONES: u32 = 100;

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



pub fn iniciar_hilos_clientes(n_clientes: u32, archivo: Arc<Mutex<Writer<File>>>) -> Vec<JoinHandle<()>> {
    let mut handles = vec![];
    let n_transaccion = Arc::new(Mutex::new(0));

    for n in 0..n_clientes {
        let transaccion_ = n_transaccion.clone();
        let archivo_ = archivo.clone();

        let cliente = Cliente::new(archivo_, n, transaccion_, 64);

        handles.push(std::thread::spawn(move || {
            cliente.operar();
        }));
    }
    handles
}


pub struct Cliente {
    file: Arc<Mutex<Writer<File>>>,
    id_cliente: u32,
    n_transaccion: Arc<Mutex<u32>>,
    rng: Mutex<StdRng>,
}

impl Cliente {

    pub fn new(file: Arc<Mutex<Writer<File>>>, id_cliente: u32, n_transaccion: Arc<Mutex<u32>>,
         semilla: u64) -> Self {
        Self { 
            file,
            id_cliente,
            n_transaccion,
            rng: Mutex::new(StdRng::seed_from_u64(semilla))
        }
    }

    pub fn operar(&self){

        let mut rng = self.rng.lock().expect("posioned rng");
        let transacciones: u32 = rng.gen_range(1..CANTIDAD_DE_OPERACIONES);

        for n in 1..transacciones {

            let es_check_in: f64 = rng.gen();

            let tipo = if  es_check_in < PROBABILIDAD_DE_CHECK_IN {
                TipoTransaccion::CashIn
            } else {
                TipoTransaccion::CashOut
            };

            let mut transaccion = self.n_transaccion.lock().expect("log poisoned");
            let id_transaccion = *transaccion;
            let monto = rng.gen::<u32>() as f32 / 1000.0;
            let id_cliente: u32 = self.id_cliente;
            
            let timestamp: u32 = rng.gen();

            let mut file = self.file.lock().expect("log poisoned");
            file.serialize(Transaccion {
                id_transaccion, id_cliente, timestamp, tipo, monto
            }).unwrap();

            *transaccion += 1;
            drop(id_transaccion);
            //esta linea es para probar que se intercalan las los hilos
            thread::sleep(Duration::from_millis(10));
        }
    }
}
