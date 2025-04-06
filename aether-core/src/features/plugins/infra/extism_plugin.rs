use crate::features::plugins::{plugin_error::PluginErrorKind, PluginApi, PluginError};

impl PluginApi for extism::Plugin {
    fn function_exists(&self, func_name: &str) -> bool {
        self.function_exists(func_name)
    }

    fn call<'a, 'b, T: extism::ToBytes<'a>, U: extism::FromBytes<'b>>(
        &'b mut self,
        func_name: &str,
        args: T,
    ) -> Result<U, PluginError> {
        self.call::<T, U>(func_name, args)
            .map_err(move |_| PluginErrorKind::CallError(func_name.to_string()).as_error())
    }
}
