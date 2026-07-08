#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamError {

    OutOfRange {

        name: &'static str,

        value: String,
    },

    InvalidValue {

        name: &'static str,
    },
}
