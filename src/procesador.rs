
use std::{
    sync::mpsc,
    sync::mpsc::{Sender, Receiver},
    fs::File,
    thread, thread::JoinHandle,
};
use csv::Reader;
use crate::transaccion::{Transaccion, TipoTransaccion};

pub struct Procesador {
    file: Reader<File>,
    cashin: Sender<Transaccion>,
    cashout: Sender<Transaccion>,
}

impl Procesador {
   pub fn iniciar(file: &str) -> Result<(Receiver<Transaccion>, Receiver<Transaccion>, JoinHandle<()>), csv::Error> {
        let (tx_cashin, rx_cashin) = mpsc::channel();
        let (tx_cashout, rx_cashout) = mpsc::channel();

        let reader = csv::Reader::from_path(file)?;
        let handle = thread::spawn(move || {
            let mut procesador = Self {
                file: reader,
                cashin: tx_cashin,
                cashout: tx_cashout
            };

            procesador.procesar();
        });

        Ok((rx_cashin, rx_cashout, handle))
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
