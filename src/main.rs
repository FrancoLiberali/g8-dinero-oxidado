extern crate rand;
extern crate csv;
extern crate serde;
extern crate clap;

mod cliente;
mod procesador;
mod logger;
mod provedor_externo;
mod worker;
mod transaccion;
mod simulacion;
//mod AI;

use std::{
    sync::Arc,
    sync::Mutex,
    sync::mpsc::channel,
    thread,
};

use clap::App;

use logger::{Logger, TaggedLogger};
use simulacion::simular_transacciones;
use procesador::Procesador;

use provedor_externo::ProvedorExterno;
use worker::iniciar_hilos_workers;

fn main()  {
    if let Err(e) = real_main() {
        println!("ERROR: {}", e);
    }
}


fn real_main() -> Result<(), String> {
    // Parsea de argumentos 
    let yaml = clap::load_yaml!("cli.yml");
    let argumentos = App::from_yaml(yaml).get_matches();

    // Inicializo el logger
    let logger = Arc::new(if argumentos.is_present("Debug") {
        Logger::new_to_file("debug.txt").expect("No se pudo crear el archivo de log.")
    } else {
        Logger::new_to_stdout()
    });

    simular_transacciones(
        TaggedLogger::new("SIMULACION", logger.clone()),
        "transacciones.csv", 
        argumentos.value_of("Clientes").unwrap_or("10").parse::<u32>().unwrap(), 
        64
    ).expect("Error al generar el archivo de transacciones");
    //------------------------------------------Procesamiento del archivo----------------------------

    // inicializo canales 
    let (tx_in, rx_in) = channel();
    let (tx_out, rx_out) = channel();
    let (tx_hash, rx_hash) = channel();
    let (tx_auth, rx_auth) = channel();

    // Proceso de las entradas del archivo 
    let mut proc = Procesador::new(String::from("transacciones.csv"), tx_in, tx_out);

    let t_procesador = thread::spawn(move || {
        proc.procesar();
    });

    //inicializo el provedor externo    
    let tx_has_mut = Arc::new(Mutex::new(tx_hash)); // para que no llore el compilador 

    let provedor_externo = Arc::new(ProvedorExterno::new(tx_has_mut.clone()));
    let provedor_ext_ref = provedor_externo.clone();
    let t_provedor = std::thread::spawn(move || {
        provedor_ext_ref.crear_hashes();
    });

    // inicializo workers cashin y cashout
    let prov_hash = Arc::new(Mutex::new(rx_hash));
    let prov_hash_1 = prov_hash.clone();
    let prov_hash_2 = prov_hash.clone();

    let cash_in = Arc::new(Mutex::new(rx_in));
    let rx_cashin = cash_in.clone();

    let cash_out = Arc::new(Mutex::new(rx_out));
    let rx_cashout = cash_out.clone();

    let tx_auth_1 = tx_auth.clone();
    let tx_auth_2 = tx_auth.clone();
    let t_workers_in = iniciar_hilos_workers(3, prov_hash_1, rx_cashin, tx_auth_1);
    let t_workers_out = iniciar_hilos_workers(3, prov_hash_2, rx_cashout, tx_auth_2);

    // No se porque se queda trabajo en el for, se tendrian que cerrar los tx_auth
    // y seguir pero se queda ahi. Lo demas parece que funca
    println!("------------------------Operaciones Auth------------------------------");
    for transaccion in rx_auth{
        println!("{:?}", transaccion);
    }

    println!("------------------------Operaciones termine------------------------------");

    for worker_in in t_workers_in {
        //worker_in.cerrar();
        worker_in.join().expect("no se pudo joinear hilo in");
    }

    for worker_out in t_workers_out {
        //worker_out.cerrar();
        worker_out.join().expect("no se pudo joinear hilo out");
    }

    t_procesador.join().unwrap();

    provedor_externo.cerrar();
    t_provedor.join().unwrap();
    
    logger.close();
    
    Ok(())
}
