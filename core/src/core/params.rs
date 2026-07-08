use serde::Serialize;


#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ParamDefinition {

    #[serde(rename = "float")]
    Float {
        name: &'static str,
        min: Option<f32>,
        max: Option<f32>,
        default: f32,
    },


    #[serde(rename = "int")]
    Int {
        name: &'static str,
        min: Option<i32>,
        max: Option<i32>,
        default: i32,
    },


    #[serde(rename = "bool")]
    Bool {
        name: &'static str,
        default: bool,
    },
}



pub enum ParamValue {

    Float(f32),

    Int(i32),

    Bool(bool),
}