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
    thread::JoinHandle, 
};

use crate::transaccion::{Transaccion, TransaccionAutorizada};

pub fn iniciar_hilos_workers(n_workers: u32, 
                             provedor_ext: Arc<Mutex<Receiver<u32>>>,
                             transacciones: Arc<Mutex<Receiver<Transaccion>>>,
                             autorizador: Sender<TransaccionAutorizada>)
                             -> Vec<JoinHandle<()>> {
    let mut handles = vec![];
    
    for _n in 0..n_workers {
        let mut worker = Worker::new((&provedor_ext).clone(),
                                 (&transacciones).clone(),
                                 (&autorizador).clone());

        handles.push(std::thread::spawn(move || {
            worker.procesar();
        }));
    }
    handles
}

pub struct Worker {
    provedor_ext: Arc<Mutex<Receiver<u32>>>,
    transacciones: Arc<Mutex<Receiver<Transaccion>>>,
    autorizador: Sender<TransaccionAutorizada>,
    apagado: AtomicBool,
}

impl Worker {

    pub fn new(provedor_ext: Arc<Mutex<Receiver<u32>>>, 
               transacciones: Arc<Mutex<Receiver<Transaccion>>>, 
               autorizador: Sender<TransaccionAutorizada>) -> Self {
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
            
            let transaccion = match prov_transaccion.recv(){
                Ok(tran) => tran,
                Err(_) => {
                    println!("termino");
                    break;
                }, //se cerro el tx
            };
            let hash = provedor.recv().unwrap();

            //agregar hash
            let transaccion_autorizada = TransaccionAutorizada::new(transaccion, hash);
            //enviar hash
            self.autorizador.send(transaccion_autorizada).unwrap();

            drop(provedor);
            drop(prov_transaccion);
        }
        println!("me voy");
    }
    pub fn cerrar(&self) {
        self.apagado.store(true, Ordering::SeqCst);
    }
}