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

use std::sync::{Arc, Mutex, mpsc::channel};

use clap::App;

use logger::{Logger, TaggedLogger};
use simulacion::simular_transacciones;
use procesador::Procesador;
use proveedor_externo::ProveedorExterno;
use worker::iniciar_workers_de_tipo;
use worker::TipoWorker;

fn main()  {
    if let Err(e) = real_main() {
        println!("ERROR: {}", e);
    }
}

const CANTIDAD_DE_CLIENTES_DEFAULT: &str = "10";

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
        argumentos.value_of("Clientes").unwrap_or(CANTIDAD_DE_CLIENTES_DEFAULT).parse::<u32>().unwrap(),
        64 // TODO semilla aleatoria
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

    let (tx_transacciones_autorizadas, rx_transacciones_autorizadas) = channel();

    log.write("Iniciando workers cash in");
    let handles_worker_cash_in = iniciar_workers_de_tipo(
        3, // TODO usar valor de entrada de cantidad de workers
        TipoWorker::CashIn,
        Arc::new(Mutex::new(rx_cashin)),
        proveedor_autorizacion.clone(),
        tx_transacciones_autorizadas.clone(),
        logger.clone()
    );
    log.write("Iniciando workers cash out");
    let handles_worker_cash_out = iniciar_workers_de_tipo(
        3, // TODO usar valor de entrada de cantidad de workers
        TipoWorker::CashOut,
        Arc::new(Mutex::new(rx_cashout)),
        proveedor_autorizacion,
        tx_transacciones_autorizadas,
        logger
    );

    for transaccion in rx_transacciones_autorizadas {
        log.write(&format!("Transacción autorizada: {}",
            transaccion.transaccion.id_transaccion))
    }

    log.write("Todas las operaciones fueron procesadas. Finalizando.");

    // Esperar a que finalicen primero los workers
    for handle_worker in handles_worker_cash_in {
        handle_worker.join().expect("Cannot join worker thread");
    }
    for handle_worker in handles_worker_cash_out {
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
