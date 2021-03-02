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
mod ia;
mod worker_final;

use std::sync::{Arc, Mutex, mpsc::channel};

use clap::App;

use logger::{Logger, TaggedLogger};
use simulacion::simular_transacciones;
use procesador::Procesador;
use proveedor_externo::ProveedorExterno;
use worker::iniciar_workers_de_tipo;
use worker::TipoWorker;
use ia::iniciar_procesadores_ia;
use worker_final::WorkerFinal;

fn main()  {
    if let Err(e) = real_main() {
        println!("ERROR: {}", e);
    }
}

const CANTIDAD_DE_CLIENTES_DEFAULT: &str = "10";
const ARCHIVO_TRANSACCIONES: &str = "transacciones.csv";

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
    let semilla = 64 as u64; // TODO semilla aleatoria o por parametro

    let clientes = simular_transacciones(
        TaggedLogger::new("SIMULACION", logger.clone()),
        ARCHIVO_TRANSACCIONES,
        argumentos.value_of("Clientes").unwrap_or(CANTIDAD_DE_CLIENTES_DEFAULT).parse::<u32>().unwrap(),
        semilla
    ).expect("Error al generar el archivo de transacciones");

    log.write("Iniciando procesador del archivo");
    let (rx_cashin,
         rx_cashout,
         handle_procesador) = match Procesador::iniciar(ARCHIVO_TRANSACCIONES) {
        Ok(r) => r,
        Err(e) => return Err(format!("{}", e))
    };

    log.write("Iniciando proveedor externo de hashes");
    let (rx_hash, handle_hash) = ProveedorExterno::iniciar();
    let proveedor_autorizacion = Arc::new(Mutex::new(rx_hash));

    let (tx_transacciones_autorizadas, rx_transacciones_autorizadas_s) = channel();
    let rx_transacciones_autorizadas = Arc::new(Mutex::new(rx_transacciones_autorizadas_s));

    let (tx_transacciones_validadas, rx_transacciones_validadas) = channel();

    let handles_procesadores_ia = iniciar_procesadores_ia(
        3, // TODO usar valor de entrada de cantidad de procesadores ia
        rx_transacciones_autorizadas,
        tx_transacciones_validadas,
        semilla + 1,
        logger.clone()
    );

    log.write("Iniciando workers cash in");
    let handles_worker_cash_in = iniciar_workers_de_tipo(
        3, // TODO usar valor de entrada de cantidad de workers cash_in
        TipoWorker::CashIn,
        Arc::new(Mutex::new(rx_cashin)),
        proveedor_autorizacion.clone(),
        tx_transacciones_autorizadas.clone(),
        logger.clone()
    );
    log.write("Iniciando workers cash out");
    let handles_worker_cash_out = iniciar_workers_de_tipo(
        3, // TODO usar valor de entrada de cantidad de workers cash_out
        TipoWorker::CashOut,
        Arc::new(Mutex::new(rx_cashout)),
        proveedor_autorizacion,
        tx_transacciones_autorizadas,
        logger.clone()
    );

    let handle_worker_final = WorkerFinal::iniciar(
        TaggedLogger::new("WORKER FINAL", logger.clone()),
        rx_transacciones_validadas,
        clientes
    );

    // Esperar que finalicen todos los demas hilos
    handle_worker_final.join().expect("Cannot join worker thread");
    log.write("Todas las operaciones fueron procesadas. Finalizando.");

    for handle_procesador_ia in handles_procesadores_ia {
        handle_procesador_ia.join().expect("Cannot join ia thread");
    }

    // Esperar a que finalicen los workers
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
