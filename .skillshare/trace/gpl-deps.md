# GPL dependency trace

Per ADR-0003, Limn accepts a temporary GPL-3.0-or-later
contamination via gpui's transitive dependency chain. `sum_tree`
itself is Apache-2.0; the GPL surface starts at `ztracing` (and
its sibling `ztracing_macro`) and continues into `zlog`. This file
tracks the upstream resolution status of each entry on the
allow-list. Maintained manually.

## Allow-list

| Crate | License | Upstream issue | Status | Last reviewed |
|---|---|---|---|---|
| `ztracing` | GPL-3.0-or-later | [zed-industries/zed#55470](https://github.com/zed-industries/zed/issues/55470) | watching | _(fill in)_ |
| `ztracing_macro` | GPL-3.0-or-later | same | watching | _(fill in)_ |
| `zlog` | GPL-3.0-or-later | same | watching | _(fill in)_ |

## Notes

- The allow-list must not grow without an ADR.
- `sum_tree` is Apache-2.0; it is the gpui-side host that
  transitively pulls in the GPL chain. Once the upstream chain is
  decoupled, this trace can be archived and ADR-0003 superseded.
