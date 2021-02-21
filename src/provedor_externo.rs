use std::{sync::Arc, sync::atomic::AtomicBool, sync::atomic::Ordering, time::Duration};

pub fn iniciar_hilo_provedor(clientes: Sender<u32>) -> JoinHandle<()> {

    let provedor_externo = ProvedorExterno::new(archivo_, n, transaccion_, 64);

    std::thread::spawn(move || {
        cliente.operar();
    })
}

pub struct ProvedorExterno {
    clientes: Sender<u32>,
    apagado: AtomicBool,
}

impl ProvedorExterno {

    pub fn new(clientes: Sender<u32>) -> Self {
        Self { 
            clientes,
            apagado: AtomicBool::new(false),
        }
    }

    pub fn crear_hashes(&self){

        while !self.apagado.load(Ordering::SeqCst) {

            //crearHash
            let hash: u32 = 2; // crear funcion de hashing copada
            clientes.send(hash);

        }
    }
        
    pub fn cerrar(&self) {
        self.apagado.store(true, Ordering::SeqCst);
    }
}
