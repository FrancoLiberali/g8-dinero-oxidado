
use std::{
    sync::mpsc::Sender,
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
   pub fn iniciar(file: &str, tx_cashin: Sender<Transaccion>, tx_cashout: Sender<Transaccion>) -> Result<JoinHandle<()>, csv::Error> {
        let reader = csv::Reader::from_path(file)?;
        let handle = thread::spawn(move || {
            let mut procesador = Self {
                file: reader,
                cashin: tx_cashin,
                cashout: tx_cashout
            };

            procesador.procesar();
        });

        Ok(handle)
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

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use super::*;
    use csv::Writer;
    use uuid::Uuid;

    #[test]
    fn procesador_envia_por_canal_cashin_cuando_lee_un_cashin() {
        let id_transaccion = 1;
        let ruta_archivo_tests = "archivo_tests.csv";
        let mut archivo = Writer::from_path(ruta_archivo_tests).unwrap();
        let transaccion = Transaccion {
            id: id_transaccion,
            id_cliente: Uuid::new_v4(),
            timestamp: 112315846_128,
            tipo: TipoTransaccion::CashIn,
            monto: 123.33
        };
        archivo.serialize(transaccion).unwrap();

        let (tx_cashin, rx_cashin) = channel();
        let (tx_cashout, _rx_cashout) = channel();

        Procesador::iniciar(ruta_archivo_tests, tx_cashin, tx_cashout).unwrap();
        assert_eq!(rx_cashin.recv().unwrap().id, id_transaccion)
    }

    #[test]
    fn procesador_envia_por_canal_cashout_cuando_lee_un_cashin() {
        let id_transaccion = 1;
        let ruta_archivo_tests = "archivo_tests.csv";
        let mut archivo = Writer::from_path(ruta_archivo_tests).unwrap();
        archivo.serialize(Transaccion {
            id: id_transaccion,
            id_cliente: Uuid::new_v4(),
            timestamp: 112315846_128,
            tipo: TipoTransaccion::CashOut,
            monto: 123.33
        }).unwrap();

        let (tx_cashin, _rx_cashin) = channel();
        let (tx_cashout, rx_cashout) = channel();

        Procesador::iniciar(ruta_archivo_tests, tx_cashin, tx_cashout).unwrap();
        assert_eq!(rx_cashout.recv().unwrap().id, id_transaccion)
    }
}
