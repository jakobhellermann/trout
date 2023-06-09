<img src="./images/logo_128.png" align="left" width="128px"/>

TSP solver for routing celeste lobbies while TASing

(ported from https://github.com/TheRoboManTAS/Celeste-TAS-lobby-router)

<br clear="left"/>

# usage

```sh
trout sj_tables/beginner_table.txt [sj_tables/intermediate_table.txt sj_tables/advanced_table.txt]
```

```
Solving sj_tables/beginner_table.txt...
[0, 1, 11, 12, 10, 9, 8, 2, 3, 4, 5, 6, 7, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22] - 4660
Routing took 0.6383271s
2906832 solutions
Pathfind function called 66641736 times.
```

Or use the website at [https://jakobhellermann.github.io/trout](https://jakobhellermann.github.io/trout).

# development

## building from source

Requires a rust toolchain (https://rustup.rs/)

```sh
cargo build --release
# executable in target/release/trout
```

## profiling

- `cargo flamegraph -- sj_tables/beginner_table.txt`
- `cargo run --release --features heap_profiling -- sj_tables/beginner_table.txt` with https://nnethercote.github.io/dh_view/dh_view.html for memory usage (not a bottleneck)

# todo

- [ ] suggest potential files to look at
- [x] show more details in website (how many solution were found, how long it took)
- [ ] fix configurable restart penalty