extern crate rand;
extern crate csv;
extern crate serde;
extern crate clap;

mod cliente;
mod procesador;
mod logger;
mod proveedor_externo;
mod worker;
mod transaccion;
mod simulacion;
//mod AI;

use std::sync::{Arc, Mutex};

use clap::App;

use logger::{Logger, TaggedLogger};
use simulacion::simular_transacciones;
use procesador::Procesador;
use proveedor_externo::ProveedorExterno;
use worker::iniciar_workers;

fn main()  {
    if let Err(e) = real_main() {
        println!("ERROR: {}", e);
    }
}

fn real_main() -> Result<(), String> {
    // Parser de argumentos 
    let yaml = clap::load_yaml!("cli.yml");
    let argumentos = App::from_yaml(yaml).get_matches();

    // Inicializo el logger
    let logger = Arc::new(if argumentos.is_present("Debug") {
        Logger::new_to_file("debug.txt").expect("No se pudo crear el archivo de log.")
    } else {
        Logger::new_to_stdout()
    });

    let log = TaggedLogger::new("CONTROLADOR", logger.clone());

    log.write("Simulando transacciones");
    simular_transacciones(
        TaggedLogger::new("SIMULACION", logger.clone()),
        "transacciones.csv", 
        argumentos.value_of("Clientes").unwrap_or("10").parse::<u32>().unwrap(), 
        64
    ).expect("Error al generar el archivo de transacciones");

    log.write("Iniciando procesador del archivo");
    let (rx_cashin, 
         rx_cashout, 
         handle_procesador) = match Procesador::iniciar("transacciones.csv") {
        Ok(r) => r,
        Err(e) => return Err(format!("{}", e))
    };

    log.write("Iniciando proveedor externo de hashes");
    let (rx_hash, handle_hash) = ProveedorExterno::iniciar();

    let proveedor_autorizacion = Arc::new(Mutex::new(rx_hash));
    let proveedores_transaccion = vec![
        Arc::new(Mutex::new(rx_cashin)),
        Arc::new(Mutex::new(rx_cashout))
    ];

    log.write("Iniciando workers");
    let (rx_auth, handles_worker) = iniciar_workers(
        3,
        &proveedores_transaccion,
        proveedor_autorizacion,
        logger
    );

    for transaccion in rx_auth {
        log.write(&format!("Transacción autorizada: {}", 
            transaccion.transaccion.id_transaccion))
    }

    log.write("Todas las operaciones fueron procesadas. Finalizando.");

    // Esperar a que finalicen primero los workers
    for handle_worker in handles_worker {
        handle_worker.join().expect("Cannot join worker thread");
    }
    log.write("Todos los workers finalizaron");
    
    // Detener el hasher
    // Podría bloquearse si rx_hash no se cierra antes
    handle_hash.join().expect("Cannot join hasher thread");
    log.write("El proveedor externo finalizó");

    // Esperar a que termine el procesador
    // Podría bloquearse si rx_cashin/rx_cashout no se cierran antes
    handle_procesador.join().expect("Cannot join processor thread");
    log.write("El procesador de archivo terminó");

    log.write("Terminado");
    Ok(())
}
