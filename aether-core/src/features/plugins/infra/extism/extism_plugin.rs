// use crate::features::plugins::{PluginApi, PluginError};

// impl PluginApi for extism::Plugin {
//     fn function_exists(&self, func_name: &str) -> bool {
//         self.function_exists(func_name)
//     }

//     fn call<'a, 'b, T: extism::ToBytes<'a>, U: extism::FromBytes<'b>>(
//         &'b mut self,
//         name: &str,
//         args: T,
//     ) -> Result<U, PluginError> {
//         self.call::<T, U>(name, args)
//             .map_err(|e| PluginError::CallError {
//                 function_name: name.to_owned(),
//                 plugin_id: self.id,
//                 error: e.to_string(),
//             })
//     }
// }
