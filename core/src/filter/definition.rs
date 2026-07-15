use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    Int,
    Float,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendSupport {
    CpuOnly,
    CpuAndGpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ParamDefinition {
    pub name: &'static str,

    #[serde(rename = "type")]
    pub kind: ParamType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f32>,

    pub default: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct FilterDefinition {
    pub name: &'static str,

    pub params: &'static [ParamDefinition],
}
