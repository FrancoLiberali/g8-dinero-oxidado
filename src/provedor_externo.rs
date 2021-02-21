use std::{
    sync::{
        Arc,
        atomic::AtomicBool,
        atomic::Ordering,
        mpsc::Sender,
    },
    thread::JoinHandle,
    time::Duration,
    };

pub fn iniciar_hilo_provedor(clientes: Sender<u32>) -> JoinHandle<()> {

    let provedor_externo = ProvedorExterno::new(clientes);

    std::thread::spawn(move || {
        provedor_externo.crear_hashes();
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
            self.clientes.send(hash).unwrap();

        }
    }
        
    pub fn cerrar(&self) {
        self.apagado.store(true, Ordering::SeqCst);
    }
}
