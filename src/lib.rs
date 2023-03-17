pub mod solver;
use anyhow::{anyhow, ensure, Result};

fn strip_around<'a>(prefix: &str, suffix: &str, input: &'a str) -> Option<&'a str> {
    input.strip_prefix(prefix)?.strip_suffix(suffix)
}

type Length = u32;
type Table = Vec<Vec<Length>>;

pub fn parse_table(table: &str) -> Result<Table> {
    let table = table
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let line = strip_around("[", "]", line)
                .ok_or_else(|| anyhow!("line is not an array: '{line}'"))?;
            let connections = line
                .split(',')
                .map(|val| {
                    ensure!(!val.is_empty(), "table contains empty rows");
                    val.trim()
                        .parse::<Length>()
                        .map_err(|e| anyhow!("failed to parse '{val}' as integer: {e} ('{line}')"))
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(connections)
        })
        .collect::<Result<Vec<_>>>()?;

    let length = table.get(0).ok_or_else(|| anyhow!("table is empty"))?.len();

    for row in &table[1..] {
        if row.len() != length {
            anyhow::bail!(
                "not every table row has the same length (expected {length}, got {})",
                row.len()
            );
        }
    }

    Ok(table)
}
