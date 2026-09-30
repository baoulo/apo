# mise

`apo` is not in the built-in mise registry. Use the GitHub backend:

```bash
mise use -g github:baoulo/apo@0.2.0
```

Or in `mise.toml`:

```toml
[tools]
"github:baoulo/apo" = "0.2.0"
```

To keep a short name in `.tool-versions` (`apo 0.2.0`):

```toml
[tool_alias]
apo = "github:baoulo/apo"
```

If multiple assets match, narrow with `matching = "apo-"`.
