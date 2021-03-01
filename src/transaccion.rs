use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub enum TipoTransaccion {
    #[serde(rename = "cash_in")]
    CashIn,
    #[serde(rename = "cash_out")]
    CashOut
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaccion {
    #[serde(rename = "Transaction")]
    pub id_transaccion: u32,
    #[serde(rename = "User_id")]
    pub id_cliente: u32,
    #[serde(rename = "Timestamp")]
    pub timestamp: u32,
    #[serde(rename = "Type")]
    pub tipo: TipoTransaccion,
    #[serde(rename = "Amount")]
    pub monto: f32
}

pub type HashAutorizacion = u32;

#[derive(Debug)]
pub struct TransaccionAutorizada {
    pub transaccion: Transaccion,
    pub autorizacion: HashAutorizacion
}

impl fmt::Display for TransaccionAutorizada {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TransaccionAutorizada (id = {})", self.transaccion.id_transaccion)
    }
}

impl TransaccionAutorizada {
    pub fn new(transaccion: Transaccion, autorizacion: HashAutorizacion) -> Self {
        Self {
            transaccion,
            autorizacion,
        }
    }
}