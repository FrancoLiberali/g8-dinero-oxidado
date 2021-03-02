use std::{sync::{Arc, mpsc::Receiver}, thread, thread::JoinHandle, time::SystemTime};
use csv::Writer;

use crate::{
    logger::TaggedLogger,
    transaccion::{TipoTransaccion, TransaccionAutorizada, TransaccionExitosa},
    cliente::Cliente,
};

const ARCHIVO_SALDOS: &str = "saldos.csv";

pub struct WorkerFinal {
    log: TaggedLogger,
    rx_transacciones_validadas: Receiver<TransaccionAutorizada>,
    clientes: Arc<Vec<Arc<Cliente>>>
}

impl WorkerFinal {
    pub fn iniciar(log: TaggedLogger,
                   rx_transacciones_validadas: Receiver<TransaccionAutorizada>,
                   clientes: Arc<Vec<Arc<Cliente>>>)
        -> JoinHandle<()>
    {
        thread::spawn(move || {
            let worker = Self {
                log,
                rx_transacciones_validadas,
                clientes
            };

            worker.procesar_transacciones();
        })
    }

    fn procesar_transacciones(&self) {
        self.log.write("Worker final iniciado");
        let mut writer = Writer::from_path(ARCHIVO_SALDOS).expect("El archivo de saldos finales no puedo ser abierto");

        while let Some(transaccion_autorizada) = self.obtener_transaccion() {
            self.log.write(&*format!("Transacción recibida: {}", transaccion_autorizada));
            let cliente_id = transaccion_autorizada.transaccion.id_cliente;
            let cliente_objetivo = self.clientes.iter().find( |&cliente| cliente.id == cliente_id).expect(&*format!("No se encuentra cliente con id {}", cliente_id));
            let monto = transaccion_autorizada.transaccion.monto;
            match transaccion_autorizada.transaccion.tipo {
                TipoTransaccion::CashIn => cliente_objetivo.cash_in(monto),
                TipoTransaccion::CashOut => cliente_objetivo.cash_out(monto),
            }
            self.log.write(&*format!("Transacción procesada: {}", transaccion_autorizada));

            let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("SystemTime before UNIX EPOCH!").as_millis();
            writer.serialize(TransaccionExitosa {
                transaccion: transaccion_autorizada,
                saldo_final: cliente_objetivo.get_saldo(),
                timestamp
            }).unwrap();
        }
        self.log.write("Worker final terminado");
    }

    fn obtener_transaccion(&self) -> Option<TransaccionAutorizada> {
        match self.rx_transacciones_validadas.recv() {
            Ok(t) => Some(t),
            Err(_) => None
        }
    }
}