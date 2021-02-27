use std::{
    sync::mpsc::Sender,
    fs::File,
};
use csv::Reader;
use crate::transaccion::{Transaccion, TipoTransaccion};

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

    pub fn procesar(&mut self) {
        for registro in self.file.deserialize() {
            let transaccion: Transaccion = registro.unwrap();
            let channel = match transaccion.tipo {
                TipoTransaccion::CashIn => &self.cashin,
                TipoTransaccion::CashOut => &self.cashout
            };
            
            channel.send(transaccion).expect("channel cerrado");
        }
    }
}