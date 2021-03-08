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
use rand::Rng;

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
const CANTIDAD_DE_IA_DEFAULT: &str = "10";
const CANTIDAD_DE_CASHIN_DEFAULT: &str = "10";
const CANTIDAD_DE_CASHOUT_DEFAULT: &str = "10";
const ARCHIVO_TRANSACCIONES: &str = "transacciones.csv";

fn real_main() -> Result<(), String> {
    // Parser de argumentos 
    let yaml = clap::load_yaml!("cli.yml");
    let argumentos = App::from_yaml(yaml).get_matches();

    let exe = &std::env::args().collect::<Vec<String>>()[0];
    let modo_debug = argumentos.is_present("Debug");
    let cantidad_clientes = argumentos.value_of("Clientes").unwrap_or(CANTIDAD_DE_CLIENTES_DEFAULT).parse::<u32>().unwrap();
    let cantidad_workers_ia = argumentos.value_of("Workers ia").unwrap_or(CANTIDAD_DE_IA_DEFAULT).parse::<u32>().unwrap();
    let cantidad_workers_cashin = argumentos.value_of("Workers cashin").unwrap_or(CANTIDAD_DE_CASHIN_DEFAULT).parse::<u32>().unwrap();
    let cantidad_workers_cashout = argumentos.value_of("Workers cashout").unwrap_or(CANTIDAD_DE_CASHOUT_DEFAULT).parse::<u32>().unwrap();

    let mut rng = rand::thread_rng();
    let semilla_simulaciones = argumentos
        .value_of("Semilla simulacion")
        .unwrap_or(&format!("{}", rng.gen::<u64>()))
        .parse::<u64>()
        .unwrap();
    let semilla_ia = argumentos
        .value_of("Semilla ia")
        .unwrap_or(&format!("{}", rng.gen::<u64>()))
        .parse::<u64>()
        .unwrap();

    // Inicializo el logger
    let logger = Arc::new(if modo_debug {
        Logger::new_to_file("debug.txt").expect("No se pudo crear el archivo de log.")
    } else {
        Logger::new_to_stdout()
    });

    let log = TaggedLogger::new("CONTROLADOR", logger.clone());
    log.write(&format!("Iniciando simulación con: {} -o {} -i {} -p {} -c {} -s {} -a {}", exe, cantidad_workers_cashout, cantidad_workers_cashin, cantidad_workers_ia, cantidad_clientes, semilla_simulaciones, semilla_ia));

    log.write("Simulando transacciones");

    let clientes = simular_transacciones(
        TaggedLogger::new("SIMULACION", logger.clone()),
        ARCHIVO_TRANSACCIONES,
        cantidad_clientes,
        semilla_simulaciones
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
        cantidad_workers_ia,
        rx_transacciones_autorizadas,
        tx_transacciones_validadas,
        semilla_ia,
        logger.clone()
    );

    log.write("Iniciando workers cash in");
    let handles_worker_cash_in = iniciar_workers_de_tipo(
        cantidad_workers_cashin,
        TipoWorker::CashIn,
        Arc::new(Mutex::new(rx_cashin)),
        proveedor_autorizacion.clone(),
        tx_transacciones_autorizadas.clone(),
        logger.clone()
    );
    log.write("Iniciando workers cash out");
    let handles_worker_cash_out = iniciar_workers_de_tipo(
        cantidad_workers_cashout,
        TipoWorker::CashOut,
        Arc::new(Mutex::new(rx_cashout)),
        proveedor_autorizacion,
        tx_transacciones_autorizadas,
        logger.clone()
    );

    let handle_worker_final = WorkerFinal::iniciar(
        TaggedLogger::new("WORKER FINAL", logger),
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
