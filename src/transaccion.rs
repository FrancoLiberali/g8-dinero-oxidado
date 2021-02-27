use serde::{Serialize, Serializer};

pub enum TipoTransaccion {
    CashIn,
    CashOut
}

impl Serialize for TipoTransaccion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            TipoTransaccion::CashIn => "cash_in",
            TipoTransaccion::CashOut => "cash_out"
        })
    }
}

#[derive(Serialize)]
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