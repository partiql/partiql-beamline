//! DDL-to-Script Converter
//!
//! This module provides functionality to convert DDL column definitions
//! (as output by Beamline's `infer-shape --output-format basic-ddl`) into
//! Beamline Ion scripts that can be used for data generation.
//!
//! # Supported DDL Format
//!
//! The input format is a comma-separated list of column definitions:
//! ```text
//! "column_name" TYPE,
//! "another_column" TYPE
//! ```
//!
//! # Type Mapping
//!
//! DDL types are mapped to Uniform distribution generators:
//! - `VARCHAR` / `STRING` → `UUID`
//! - `TINYINT` / `INT8` → `UniformI8`
//! - `SMALLINT` / `INT16` → `UniformI16`
//! - `INT` / `INT32` / `INTEGER` → `UniformI32`
//! - `INT64` / `BIGINT` → `UniformI64`
//! - `DOUBLE` / `FLOAT` / `FLOAT64` → `UniformF64`
//! - `DECIMAL` / `DECIMAL(p,s)` → `UniformDecimal`
//! - `BOOL` / `BOOLEAN` → `Bool`
//! - `TIMESTAMP` / `DATETIME` → `Instant`
//! - `STRUCT<...>` → nested struct with recursive type mapping
//! - `ARRAY<T>` → `UniformArray` with element type mapped recursively

use std::fmt::Write;
use thiserror::Error;

/// Errors that can occur during DDL parsing and conversion.
#[derive(Debug, Error)]
pub enum DdlConversionError {
    #[error("Failed to parse DDL: {0}")]
    ParseError(String),

    #[error("Unsupported DDL type: {0}")]
    UnsupportedType(String),

    #[error("Invalid column definition: {0}")]
    InvalidColumn(String),

    #[error("Unexpected end of input while parsing {0}")]
    UnexpectedEnd(String),
}

pub type DdlConversionResult<T> = Result<T, DdlConversionError>;

/// A parsed column definition from DDL.
#[derive(Debug, Clone)]
struct ColumnDef {
    name: String,
    col_type: DdlType,
}

/// Represents a parsed DDL type.
#[derive(Debug, Clone)]
enum DdlType {
    /// VARCHAR / STRING → UUID generator
    Varchar,
    /// TINYINT / INT8
    TinyInt,
    /// SMALLINT / INT16
    SmallInt,
    /// INT / INT32 / INTEGER
    Int,
    /// INT64 / BIGINT
    BigInt,
    /// DOUBLE / FLOAT / FLOAT64
    Double,
    /// DECIMAL with optional precision and scale
    Decimal(Option<(u32, u32)>),
    /// BOOL / BOOLEAN
    Bool,
    /// TIMESTAMP / DATETIME
    Timestamp,
    /// STRUCT<field1: TYPE1, field2: TYPE2, ...>
    Struct(Vec<ColumnDef>),
    /// ARRAY<element_type>
    Array(Box<DdlType>),
    /// UNION<TYPE1, TYPE2, ...>
    Union(Vec<DdlType>),
}

/// Converts DDL column definitions into a Beamline Ion script string.
///
/// # Arguments
/// * `ddl` - The DDL column definitions string
/// * `dataset_name` - The name to use for the dataset in the generated script
///
/// # Returns
/// A string containing a valid Beamline Ion script
///
/// # Example
/// ```
/// use partiql_beamline::ddl_to_script::ddl_to_script;
///
/// let ddl = r#""sensor_id" VARCHAR, "temperature" DOUBLE, "active" BOOL"#;
/// let script = ddl_to_script(ddl, "sensors").unwrap();
/// // script will be a valid Ion script with Uniform generators
/// ```
pub fn ddl_to_script(ddl: &str, dataset_name: &str) -> DdlConversionResult<String> {
    let columns = parse_ddl(ddl)?;
    generate_script(&columns, dataset_name)
}

/// Parses DDL column definitions into a list of `ColumnDef`.
fn parse_ddl(ddl: &str) -> DdlConversionResult<Vec<ColumnDef>> {
    let ddl = ddl.trim();
    if ddl.is_empty() {
        return Err(DdlConversionError::ParseError(
            "Empty DDL input".to_string(),
        ));
    }

    let mut columns = Vec::new();
    let mut chars = ddl.chars().peekable();

    loop {
        skip_whitespace(&mut chars);
        if chars.peek().is_none() {
            break;
        }

        let col = parse_column_def(&mut chars)?;
        columns.push(col);

        skip_whitespace(&mut chars);
        // Consume optional comma
        if chars.peek() == Some(&',') {
            chars.next();
        }
    }

    if columns.is_empty() {
        return Err(DdlConversionError::ParseError(
            "No columns found in DDL".to_string(),
        ));
    }

    Ok(columns)
}

/// Parses a single column definition: `"name" TYPE [NOT NULL] [OPTIONAL]`
fn parse_column_def(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> DdlConversionResult<ColumnDef> {
    skip_whitespace(chars);

    let name = parse_column_name(chars)?;
    skip_whitespace(chars);

    let col_type = parse_type(chars)?;

    // Skip optional NOT NULL / OPTIONAL modifiers (we ignore them for generation)
    skip_whitespace(chars);
    skip_modifiers(chars);

    Ok(ColumnDef { name, col_type })
}

/// Parses a column name, either quoted with `"` or unquoted.
fn parse_column_name(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> DdlConversionResult<String> {
    skip_whitespace(chars);

    match chars.peek() {
        Some(&'"') => {
            chars.next(); // consume opening quote
            let mut name = String::new();
            loop {
                match chars.next() {
                    Some('"') => break,
                    Some(c) => name.push(c),
                    None => {
                        return Err(DdlConversionError::UnexpectedEnd(
                            "column name".to_string(),
                        ))
                    }
                }
            }
            Ok(name)
        }
        Some(&c) if c.is_alphabetic() || c == '_' => {
            let mut name = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    name.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            Ok(name)
        }
        Some(&c) => Err(DdlConversionError::InvalidColumn(format!(
            "Unexpected character '{c}' at start of column name"
        ))),
        None => Err(DdlConversionError::UnexpectedEnd(
            "column name".to_string(),
        )),
    }
}

/// Parses a DDL type expression.
fn parse_type(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> DdlConversionResult<DdlType> {
    skip_whitespace(chars);

    // Skip prefix modifiers like OPTIONAL before the type name
    skip_prefix_modifiers(chars);

    let type_name = parse_identifier(chars)?;
    let type_upper = type_name.to_uppercase();

    skip_whitespace(chars);

    match type_upper.as_str() {
        "VARCHAR" | "STRING" | "CHAR" | "TEXT" => Ok(DdlType::Varchar),
        "TINYINT" | "INT8" => Ok(DdlType::TinyInt),
        "SMALLINT" | "INT16" => Ok(DdlType::SmallInt),
        "INT" | "INT32" | "INTEGER" => Ok(DdlType::Int),
        "INT64" | "BIGINT" => Ok(DdlType::BigInt),
        "DOUBLE" | "FLOAT" | "FLOAT64" | "FLOAT8" | "REAL" => Ok(DdlType::Double),
        "DECIMAL" | "NUMERIC" => {
            skip_whitespace(chars);
            if chars.peek() == Some(&'(') {
                let (p, s) = parse_decimal_params(chars)?;
                Ok(DdlType::Decimal(Some((p, s))))
            } else {
                Ok(DdlType::Decimal(None))
            }
        }
        "BOOL" | "BOOLEAN" => Ok(DdlType::Bool),
        "TIMESTAMP" | "DATETIME" => Ok(DdlType::Timestamp),
        "STRUCT" => {
            skip_whitespace(chars);
            if chars.peek() == Some(&'<') {
                let fields = parse_struct_fields(chars)?;
                Ok(DdlType::Struct(fields))
            } else {
                Err(DdlConversionError::ParseError(
                    "STRUCT type requires <field definitions>".to_string(),
                ))
            }
        }
        "ARRAY" => {
            skip_whitespace(chars);
            if chars.peek() == Some(&'<') {
                chars.next(); // consume '<'
                skip_whitespace(chars);
                let elem_type = parse_type(chars)?;
                // Skip any modifiers on the element type (e.g., NOT NULL)
                skip_whitespace(chars);
                skip_modifiers(chars);
                skip_whitespace(chars);
                expect_char(chars, '>')?;
                Ok(DdlType::Array(Box::new(elem_type)))
            } else {
                Err(DdlConversionError::ParseError(
                    "ARRAY type requires <element_type>".to_string(),
                ))
            }
        }
        "UNION" => {
            skip_whitespace(chars);
            if chars.peek() == Some(&'<') {
                let types = parse_union_types(chars)?;
                Ok(DdlType::Union(types))
            } else {
                Err(DdlConversionError::ParseError(
                    "UNION type requires <type1, type2, ...>".to_string(),
                ))
            }
        }
        _ => Err(DdlConversionError::UnsupportedType(type_name)),
    }
}

/// Parses DECIMAL(precision, scale) parameters.
fn parse_decimal_params(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> DdlConversionResult<(u32, u32)> {
    expect_char(chars, '(')?;
    skip_whitespace(chars);

    let precision = parse_number(chars)?;
    skip_whitespace(chars);
    expect_char(chars, ',')?;
    skip_whitespace(chars);
    let scale = parse_number(chars)?;
    skip_whitespace(chars);
    expect_char(chars, ')')?;

    Ok((precision, scale))
}

/// Parses STRUCT<"field1": TYPE1, "field2": TYPE2> fields.
fn parse_struct_fields(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> DdlConversionResult<Vec<ColumnDef>> {
    expect_char(chars, '<')?;
    let mut fields = Vec::new();

    loop {
        skip_whitespace(chars);
        if chars.peek() == Some(&'>') {
            chars.next();
            break;
        }

        let name = parse_column_name(chars)?;
        skip_whitespace(chars);
        expect_char(chars, ':')?;
        skip_whitespace(chars);
        let col_type = parse_type(chars)?;

        // Skip modifiers
        skip_whitespace(chars);
        skip_modifiers(chars);

        fields.push(ColumnDef { name, col_type });

        skip_whitespace(chars);
        if chars.peek() == Some(&',') {
            chars.next();
        }
    }

    Ok(fields)
}

/// Parses UNION<TYPE1, TYPE2, ...> types.
fn parse_union_types(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> DdlConversionResult<Vec<DdlType>> {
    expect_char(chars, '<')?;
    let mut types = Vec::new();

    loop {
        skip_whitespace(chars);
        if chars.peek() == Some(&'>') {
            chars.next();
            break;
        }

        let t = parse_type(chars)?;
        // Skip modifiers on union member types
        skip_whitespace(chars);
        skip_modifiers(chars);
        types.push(t);

        skip_whitespace(chars);
        if chars.peek() == Some(&',') {
            chars.next();
        }
    }

    Ok(types)
}

/// Parses an identifier (type name or unquoted name).
fn parse_identifier(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> DdlConversionResult<String> {
    let mut name = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' {
            name.push(c);
            chars.next();
        } else {
            break;
        }
    }
    if name.is_empty() {
        Err(DdlConversionError::ParseError(
            "Expected type name".to_string(),
        ))
    } else {
        Ok(name)
    }
}

/// Parses a non-negative integer.
fn parse_number(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> DdlConversionResult<u32> {
    let mut num_str = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            num_str.push(c);
            chars.next();
        } else {
            break;
        }
    }
    num_str
        .parse()
        .map_err(|_| DdlConversionError::ParseError(format!("Expected number, got '{num_str}'")))
}

/// Expects and consumes a specific character.
fn expect_char(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    expected: char,
) -> DdlConversionResult<()> {
    match chars.next() {
        Some(c) if c == expected => Ok(()),
        Some(c) => Err(DdlConversionError::ParseError(format!(
            "Expected '{expected}', got '{c}'"
        ))),
        None => Err(DdlConversionError::UnexpectedEnd(format!(
            "'{expected}'"
        ))),
    }
}

/// Skips whitespace characters.
fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

/// Skips prefix modifiers like `OPTIONAL` that appear before a type name.
fn skip_prefix_modifiers(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    loop {
        skip_whitespace(chars);
        let remaining: String = chars.clone().collect();
        let remaining_upper = remaining.to_uppercase();

        if remaining_upper.starts_with("OPTIONAL") {
            let after = remaining.get(8..9).and_then(|s| s.chars().next());
            if after.is_none() || !after.unwrap().is_alphanumeric() {
                for _ in 0..8 {
                    chars.next();
                }
                skip_whitespace(chars);
                continue;
            }
        }

        break;
    }
}

/// Skips optional modifiers like `NOT NULL`, `OPTIONAL`.
fn skip_modifiers(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    loop {
        skip_whitespace(chars);

        // Peek ahead to see if we have a modifier keyword
        let remaining: String = chars.clone().collect();
        let remaining_upper = remaining.to_uppercase();

        if remaining_upper.starts_with("NOT NULL") {
            // Check it's followed by a word boundary
            let after = remaining.get(8..8 + 1).map(|s| s.chars().next()).flatten();
            if after.is_none() || !after.unwrap().is_alphanumeric() {
                for _ in 0..8 {
                    chars.next();
                }
                continue;
            }
        }

        if remaining_upper.starts_with("OPTIONAL") {
            let after = remaining.get(8..8 + 1).map(|s| s.chars().next()).flatten();
            if after.is_none() || !after.unwrap().is_alphanumeric() {
                for _ in 0..8 {
                    chars.next();
                }
                continue;
            }
        }

        if remaining_upper.starts_with("NOT") {
            let after = remaining.get(3..3 + 1).map(|s| s.chars().next()).flatten();
            if after.is_none() || !after.unwrap().is_alphanumeric() {
                // Just "NOT" alone - skip it
                for _ in 0..3 {
                    chars.next();
                }
                skip_whitespace(chars);
                // Check for NULL after NOT
                let remaining2: String = chars.clone().collect();
                if remaining2.to_uppercase().starts_with("NULL") {
                    let after2 = remaining2.get(4..4 + 1).map(|s| s.chars().next()).flatten();
                    if after2.is_none() || !after2.unwrap().is_alphanumeric() {
                        for _ in 0..4 {
                            chars.next();
                        }
                    }
                }
                continue;
            }
        }

        break;
    }
}

/// Generates a Beamline Ion script from parsed column definitions.
fn generate_script(columns: &[ColumnDef], dataset_name: &str) -> DdlConversionResult<String> {
    let mut script = String::new();

    writeln!(script, "rand_processes::{{").unwrap();
    writeln!(script, "    {dataset_name}: rand_process::{{").unwrap();
    writeln!(
        script,
        "        $r: Uniform::{{ choices: [5, 10] }},"
    )
    .unwrap();
    writeln!(
        script,
        "        $arrival: HomogeneousPoisson::{{ interarrival: milliseconds::$r }},"
    )
    .unwrap();
    writeln!(script, "        $data: {{").unwrap();

    for (i, col) in columns.iter().enumerate() {
        let trailing_comma = if i < columns.len() - 1 { "," } else { "" };
        let gen = type_to_generator(&col.col_type, 12)?;
        writeln!(
            script,
            "            {}: {gen}{trailing_comma}",
            col.name
        )
        .unwrap();
    }

    writeln!(script, "        }}").unwrap();
    writeln!(script, "    }}").unwrap();
    writeln!(script, "}}").unwrap();

    Ok(script)
}

/// Converts a DDL type to its corresponding Beamline generator expression.
fn type_to_generator(ddl_type: &DdlType, indent: usize) -> DdlConversionResult<String> {
    match ddl_type {
        DdlType::Varchar => Ok("UUID".to_string()),
        DdlType::TinyInt => Ok("UniformI8".to_string()),
        DdlType::SmallInt => Ok("UniformI16".to_string()),
        DdlType::Int => Ok("UniformI32".to_string()),
        DdlType::BigInt => Ok("UniformI64".to_string()),
        DdlType::Double => Ok("UniformF64".to_string()),
        DdlType::Decimal(params) => {
            if let Some((p, s)) = params {
                // Generate a decimal with appropriate range based on precision and scale
                let max_int_digits = if *p > *s { p - s } else { 0 };
                let max_val = 10f64.powi(max_int_digits as i32);
                let min_val = -max_val;
                Ok(format!(
                    "UniformDecimal::{{ low: {min_val}, high: {max_val} }}"
                ))
            } else {
                Ok("UniformDecimal".to_string())
            }
        }
        DdlType::Bool => Ok("Bool".to_string()),
        DdlType::Timestamp => Ok("Instant".to_string()),
        DdlType::Struct(fields) => {
            let mut s = String::new();
            write!(s, "{{").unwrap();
            for (i, field) in fields.iter().enumerate() {
                let trailing_comma = if i < fields.len() - 1 { "," } else { "" };
                let gen = type_to_generator(&field.col_type, indent + 4)?;
                write!(
                    s,
                    "\n{:width$}{}: {gen}{trailing_comma}",
                    "",
                    field.name,
                    width = indent + 4
                )
                .unwrap();
            }
            write!(s, "\n{:width$}}}", "", width = indent).unwrap();
            Ok(s)
        }
        DdlType::Array(elem_type) => {
            let elem_gen = type_to_generator(elem_type, indent)?;
            Ok(format!(
                "UniformArray::{{ min_size: 1, max_size: 5, element_type: {elem_gen} }}"
            ))
        }
        DdlType::Union(types) => {
            let mut type_strs = Vec::new();
            for t in types {
                type_strs.push(type_to_generator(t, indent)?);
            }
            Ok(format!(
                "UniformAnyOf::{{ types: [{}] }}",
                type_strs.join(", ")
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_ddl() {
        let ddl = r#""sensor_id" VARCHAR, "temperature" DOUBLE, "active" BOOL"#;
        let script = ddl_to_script(ddl, "sensors").unwrap();
        assert!(script.contains("rand_processes::"));
        assert!(script.contains("sensors: rand_process::"));
        assert!(script.contains("sensor_id: UUID"));
        assert!(script.contains("temperature: UniformF64"));
        assert!(script.contains("active: Bool"));
    }

    #[test]
    fn test_numeric_types() {
        let ddl = r#""a" TINYINT, "b" SMALLINT, "c" INT, "d" BIGINT, "e" DOUBLE"#;
        let script = ddl_to_script(ddl, "test_data").unwrap();
        assert!(script.contains("a: UniformI8"));
        assert!(script.contains("b: UniformI16"));
        assert!(script.contains("c: UniformI32"));
        assert!(script.contains("d: UniformI64"));
        assert!(script.contains("e: UniformF64"));
    }

    #[test]
    fn test_decimal_with_params() {
        let ddl = r#""price" DECIMAL(5, 2)"#;
        let script = ddl_to_script(ddl, "products").unwrap();
        assert!(script.contains("price: UniformDecimal::{ low:"));
    }

    #[test]
    fn test_timestamp() {
        let ddl = r#""created_at" TIMESTAMP"#;
        let script = ddl_to_script(ddl, "events").unwrap();
        assert!(script.contains("created_at: Instant"));
    }

    #[test]
    fn test_struct_type() {
        let ddl = r#""data" STRUCT<"x": DOUBLE, "y": DOUBLE>"#;
        let script = ddl_to_script(ddl, "points").unwrap();
        assert!(script.contains("data: {"));
        assert!(script.contains("x: UniformF64"));
        assert!(script.contains("y: UniformF64"));
    }

    #[test]
    fn test_array_type() {
        let ddl = r#""tags" ARRAY<VARCHAR>"#;
        let script = ddl_to_script(ddl, "items").unwrap();
        assert!(script.contains("tags: UniformArray::{ min_size: 1, max_size: 5, element_type: UUID }"));
    }

    #[test]
    fn test_union_type() {
        let ddl = r#""value" UNION<INT8, DOUBLE>"#;
        let script = ddl_to_script(ddl, "mixed").unwrap();
        assert!(script.contains("value: UniformAnyOf::{ types: [UniformI8, UniformF64] }"));
    }

    #[test]
    fn test_not_null_modifier() {
        let ddl = r#""id" VARCHAR NOT NULL, "name" VARCHAR"#;
        let script = ddl_to_script(ddl, "users").unwrap();
        assert!(script.contains("id: UUID"));
        assert!(script.contains("name: UUID"));
    }

    #[test]
    fn test_optional_modifier() {
        let ddl = r#""price" OPTIONAL DECIMAL(5, 4)"#;
        let script = ddl_to_script(ddl, "products").unwrap();
        assert!(script.contains("price: UniformDecimal"));
    }

    #[test]
    fn test_unquoted_column_names() {
        let ddl = r#"sensor_id VARCHAR, temperature DOUBLE"#;
        let script = ddl_to_script(ddl, "sensors").unwrap();
        assert!(script.contains("sensor_id: UUID"));
        assert!(script.contains("temperature: UniformF64"));
    }

    #[test]
    fn test_beamline_ddl_output_roundtrip() {
        // This is the format output by `infer-shape --output-format basic-ddl`
        let ddl = r#""tick" INT8,
"i8" TINYINT,
"f" DOUBLE,
"w" OPTIONAL DECIMAL(5, 4),
"d" DECIMAL(2, 0) NOT NULL"#;
        let script = ddl_to_script(ddl, "sensors").unwrap();
        assert!(script.contains("tick: UniformI8"));
        assert!(script.contains("i8: UniformI8"));
        assert!(script.contains("f: UniformF64"));
        assert!(script.contains("w: UniformDecimal"));
        assert!(script.contains("d: UniformDecimal"));
    }

    #[test]
    fn test_complex_ddl() {
        let ddl = r#""Request" VARCHAR,
"StartTime" TIMESTAMP,
"Program" VARCHAR,
"Operation" VARCHAR,
"Weight" DECIMAL(5, 4),
"Distance" DECIMAL(2, 0),
"Account" VARCHAR,
"client" VARCHAR,
"success" BOOL"#;
        let script = ddl_to_script(ddl, "service").unwrap();
        assert!(script.contains("Request: UUID"));
        assert!(script.contains("StartTime: Instant"));
        assert!(script.contains("Weight: UniformDecimal"));
        assert!(script.contains("success: Bool"));
    }

    #[test]
    fn test_empty_ddl_error() {
        let result = ddl_to_script("", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_array_with_not_null_element() {
        let ddl = r#""values" ARRAY<DECIMAL(5, 4) NOT NULL>"#;
        let script = ddl_to_script(ddl, "test").unwrap();
        assert!(script.contains("values: UniformArray"));
    }

    #[test]
    fn test_union_with_not_null_member() {
        let ddl = r#""anyof" UNION<INT8,DECIMAL(5, 4) NOT NULL>"#;
        let script = ddl_to_script(ddl, "test").unwrap();
        assert!(script.contains("anyof: UniformAnyOf::{ types: [UniformI8, UniformDecimal"));
    }

    #[test]
    fn test_generated_script_is_valid_ion_structure() {
        let ddl = r#""id" VARCHAR, "value" DOUBLE"#;
        let script = ddl_to_script(ddl, "test_data").unwrap();

        // Verify the script has the expected structure
        assert!(script.starts_with("rand_processes::{"));
        assert!(script.contains("$arrival: HomogeneousPoisson::"));
        assert!(script.contains("$data: {"));
        assert!(script.ends_with("}\n"));
    }
}
