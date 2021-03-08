use std::{sync::{
        mpsc::{Sender, Receiver},
        Arc, Mutex,
    }, thread, thread::JoinHandle, time::Duration};
use rand::{Rng, SeedableRng, prelude::StdRng};
use crate::{
    logger::{Logger, TaggedLogger},
    transaccion::TransaccionAutorizada
};

const TIEMPO_MAXIMO_IA: u64 = 25; // 25 millis
const PROBABILIDAD_DE_INVALIDA: f64 = 0.1; // 10%

/// Inicia n_procesadores de autorizacion ia
pub fn iniciar_procesadores_ia(n_procesadores: u32,
                               rx_transacciones_autorizadas: Arc<Mutex<Receiver<TransaccionAutorizada>>>,
                               tx_transacciones_validas: Sender<TransaccionAutorizada>,
                               semilla: u64,
                               logger: Arc<Logger>)
    -> Vec<JoinHandle<()>>
{
    let mut handles_procesadores_ia = vec![];
    let rng = Arc::new(Mutex::new(StdRng::seed_from_u64(semilla)));
    for procesador_id in 0..n_procesadores {
        handles_procesadores_ia.push(
            ProcesadorIA::iniciar(
                TaggedLogger::new(&format!("PROCESADOR IA {}", procesador_id), logger.clone()),
                rx_transacciones_autorizadas.clone(),
                tx_transacciones_validas.clone(),
                rng.clone()
            )
        );
    }

    handles_procesadores_ia
}

pub struct ProcesadorIA {
    log: TaggedLogger,
    rx_transacciones_autorizadas: Arc<Mutex<Receiver<TransaccionAutorizada>>>,
    tx_transacciones_validas: Sender<TransaccionAutorizada>,
    rng: Arc<Mutex<StdRng>>,
}

impl ProcesadorIA {
    pub fn iniciar(log: TaggedLogger,
                   rx_transacciones_autorizadas: Arc<Mutex<Receiver<TransaccionAutorizada>>>,
                   tx_transacciones_validas: Sender<TransaccionAutorizada>,
                   rng: Arc<Mutex<StdRng>>)
        -> JoinHandle<()>
    {
        thread::spawn(move || {
            let procesador = Self {
                log,
                rx_transacciones_autorizadas,
                tx_transacciones_validas,
                rng,
            };

            procesador.procesar_transacciones();
        })
    }

    fn procesar_transacciones(&self) {
        self.log.write("Procesador iniciado");
        while let Some(transaccion) = self.obtener_transaccion() {
            let validacion = self.detectar_lavado(transaccion);
            match validacion {
                Ok(transaccion_validada) => {
                    self.log.write(&*format!("Transacción validada: {}", transaccion_validada));
                    self.enviar_transaccion_validada(transaccion_validada)
                },
                Err(transaccion_invalidada) => self.log.write(&format!("Lavado de dinero detectado: {}", transaccion_invalidada)),
            }
        }
        self.log.write("Procesador terminado");
    }

    fn obtener_transaccion(&self) -> Option<TransaccionAutorizada> {
        let proveedor = self
            .rx_transacciones_autorizadas
            .lock()
            .expect("Mutex de transacciones poisoned");
        
        match proveedor.recv() {
            Ok(t) => Some(t),
            Err(_) => None
        }
    }

    fn detectar_lavado(&self, transaccion: TransaccionAutorizada) -> Result<TransaccionAutorizada, TransaccionAutorizada> {
        let mut rng = self.rng.lock().expect("posioned rng");
        thread::sleep(
            Duration::from_millis(
                rng.gen_range(0..TIEMPO_MAXIMO_IA) as u64
            )
        );
        let valida: f64 = rng.gen();
        if valida < PROBABILIDAD_DE_INVALIDA {
            Err(transaccion)
        } else {
            Ok(transaccion)
        }
    }

    fn enviar_transaccion_validada(&self, transaccion_validada: TransaccionAutorizada) {
        self.tx_transacciones_validas.send(transaccion_validada).expect("Channel cerrado");
    }
}