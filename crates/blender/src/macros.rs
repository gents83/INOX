#[macro_export]
macro_rules! implement_output_pin {
    ($Type:ident) => {
        #[pyclass(module = "sabi_blender")]
        #[derive(Serialize, Deserialize)]
        #[serde(crate = "sabi_serialize")]
        pub struct $Type {
            type_name: String,
        }
        impl Default for $Type {
            fn default() -> Self {
                let type_name = std::any::type_name::<Self>()
                    .split(':')
                    .collect::<Vec<&str>>()
                    .last()
                    .unwrap()
                    .to_string();
                Self { type_name }
            }
        }
    };
}

#[macro_export]
macro_rules! implement_node {
    ($Type:ident, $Description:expr) => {
        impl Node for $Type {
            fn base_type() -> &'static str {
                "LogicNodeBase"
            }
            fn description() -> &'static str {
                $Description
            }
        }
    };
}
