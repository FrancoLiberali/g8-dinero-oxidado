use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};
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
    pub id: u32,
    #[serde(rename = "User_id")]
    pub id_cliente: uuid::Uuid,
    #[serde(rename = "Timestamp")]
    pub timestamp: u128,
    #[serde(rename = "Type")]
    pub tipo: TipoTransaccion,
    #[serde(rename = "Amount")]
    pub monto: f32
}

pub type HashAutorizacion = uuid::Uuid;

#[derive(Debug)]
pub struct TransaccionAutorizada {
    pub transaccion: Transaccion,
    pub autorizacion: HashAutorizacion
}

impl fmt::Display for TransaccionAutorizada {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TransaccionAutorizada (id = {})", self.transaccion.id)
    }
}

#[derive(Debug)]
pub struct TransaccionExitosa {
    pub transaccion: TransaccionAutorizada,
    pub saldo_final: f32,
    pub timestamp: u128
}

impl Serialize for TransaccionExitosa {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 7 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("TransaccionExitosa", 7)?;
        state.serialize_field("Transaction", &self.transaccion.transaccion.id)?;
        state.serialize_field("User_id", &self.transaccion.transaccion.id_cliente)?;
        state.serialize_field("Transaction_Timestamp", &self.transaccion.transaccion.timestamp)?;
        state.serialize_field("Type", &self.transaccion.transaccion.tipo)?;
        state.serialize_field("Amount", &self.transaccion.transaccion.monto)?;
        state.serialize_field("Authorization_hash", &self.transaccion.autorizacion)?;
        state.serialize_field("Timestamp", &self.timestamp)?;
        state.serialize_field("Final_balance", &self.saldo_final)?;
        state.end()
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