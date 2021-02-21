use std::{
    sync::{
        mpsc::{Sender, Receiver},
        Arc,
        Mutex,
        atomic::{
            AtomicBool,
            Ordering
        },
    },
    fs::File,
};

use crate::procesador::Transaccion;

pub struct TransaccionAuth {
    transaccion: Transaccion,
    hash_auth: u32,
}

impl TransaccionAuth{
    pub fn new(transaccion: Transaccion, hash_auth: u32) -> Self {
        Self {
            transaccion,
            hash_auth,
        }
    }
}

pub struct Worker {
    provedor_ext: Arc<Mutex<Receiver<u32>>>,
    transacciones: Arc<Mutex<Receiver<Transaccion>>>,
    autorizador: Sender<TransaccionAuth>,
    apagado: AtomicBool,
}

impl Worker {

    pub fn new(provedor_ext: Arc<Mutex<Receiver<u32>>>, transacciones: Arc<Mutex<Receiver<Transaccion>>>, autorizador: Sender<TransaccionAuth>) -> Self {
       Self { 
            provedor_ext,
            transacciones,
            autorizador,
            apagado: AtomicBool::new(false),
       }
   }

    pub fn procesar(&mut self){

        while !self.apagado.load(Ordering::SeqCst) {

            let prov_transaccion = self.transacciones.lock().unwrap();
            let provedor = self.provedor_ext.lock().unwrap();
        
            let transaccion = prov_transaccion.recv().unwrap();
            let hash = provedor.recv().unwrap();

            //agregar hash
            let transaccion_autorizada = TransaccionAuth::new(transaccion, hash);
            //enviar hash
            self.autorizador.send(transaccion_autorizada).unwrap();

            drop(provedor);
            drop(prov_transaccion);
        }
    }
    pub fn cerrar(&self) {
        self.apagado.store(true, Ordering::SeqCst);
    }
}