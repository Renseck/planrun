//! Reading a `[[recognizer]]` table: every key used exactly as the kind defines it, unknown keys
//! refused, every regex compiled with its required captures (when the spec loads).

use planrun_model::{ExitExpect, RecognizerId};
use regex::Regex;

use crate::spec::compile_regex;

/// A recognizer's parameters, with the keys read so far.
pub(super) struct Params<'a> {
    table: &'a toml::Table,
    read: Vec<&'static str>,
}

impl<'a> Params<'a> {
    pub(super) fn new(table: &'a toml::Table) -> Self {
        Self {
            table,
            read: Vec::new(),
        }
    }

    fn get(&mut self, key: &'static str) -> Option<&'a toml::Value> {
        self.read.push(key);
        self.table.get(key)
    }

    /// The required `id`.
    pub(super) fn id(&mut self) -> Result<RecognizerId, String> {
        Ok(RecognizerId(self.string("id")?.into()))
    }

    pub(super) fn string(&mut self, key: &'static str) -> Result<&'a str, String> {
        self.opt_string(key)?
            .ok_or_else(|| format!("`{key}` is required"))
    }

    pub(super) fn opt_string(&mut self, key: &'static str) -> Result<Option<&'a str>, String> {
        match self.get(key) {
            None => Ok(None),
            Some(v) => v
                .as_str()
                .map(Some)
                .ok_or_else(|| format!("`{key}` must be a string")),
        }
    }

    /// An optional array of strings.
    pub(super) fn opt_strings(
        &mut self,
        key: &'static str,
    ) -> Result<Option<Vec<&'a str>>, String> {
        let Some(value) = self.get(key) else {
            return Ok(None);
        };
        let wrong = || format!("`{key}` must be an array of strings");
        let list = value.as_array().ok_or_else(wrong)?;
        list.iter()
            .map(|v| v.as_str().ok_or_else(wrong))
            .collect::<Result<Vec<_>, _>>()
            .map(Some)
    }

    pub(super) fn regex(&mut self, key: &'static str, required: &[&str]) -> Result<Regex, String> {
        let pattern = self.string(key)?;
        compile_regex(key, pattern, required)
    }

    pub(super) fn opt_bool(&mut self, key: &'static str) -> Result<Option<bool>, String> {
        match self.get(key) {
            None => Ok(None),
            Some(v) => v
                .as_bool()
                .map(Some)
                .ok_or_else(|| format!("`{key}` must be true or false")),
        }
    }

    pub(super) fn table(&mut self, key: &'static str) -> Result<&'a toml::Table, String> {
        self.get(key)
            .ok_or_else(|| format!("`{key}` is required"))?
            .as_table()
            .ok_or_else(|| format!("`{key}` must be a table"))
    }

    /// An array of tables, each with exactly the keys `keys`.
    pub(super) fn rows(
        &mut self,
        key: &'static str,
        keys: &[&str],
    ) -> Result<Vec<&'a toml::Table>, String> {
        let Some(value) = self.get(key) else {
            return Ok(Vec::new());
        };
        let rows = value
            .as_array()
            .ok_or_else(|| format!("`{key}` must be an array of tables"))?;
        rows.iter()
            .map(|row| {
                let row = row
                    .as_table()
                    .ok_or_else(|| format!("`{key}` must be an array of tables"))?;
                if let Some(extra) = row.keys().find(|k| !keys.contains(&k.as_str())) {
                    return Err(format!("`{key}`: unknown key `{extra}`"));
                }
                Ok(row)
            })
            .collect()
    }

    /// Refuses any key the kind did not read.
    pub(super) fn finish(self) -> Result<(), String> {
        let unknown = self.table.keys().find(|k| !self.read.contains(&k.as_str()));
        match unknown {
            Some(key) => Err(format!("unknown parameter `{key}`")),
            None => Ok(()),
        }
    }
}

/// A string field of a row.
pub(super) fn row_string<'a>(row: &'a toml::Table, key: &str) -> Result<&'a str, String> {
    row.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("a row without a string `{key}`"))
}

/// One row of an exit table: a regex and what its match means.
pub(super) struct ExitRow {
    pub(super) regex: Regex,
    pub(super) exit: ExitValue,
}

/// What an exit row maps to.
#[derive(Debug, Clone, Copy)]
pub(super) enum ExitValue {
    /// A fixed expectation.
    Fixed(ExitExpect),
    /// The number the row's `code` capture holds (0 is `Success`).
    Captured,
}

/// The `exit` rows of an expectation kind, matched on `field` (`spec` or `text`).
pub(super) fn exit_rows(
    params: &mut Params<'_>,
    field: &'static str,
) -> Result<Vec<ExitRow>, String> {
    params
        .rows("exit", &[field, "exit"])?
        .into_iter()
        .map(|row| {
            let exit = exit_value(row_string(row, "exit")?)?;
            let required: &[&str] = match exit {
                ExitValue::Captured => &["code"],
                ExitValue::Fixed(_) => &[],
            };
            let regex = compile_regex(field, row_string(row, field)?, required)?;
            Ok(ExitRow { regex, exit })
        })
        .collect()
}

fn exit_value(value: &str) -> Result<ExitValue, String> {
    match value {
        "success" => Ok(ExitValue::Fixed(ExitExpect::Success)),
        "failure" => Ok(ExitValue::Fixed(ExitExpect::Failure)),
        "any" => Ok(ExitValue::Fixed(ExitExpect::Unknown)),
        "code" => Ok(ExitValue::Captured),
        other => other
            .strip_prefix("code:")
            .and_then(|n| n.parse().ok())
            .map(|n| ExitValue::Fixed(code(n)))
            .ok_or_else(|| format!("exit `{other}` is not success, failure, any, code or code:N")),
    }
}

/// `n` as an expectation: 0 is `Success`.
pub(super) fn code(n: i32) -> ExitExpect {
    if n == 0 {
        ExitExpect::Success
    } else {
        ExitExpect::Code(n)
    }
}

/// The first row that matches `text`, as an expectation.
pub(super) fn first_exit(rows: &[ExitRow], text: &str) -> Option<ExitExpect> {
    rows.iter().find_map(|row| {
        let caps = row.regex.captures(text)?;
        match row.exit {
            ExitValue::Fixed(exit) => Some(exit),
            ExitValue::Captured => caps
                .name("code")
                .and_then(|m| m.as_str().parse().ok())
                .map(code),
        }
    })
}
