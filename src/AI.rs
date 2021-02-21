use std::{
    sync::mpsc::{Sender, Receiver},
    fs::File,
};
use std::{sync::{Mutex, Arc, atomic::{AtomicBool, Ordering}}, time::Duration};
use rand::{Rng, SeedableRng, prelude::StdRng};

use crate::worker::TransaccionAuth;

const PROBABILIDAD_DE_INVALIDA: f64 = 0.1; // 50%

pub struct AI {
    transacciones_auth: Arc<Mutex<Receiver<TransaccionAuth>>>,
    transacciones_validas: Sender<TransaccionAuth>,
    rng: Mutex<StdRng>,
    apagado: AtomicBool,
}

impl AI {

    pub fn new(transacciones_auth: Arc<Mutex<Receiver<TransaccionAuth>>>, transacciones_validas: Sender<TransaccionAuth>, semilla: u64) -> Self {
       Self { 
            transacciones_auth,
            transacciones_validas,
            rng: Mutex::new(StdRng::seed_from_u64(semilla)),
            apagado: AtomicBool::new(false),
       }
   }

    pub fn detectar_lavado(&mut self){

        while !self.apagado.load(Ordering::SeqCst) {

            let transacciones = self.transacciones_auth.lock().unwrap();
            let mut rng = self.rng.lock().expect("posioned rng");

            let transaccion_auth = transacciones.recv().unwrap();
            
            let valida: f64 = rng.gen();

            if  valida < PROBABILIDAD_DE_INVALIDA {
                // Descarto la transaccion
                continue;
            };
            // Transaccion valida
            self.transacciones_validas.send(transaccion_auth).unwrap();

            drop(rng);
            drop(transacciones);
        }
    }
    pub fn cerrar(&self) {
        self.apagado.store(true, Ordering::SeqCst);
    }
}