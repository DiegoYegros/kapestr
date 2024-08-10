
global_conf = configuration_data()
global_conf.set_quoted('BASE_ID', base_id)
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('VERSION', version + version_suffix)
configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)
# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.project_build_root() / 'src' / 'config.rs',
  meson.project_source_root() / 'src' / 'config.rs',
  check: true
)

cargo_options = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_options += [ '--target-dir', meson.project_build_root() / 'src' ]

if get_option('profile') == 'default'
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
else
  rust_target = 'debug'
  message('Building in debug mode')
endif

cargo_env = [ 'CARGO_HOME=' + meson.project_build_root() / 'cargo-home' ]

kapestr_bin = meson.project_name()
if host_machine.system() == 'windows'
  kapestr_bin = '@0@.exe'.format(kapestr_bin)
endif

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: rust_target,
  console: true,
  env: cargo_env,
  command: [
    cargo, 'build',
    cargo_options
  ]
)

kapestr_copy = custom_target(
  'kapestr-copy-binary',
  depends: cargo_build,
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  output: kapestr_bin,
  command: [
    'cp',
    'src' / rust_target / kapestr_bin,
    '@OUTPUT@',
  ]
)

