# Usage

To convert a folder full of markdown into a static HTML site is as simple as
`docket`. By default the current directory is searched for markdown and the
target directory where the site is rendered is `./build`. The source and target
directory can be overridden with `-s` and `-t`:

```nohilight
Docket Documentation Generator

Usage: docket [options]

Options:
  -h --help           Show this screen.
  -w, --watch         Watch for changes and re-generate.
  -s, --source=<in>   Documentation directory, default is current directory.
  -t, --target=<out>  Write the output to <out>, default is `./build/`.
```

Further configuration is deliberately left impossible. The aim is to provide a
simple way to create documentation without any configuration or theming to
provide distractions.

## Logging

By default `docket` keeps quiet about what it does. You can override this by
setting the environment variables `DOCKET_LOG` and `DOCKET_LOG_STYLE` to [an
`env_logger` compatible value][envlog].

 [envlog]: https://docs.rs/env_logger/latest/env_logger/fn.init_from_env.html#examples