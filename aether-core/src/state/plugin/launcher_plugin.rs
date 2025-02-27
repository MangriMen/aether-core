extism::typed_plugin!(
  pub LauncherPlugin {
  on_load(()) -> ();
  on_unload(()) ->  ();
  import(String) -> String;
  // get_import_handler(()) -> String;
});

impl std::fmt::Debug for LauncherPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LauncherPlugin {{ ... }}")
    }
}
