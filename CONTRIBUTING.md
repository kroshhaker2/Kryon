# Contributing to Kryon

Kryon is based on Paper and follows Paper's patch-based development model.
Contributions require Git and JDK 25 or newer.

## Working with patches

Apply the current patch set before editing Minecraft sources:

```bash
./gradlew applyPatches
```

Direct changes outside `paper-server/src/minecraft` are tracked normally.
Changes inside `paper-server/src/minecraft` must be rebuilt into Paper patches:

```bash
./gradlew rebuildPatches
```

Keep Kryon changes small and isolated from upstream Paper code. Larger
optimizations should be separate feature patches so they can be reviewed,
disabled, and rebased independently.

## Verification

Before proposing a change, run:

```bash
./gradlew build
./gradlew createPaperclipJar
```

Performance changes must include a reproducible benchmark or profile comparing
the same workload before and after the change. Native integrations must also
test failure behavior when the native library is missing or incompatible.

Contributors are responsible for understanding, reviewing, and personally
verifying everything they submit. Project-facing communication must be written
and approved by the contributor in their own words.
