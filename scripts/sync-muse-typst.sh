#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

src="${MUSE_TYPST:-$HOME/Code/Facile/muse-typst}"
out=crates/encre/typst/muse-typst

if [ -n "$(git -C "$src" status --porcelain)" ]; then
  echo "sync-muse-typst: $src has uncommitted changes; commit them first" >&2
  exit 1
fi

rm -rf "$out"
mkdir -p "$out/assets"
cp "$src/lib.typ" "$src/tokens.typ" "$src/components.typ" "$src/template.typ" "$src/LICENSE" "$out/"
cp "$src"/assets/*.svg "$out/assets/"
commit="$(git -C "$src" rev-parse HEAD)"
date="$(git -C "$src" log -1 --format=%cs)"
compiler="$(sed -n 's/^typst = "\(.*\)"$/\1/p' "$src/mise.toml")"
printf 'repository https://github.com/FacileStudio/muse-typst\ncommit %s\ndate %s\ntypst %s\n' "$commit" "$date" "$compiler" > "$out/SOURCE"
echo "synced muse-typst $commit (typst $compiler)"
