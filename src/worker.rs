use std::{
    sync::{
        mpsc,
        mpsc::{Sender, Receiver},
        Arc, Mutex,
    },
    thread,
    thread::JoinHandle,
    fmt,
};

use crate::{
    logger::{Logger, TaggedLogger},
    transaccion::{HashAutorizacion, Transaccion, TransaccionAutorizada}
};

#[derive(Debug)]
pub enum TipoWorker {
    CashIn,
    CashOut
}

impl fmt::Display for TipoWorker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Inicia n_workers del tipo tipo_worker
pub fn iniciar_workers_de_tipo(n_workers: u32,
                       tipo_worker: TipoWorker,
                       rx_transacciones: Arc<Mutex<Receiver<Transaccion>>>,
                       proveedor_autorizacion: Arc<Mutex<Receiver<HashAutorizacion>>>,
                       tx_transacciones_autorizadas: Sender<TransaccionAutorizada>,
                       logger: Arc<Logger>)
    -> Vec<JoinHandle<()>>
{
    let mut handles_worker = vec![];
    for worker_id in 0..n_workers {
        handles_worker.push(
            Worker::iniciar(
                TaggedLogger::new(&format!("WORKER {} {}", tipo_worker, worker_id), logger.clone()),
                rx_transacciones.clone(),
                proveedor_autorizacion.clone(),
                tx_transacciones_autorizadas.clone()
            )
        );
    }

    handles_worker
}

pub struct Worker {
    log: TaggedLogger,
    rx_transacciones: Arc<Mutex<Receiver<Transaccion>>>,
    proveedor_autorizacion: Arc<Mutex<Receiver<HashAutorizacion>>>,
    tx_transacciones_autorizadas: Sender<TransaccionAutorizada>,
}

impl Worker {
    pub fn iniciar(log: TaggedLogger,
                   rx_transacciones: Arc<Mutex<Receiver<Transaccion>>>,
                   proveedor_autorizacion: Arc<Mutex<Receiver<HashAutorizacion>>>,
                   tx_transacciones_autorizadas: Sender<TransaccionAutorizada>)
        -> JoinHandle<()>
    {
        thread::spawn(move || {
            let worker = Self {
                log,
                rx_transacciones,
                proveedor_autorizacion,
                tx_transacciones_autorizadas
            };

            worker.procesar();
        })
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
            self.log.write(&*format!("{}", transaccion_autorizada));

            self.enviar_transaccion_autorizada(transaccion_autorizada);
        }
        self.log.write("Worker terminado");
    }

    fn obtener_transaccion(&self) -> Option<Transaccion> {
        let proveedor = self
            .rx_transacciones
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
        self.tx_transacciones_autorizadas.send(transaccion_autorizada).expect("Channel cerrado");
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use super::*;
    use uuid::Uuid;
    use crate::transaccion::TipoTransaccion;

    #[test]
    fn worker_solicitia_hash_y_envia_transaccion_autorizada() {
        let id_transaccion = 2;
        let transaccion = Transaccion {
            id: id_transaccion,
            id_cliente: Uuid::new_v4(),
            timestamp: 112315846_128,
            tipo: TipoTransaccion::CashIn,
            monto: 123.33
        };

        let (tx_transacciones, rx_transacciones_) = channel();
        let rx_transacciones = Arc::new(Mutex::new(rx_transacciones_));
        let (tx_autorizacion, rx_autorizacion_) = channel();
        let rx_autorizacion = Arc::new(Mutex::new(rx_autorizacion_));
        let (tx_transacciones_autorizadas, rx_transacciones_autorizadas) = channel();

        tx_transacciones.send(transaccion).unwrap();
        let hash = Uuid::new_v4();
        tx_autorizacion.send(hash).unwrap();

        Worker::iniciar(crear_logger(),
                   rx_transacciones,
                   rx_autorizacion,
                   tx_transacciones_autorizadas);
        let recibida = rx_transacciones_autorizadas.recv().unwrap();
        assert_eq!(recibida.transaccion.id, id_transaccion);
        assert_eq!(recibida.autorizacion, hash);
    }

    fn crear_logger() -> TaggedLogger {
        TaggedLogger::new("WORKER", Arc::new(Logger::new_to_stdout()))
    }
}