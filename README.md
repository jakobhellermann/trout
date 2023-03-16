TSP solver for routing celeste lobbies for TASing (ported from https://github.com/TheRoboManTAS/Celeste-TAS-lobby-router)

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

- [ ] show top n solutions
- [ ] suggest potential files to look at