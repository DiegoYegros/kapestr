
subdir('icons')

blueprint_files = [
  'ui/main_window.blp',
  'ui/post.blp'
]

blueprint_targets = []
foreach blp : blueprint_files
  blueprint_targets += blp.replace('.blp', '.ui').split('/')[-1]
endforeach

blueprints = custom_target('blueprints',
  input: files(blueprint_files),
  output: blueprint_targets,
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTDIR@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

resources = gnome.compile_resources(
  'kapestr',
  'kapestr.gresource.xml',
  dependencies: blueprints,
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  install: true,
  install_dir: pkgdatadir,
)

install_data('py.com.kapestr.gschema.xml', install_dir: datadir / 'glib-2.0' / 'schemas')
glib_compile_schemas = find_program('glib-compile-schemas', required: false)
if glib_compile_schemas.found()
  test('Validate schema file', glib_compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()]
  )
endif


desktop_file = configure_file(
  input: 'kapestr.desktop.in',
  output: '@0@.desktop'.format(application_id),
  configuration: { 'app_id': application_id },
  install_dir: join_paths(get_option('datadir'), 'applications')
)
desktop_utils = find_program('desktop-file-validate', required: true)
test('Validate desktop file', desktop_utils, args: [desktop_file])

