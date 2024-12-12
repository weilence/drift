
use std::fmt;

#[derive(Debug)]
pub enum DataType {
    Integer,
    Text,
    Boolean,
    Decimal,
    Timestamp,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::Integer => write!(f, "INTEGER"),
            DataType::Text => write!(f, "TEXT"),
            DataType::Boolean => write!(f, "BOOLEAN"),
            DataType::Decimal => write!(f, "DECIMAL"),
            DataType::Timestamp => write!(f, "TIMESTAMP"),
        }
    }
}