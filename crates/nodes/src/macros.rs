#[macro_export]
macro_rules! implement_pin {
    ($Type:ident) => {
        #[typetag::serde]
        impl $crate::Pin for $Type {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            fn get_type_id(&self) -> std::any::TypeId {
                std::any::TypeId::of::<$Type>()
            }
            fn get_type_name(&self) -> &'static str {
                std::any::type_name::<$Type>()
                    .split(':')
                    .collect::<Vec<&str>>()
                    .last()
                    .unwrap()
            }
        }
        impl $crate::PinType for $Type {
            fn type_id(&self) -> std::any::TypeId {
                std::any::TypeId::of::<$Type>()
            }
            fn resolve_pin(
                &self,
                from_node: &Node,
                from_pin: &str,
                to_node: &mut Node,
                to_pin: &str,
            ) {
                if let Some(i) = to_node.get_input_mut::<$Type>(to_pin) {
                    if let Some(o) = from_node.get_output::<$Type>(from_pin) {
                        *i = o.clone();
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! implement_node {
    ($Type:ident, $NodeField:ident, $Category:expr, $Description:expr) => {
        #[typetag::serde]
        impl $crate::NodeTrait for $Type {
            fn get_type() -> &'static str
            where
                Self: Sized,
            {
                stringify!($Type)
            }
            fn category() -> &'static str
            where
                Self: Sized,
            {
                $Category
            }
            fn description() -> &'static str
            where
                Self: Sized,
            {
                $Description
            }
            fn node(&self) -> &$crate::Node {
                &self.$NodeField
            }
            fn node_mut(&mut self) -> &mut crate::Node {
                &mut self.$NodeField
            }
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            fn serialize_node(&self) -> String {
                sabi_serialize::serialize(self)
            }
            fn deserialize_node(&self, s: &str) -> Option<Self>
            where
                Self: Sized,
            {
                if let Ok(n) = sabi_serialize::deserialize(s) {
                    return Some(n);
                }
                None
            }
        }
    };
}
