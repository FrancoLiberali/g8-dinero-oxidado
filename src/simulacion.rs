use std::{
    sync::{Arc, Mutex, atomic::AtomicU32}
};
use rand::{SeedableRng, prelude::StdRng};
use csv::Writer;
use crate::{
    cliente::Cliente,
    logger::TaggedLogger
};

const CANTIDAD_OPERACIONES_POR_CLIENTE: u32 = 10;

pub fn simular_transacciones(log: TaggedLogger,
                             ruta_archivo: &str, 
                             n_clientes: u32,
                             semilla: u64) -> Result<(), csv::Error> {
    log.write(&format!("Generando {} con {} clientes y {} operaciones por cliente.", 
        ruta_archivo, n_clientes, CANTIDAD_OPERACIONES_POR_CLIENTE));
    
    // Abro archivo de transacciones
    let archivo = Arc::new(Mutex::new(Writer::from_path(ruta_archivo)?));
    
    // Simulo transacciones de un dia 
    let mut handles = vec![];
    let n_transaccion = Arc::new(AtomicU32::new(1));
    let rng = Arc::new(Mutex::new(StdRng::seed_from_u64(semilla)));

    for n in 0..n_clientes {
        let cliente = Cliente::new(
            archivo.clone(), 
            n + 1, 
            n_transaccion.clone(), 
            rng.clone(),
            CANTIDAD_OPERACIONES_POR_CLIENTE
        );

        handles.push(std::thread::spawn(move || {
            cliente.operar();
        }));
    }
    
    log.write("Esperando a que termine la simulación");
    // Esperar a que terminen todos los clientes
    for cliente in handles {
        cliente.join().expect("no se pudo joinear hilo de cliente");
    }

    log.write("Simulación terminada");
    Ok(())
}
