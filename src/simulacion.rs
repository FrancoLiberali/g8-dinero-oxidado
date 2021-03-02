use std::{
    sync::{Arc, Mutex, atomic::AtomicU32}
};
use csv::Writer;
use crate::{
    cliente::Cliente,
    logger::TaggedLogger
};

const CANTIDAD_OPERACIONES_POR_CLIENTE: u32 = 10;

// Simular transacciones de un dia entre los n_clientes clientes y guardar las transacción pendientes en el archivo
pub fn simular_transacciones(log: TaggedLogger,
                             ruta_archivo: &str,
                             n_clientes: u32,
                             semilla: u64) -> Result<Vec<Arc<Cliente>>, csv::Error> {
    log.write(&format!("Generando {} con {} clientes y {} operaciones por cliente.", 
        ruta_archivo, n_clientes, CANTIDAD_OPERACIONES_POR_CLIENTE));

    let n_transaccion = Arc::new(AtomicU32::new(1));

    // Crear los clientes
    let mut clientes = vec![];
    for n in 0..n_clientes {
        let cliente = Arc::new(
            Cliente::new(
                n + 1,
                n_transaccion.clone()
            )
        );

        clientes.push(cliente);
    }
    
    // Abro archivo de transacciones pendientes y hacer que los clientes realicen transacciones entre ellos
    let archivo = Arc::new(Mutex::new(Writer::from_path(ruta_archivo)?));
    let mut handles = vec![];
    for cliente in &clientes {
        let archivo_c = archivo.clone();
        let cliente_c = cliente.clone();

        handles.push(std::thread::spawn(move || {
            cliente_c.operar(
                archivo_c, 
                CANTIDAD_OPERACIONES_POR_CLIENTE,
                semilla
            );
        }));
    }

    log.write("Esperando a que termine la simulación");
    // Esperar a que todos los clientes terminen sus operaciones del día
    for cliente_handle in handles {
        cliente_handle.join().expect("no se pudo joinear hilo de cliente");
    }

    log.write("Simulación terminada");
    Ok(clientes)
}
