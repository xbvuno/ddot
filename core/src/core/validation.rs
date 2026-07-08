use super::params::{
    ParamDefinition,
    ParamValue,
};


pub fn validate_param(
    definition: &ParamDefinition,
    value: &ParamValue,
) -> Result<(), String> {


    match (definition, value) {


        (
            ParamDefinition::Float {
                min,
                max,
                ..
            },

            ParamValue::Float(value)
        ) => {

            if let Some(min) = min {

                if value < min {
                    return Err(
                        format!(
                            "Value {} is below minimum {}",
                            value,
                            min
                        )
                    );
                }

            }


            if let Some(max) = max {

                if value > max {
                    return Err(
                        format!(
                            "Value {} is above maximum {}",
                            value,
                            max
                        )
                    );
                }

            }

        }



        (
            ParamDefinition::Int {
                min,
                max,
                ..
            },

            ParamValue::Int(value)
        ) => {


            if let Some(min) = min {

                if value < min {
                    return Err(
                        format!(
                            "Value {} is below minimum {}",
                            value,
                            min
                        )
                    );
                }

            }


            if let Some(max) = max {

                if value > max {
                    return Err(
                        format!(
                            "Value {} is above maximum {}",
                            value,
                            max
                        )
                    );
                }

            }

        }



        (
            ParamDefinition::Bool { .. },
            ParamValue::Bool(_)
        ) => {}



        _ => {

            return Err(
                "Parameter type mismatch".into()
            );

        }

    }


    Ok(())
}