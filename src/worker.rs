use std::{
    sync::{
        mpsc,
        mpsc::{Sender, Receiver},
        Arc, Mutex,
    },
    thread,
    thread::JoinHandle, 
};

use crate::{
    logger::{Logger, TaggedLogger},
    transaccion::{HashAutorizacion, Transaccion, TransaccionAutorizada}
};


/// Inicia n_workers para cashout y n_workers para cashin
/// (en total habrá 2*n_workers hilos corriendo).
pub fn iniciar_workers(n_workers: u32, 
                       proveedores_transaccion: &[Arc<Mutex<Receiver<Transaccion>>>],
                       proveedor_autorizacion: Arc<Mutex<Receiver<HashAutorizacion>>>,
                       logger: Arc<Logger>)
    -> (Receiver<TransaccionAutorizada>, Vec<JoinHandle<()>>)
{
    let (autorizador, rx) = mpsc::channel();
    let mut handles_worker = vec![];
    let n_sources = proveedores_transaccion.len() as u32;
    for worker_id in 0..n_workers {
        let mut id = n_sources * worker_id;
        for src in proveedores_transaccion {
            handles_worker.push(
                Worker::iniciar(
                    TaggedLogger::new(&format!("WORKER {}", id), logger.clone()),
                    proveedor_autorizacion.clone(),
                    src.clone(),
                    autorizador.clone()
                )
            );

            id += 1;
        }
    }

    (rx, handles_worker)
}

pub struct Worker {
    log: TaggedLogger,
    proveedor_autorizacion: Arc<Mutex<Receiver<HashAutorizacion>>>,
    proveedor_transacciones: Arc<Mutex<Receiver<Transaccion>>>,
    autorizador: Sender<TransaccionAutorizada>,
}

impl Worker {
    pub fn iniciar(log: TaggedLogger,
                   proveedor_autorizacion: Arc<Mutex<Receiver<HashAutorizacion>>>,
                   proveedor_transacciones: Arc<Mutex<Receiver<Transaccion>>>,
                   autorizador: Sender<TransaccionAutorizada>)
        -> JoinHandle<()>
    {
        thread::spawn(move || {
            let worker = Self {
                log,
                proveedor_autorizacion,
                proveedor_transacciones,
                autorizador
            };

            worker.procesar();
        })
    }

    fn obtener_transaccion(&self) -> Option<Transaccion> {
        let proveedor = self
            .proveedor_transacciones
            .lock()
            .expect("Mutex de transacciones poisoned");
        
        match proveedor.recv() {
            Ok(t) => Some(t),
            Err(_) => None
        }
    }

    fn obtener_hash(&self) -> Result<HashAutorizacion, mpsc::RecvError> {
        let proveedor = self
            .proveedor_autorizacion
            .lock()
            .expect("Mutex de hashes poisoned");
        
        proveedor.recv()
    }

    fn enviar_transaccion_autorizada(&self, transaccion_autorizada: TransaccionAutorizada) {
        self.autorizador.send(transaccion_autorizada).expect("Channel cerrado");
    }

    fn procesar(&self) {
        self.log.write("Worker iniciado");
        while let Some(transaccion) = self.obtener_transaccion() {
            let hash = match self.obtener_hash() {
                Ok(h) => h,
                Err(_) => {
                    println!("Se cerró el hasheador");
                    break;
                }
            };

            let transaccion_autorizada = TransaccionAutorizada::new(
                transaccion, 
                hash
            );

            self.enviar_transaccion_autorizada(transaccion_autorizada);
        }
        self.log.write("Worker terminado");
    }
}