pub mod solver;
use anyhow::{anyhow, Result};

fn strip_around<'a>(prefix: &str, suffix: &str, input: &'a str) -> Option<&'a str> {
    input.strip_prefix(prefix)?.strip_suffix(suffix)
}

type Length = u32;
type Table = Vec<Vec<Length>>;

pub fn parse_table(table: &str) -> Result<Table> {
    let table = table
        .lines()
        .map(|line| {
            let line = strip_around("[", "]", line)
                .ok_or_else(|| anyhow!("table doesn't contain arrays"))?;
            let connections = line
                .split(',')
                .map(|val| {
                    val.trim()
                        .parse::<Length>()
                        .map_err(|e| anyhow!("failed to parse int: {e}"))
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(connections)
        })
        .collect::<Result<Vec<_>>>()?;

    let length = table.get(0).ok_or_else(|| anyhow!("no table data"))?.len();
    if table[1..].iter().any(|row| row.len() != length) {
        anyhow::bail!("not every table row has the same length (expected {length})");
    }

    Ok(table)
}
