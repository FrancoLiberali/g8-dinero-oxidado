use std::{
    sync::{
        Arc,
        atomic::AtomicBool,
        atomic::Ordering,
        mpsc::Sender,
        Mutex,
    },
};

pub struct ProvedorExterno {
    clientes: Arc<Mutex<Sender<u32>>>,
    apagado: AtomicBool,
}

impl ProvedorExterno {

    pub fn new(clientes: Arc<Mutex<Sender<u32>>>) -> Self {
        Self { 
            clientes,
            apagado: AtomicBool::new(false),
        }
    }

    pub fn crear_hashes(&self){

        while !self.apagado.load(Ordering::SeqCst) {

            //crearHash
            let clientes = self.clientes.lock().unwrap();
            let hash: u32 = 2; // crear funcion de hashing copada
            clientes.send(hash).unwrap();

        }
    }
        
    pub fn cerrar(&self) {
        self.apagado.store(true, Ordering::SeqCst);
    }
}
