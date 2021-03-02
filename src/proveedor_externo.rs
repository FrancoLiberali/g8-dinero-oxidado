use std::{sync::mpsc, thread};
use uuid::Uuid;

use crate::transaccion::HashAutorizacion;

pub struct ProveedorExterno {
    clientes: mpsc::Sender<HashAutorizacion>,
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
            let hash: HashAutorizacion = Uuid::new_v4();

            if self.clientes.send(hash).is_err() {
                // Nadie más quiere hashes
                break;
            }
        }
    }
}
