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
        let mut writer = Writer::from_path(ARCHIVO_SALDOS).expect("El archivo de saldos finales no pudo ser abierto");

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

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, atomic::AtomicU32, mpsc::channel};

    use super::*;
    use csv::StringRecord;
    use rand::{SeedableRng, prelude::StdRng};
    use crate::{logger::Logger, transaccion::{Transaccion, TransaccionAutorizada, TipoTransaccion}};
    use uuid::Uuid;

    #[test]
    fn realizar_transferencia_no_pasa_saldo_de_un_cliente_a_otro_si_queda_sin_procesar() {
        let cliente = Arc::new(crear_cliente());

        let (tx_transacciones_validadas, rx_transacciones_validadas) = channel();

        let saldo_anterior = cliente.get_saldo();
        let transaccion_id = 2;
        let monto = 123.33;
        let transaccion = Transaccion {
            id: transaccion_id,
            id_cliente: cliente.id,
            timestamp: 112315846_128,
            tipo: TipoTransaccion::CashIn,
            monto
        };
        let hash = Uuid::new_v4();
        let transaccion_autorizada = TransaccionAutorizada {
            transaccion: transaccion,
            autorizacion: hash
        };

        tx_transacciones_validadas.send(transaccion_autorizada).unwrap();
        let handle = WorkerFinal::iniciar(crear_logger(),
                   rx_transacciones_validadas,
                   Arc::new(vec![cliente.clone()]));
        drop(tx_transacciones_validadas);
        handle.join().unwrap();

        let mut reader = csv::Reader::from_path(ARCHIVO_SALDOS).unwrap();
        let mut record = StringRecord::new();
        reader.read_record(&mut record).unwrap();
        assert_eq!(record[0], transaccion_id.to_string());
        assert_eq!(record[1], cliente.id.to_hyphenated().to_string());
        assert_eq!(record[3], *"cash_in");
        assert_eq!(record[4], monto.to_string());
        assert_eq!(record[5], hash.to_hyphenated().to_string());
        assert_eq!(record[7], (saldo_anterior + monto).to_string());
    }

    fn crear_cliente() -> Cliente {
        crear_cliente_con_semilla(2_64)
    }

    fn crear_cliente_con_semilla(semilla: u64) -> Cliente {
        Cliente::new(
            Uuid::new_v4(),
               Arc::new(AtomicU32::new(1)),
               Arc::new(Mutex::new(StdRng::seed_from_u64(semilla)))
        )
    }

    fn crear_logger() -> TaggedLogger {
        TaggedLogger::new("WORKER FINAL", Arc::new(Logger::new_to_stdout()))
    }
}