use serde::{Deserialize, Serialize};

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

type HashAutorizacion = u32;

#[derive(Debug)]
pub struct TransaccionAutorizada {
    transaccion: Transaccion,
    autorizacion: HashAutorizacion
}

impl TransaccionAutorizada {
    pub fn new(transaccion: Transaccion, autorizacion: HashAutorizacion) -> Self {
        Self {
            transaccion,
            autorizacion,
        }
    }
}