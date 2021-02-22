extern crate csv;

use std::{
    sync::mpsc::Sender,
    fs::File,
};
use csv::Reader;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Transaccion {
    Transaction: u32,
    User_id: u32,
    Timestamp: u32,
    Type: String,
    Amount: f32,
}

pub struct Procesador {
    file: Reader<File>,
    cashin: Sender<Transaccion>,
    cashout: Sender<Transaccion>,
}

impl Procesador {

    pub fn new(file: String, cashin: Sender<Transaccion>, cashout: Sender<Transaccion>) -> Self {
       Self { 
           file: csv::Reader::from_path(file).unwrap(),
           cashin,
           cashout,
       }
   }

   pub fn procesar(&mut self){
    for result in self.file.deserialize() {
        let record: Transaccion = result.unwrap();
        //println!("{:?}", record);
        if record.Type == "cash_in"{
            self.cashin.send(record).unwrap();
        } else {
            self.cashout.send(record).unwrap();
        }
    }
   }

}