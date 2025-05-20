use crate::features::plugins::PluginError;

pub trait PluginApi {
    fn function_exists(&self, func_name: &str) -> bool;
    fn call<'a, 'b, T: extism::ToBytes<'a>, U: extism::FromBytes<'b>>(
        &'b mut self,
        func_name: &str,
        args: T,
    ) -> Result<U, PluginError>;
}
