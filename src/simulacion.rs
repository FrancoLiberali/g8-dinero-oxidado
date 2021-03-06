use std::{
    sync::{Arc, Mutex, atomic::AtomicU32}
};
use csv::Writer;
use rand::{SeedableRng, prelude::StdRng};
use uuid::Uuid;
use crate::{
    cliente::Cliente,
    logger::TaggedLogger
};

// Simular transacciones de un dia entre los n_clientes clientes y guardar las transacción pendientes en el archivo
pub fn simular_transacciones(log: TaggedLogger,
                             ruta_archivo: &str,
                             n_clientes: u32,
                             semilla: u64) -> Result<Arc<Vec<Arc<Cliente>>>, csv::Error> {

    log.write(&format!("Generando {} con {} clientes", ruta_archivo, n_clientes));
    let n_transaccion = Arc::new(AtomicU32::new(1));

    // Crear los clientes
    let mut clientes = vec![];
    let rng = Arc::new(Mutex::new(StdRng::seed_from_u64(semilla)));
    for _ in 0..n_clientes {
        let cliente = Arc::new(
            Cliente::new(
                Uuid::new_v4(),
                n_transaccion.clone(),
                rng.clone()
            )
        );

        clientes.push(cliente);
    }
    let clientes_arc = Arc::new(clientes);

    // Abro archivo de transacciones pendientes y hacer que los clientes realicen transacciones entre ellos
    let archivo = Arc::new(Mutex::new(Writer::from_path(ruta_archivo)?));
    let mut handles = vec![];
    for cliente in &*clientes_arc {
        let archivo_c = archivo.clone();
        let cliente_c = cliente.clone();
        let clientes_c = clientes_arc.clone();

        handles.push(std::thread::spawn(move || {
            cliente_c.operar(
                clientes_c,
                archivo_c
            );
        }));
    }

    log.write("Esperando a que termine la simulación");
    // Esperar a que todos los clientes terminen sus operaciones del día
    for cliente_handle in handles {
        cliente_handle.join().expect("no se pudo joinear hilo de cliente");
    }

    log.write("Simulación terminada");
    Ok(clientes_arc)
}
