extern crate rand;
extern crate csv;
extern crate serde;
extern crate clap;

mod cliente;
mod procesador;
mod logger;
//mod provedor_externo;
//mod worker;
//mod AI;

use std::{
    sync::Arc,
    sync::Mutex,
    sync::mpsc::channel,
    thread,
    fs::File,
};

use clap::App;

use logger::{Logger, TaggedLogger};
use cliente::iniciar_hilos_clientes;
use procesador::Procesador;

//use provedor_externo::ProvedorExterno;
//use worker::Worker;


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

    let log = TaggedLogger::new("ADMIN", logger.clone());
    log.write(&format!("Iniciando simulación con: {}", "onda"));//args.as_str()));

    // Abro archivo de transacciones
    let archivo = Arc::new(Mutex::new(csv::Writer::from_path("transacciones.csv").unwrap()));
    
    // Simulo transacciones de un dia 
    let archivo_ = archivo.clone();
    let numero_de_clientes = argumentos.value_of("Clientes").unwrap_or("10").parse::<u32>().unwrap();
    let clientes_threads = iniciar_hilos_clientes(numero_de_clientes, archivo_); 

    for cliente in clientes_threads {
        cliente.join().expect("no se pudo joinear hilo de cliente");
    }

    let mut file = archivo.lock().expect("log mutex poisoned");
    file.flush().expect("Error al flushear el log");

    let (tx, rx) = channel();
    let (tx2, rx2) = channel();

    let mut proc = Procesador::new(String::from("transacciones.csv"), tx, tx2);
    proc.procesar();

    println!("------------------------------Operaciones Cashin------------------------------");
    for cashin in rx{
        println!("{:?}", cashin);
    }

    println!("------------------------------Operaciones Cashout------------------------------");
    for cashout in rx2{
        println!("{:?}", cashout);
    }
    
    logger.close();
    
    Ok(())

    /*let (tx, rx) = channel();

    let rx_2 = Arc::new(Mutex::new(rx));
    let rx_3 = rx_2.clone();
    let t1 = thread::spawn(move || {
        loop {
            println!("t1 esperando mutex");
            let rx_t1 = rx_2.lock().expect("poison");
            println!("t1 agarré mutex");
            let rcv = rx_t1.recv().unwrap();
            println!("t1 recibió: {}", rcv);
            if rcv > 90000 { println!("t1 bai"); break; }
        }
    });

    let t2 = thread::spawn(move || {
        loop {
            println!("t2 esperando mutex");
            let rx_t2 = rx_3.lock().expect("poison");
            println!("t2 agarré mutex");
            let rcv = rx_t2.recv().unwrap();
            println!("t2 recibió: {}", rcv);
            if rcv > 90000 { println!("t2 bai"); break; }
        }
    });

    for i in 0..100000 {
        tx.send(i).unwrap();
        // thread::sleep(Duration::from_millis(10));
    }

    t1.join().unwrap();
    t2.join().unwrap();*/
}
