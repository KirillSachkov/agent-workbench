# Beta skills

Skills that are not yet part of the default method. `workbench init` and `workbench update` do not
install them unless a project opts in, one by one, in its `workbench.toml`:

```toml
[skills]
beta = ["<skill-name>"]
```

Each beta skill is a directory with a `SKILL.md`, like the stable skills in `../skills/`. A beta
skill is promoted to `skills/` once it has been used in a real project; see
[docs/releasing.md](../docs/releasing.md).

There are no beta skills in this release.
