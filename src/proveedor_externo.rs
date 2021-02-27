use std::{sync::mpsc, thread};
use crate::transaccion::HashAutorizacion;

pub struct ProveedorExterno {
    clientes: mpsc::Sender<u32>,
}

impl ProveedorExterno {
    pub fn iniciar() -> (mpsc::Receiver<HashAutorizacion>, thread::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let proveedor = Self { clientes: tx };

            proveedor.crear_hashes();
        });

        (rx, handle)
    }

    pub fn crear_hashes(&self) {
        loop {
            let hash: HashAutorizacion = 2; // crear funcion de hashing copada

            if self.clientes.send(hash).is_err() {
                // Nadie más quiere hashes
                break;
            }
        }
    }
}
