# Workbench

![a detailed color illustration of workbench, with a ruby red toolbox sitting on it along with some wrenches, hammers, goggles, pencils, blueprints, etc, in the style of a scientific drawing](assets/workbench-hero.png)

A personal CLI for managing my own projects. Most stuff is hard coded for myself. Feel free to fork and hard code for yourself instead or make it more generic or whatever. Run `wb -h` to learn more.

## Templates

- `wb init --kind website` now pulls the latest React Router starter from `remix-run/react-router-templates` (defaults to `main`) before applying Workbench overlays (Vite config, wrangler, tsconfig, prettier/oxlint, etc.).
- Project config now records `project.*`, `web.*`, `data.*`, `ui.*`, `tooling.*`, and `agents.*` fields in `.workbench/config.jsonc` so downstream updates can reconcile with the generated files.

## Development

To get started:

```sh
mise install
mise run build
./bin/wb <command>
```

If you want to do some testing in a temporary directory, you can use the `mise run dev` command instead of building to `./bin`.
