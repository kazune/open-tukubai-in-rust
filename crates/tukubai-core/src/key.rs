use std::cmp::Ordering;
use std::error::Error as StdError;
use std::fmt;

use crate::{FieldPosition, split_fields};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyOptions {
    pub allow_numeric: bool,
    pub allow_descending: bool,
}

impl KeyOptions {
    pub const fn unrestricted() -> Self {
        Self {
            allow_numeric: true,
            allow_descending: true,
        }
    }
}

impl Default for KeyOptions {
    fn default() -> Self {
        Self::unrestricted()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyKind {
    Byte,
    Numeric,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyAttr {
    pub kind: KeyKind,
    pub descending: bool,
}

impl Default for KeyAttr {
    fn default() -> Self {
        Self {
            kind: KeyKind::Byte,
            descending: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyProgram {
    items: Vec<KeyItem>,
}

impl KeyProgram {
    pub fn new(items: Vec<KeyItem>) -> Self {
        Self { items }
    }

    pub fn items(&self) -> &[KeyItem] {
        &self.items
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KeyItem {
    Field(KeyField),
    Range(KeyFieldExpr, KeyFieldExpr, KeyAttr),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyField {
    pub expr: KeyFieldExpr,
    pub attr: KeyAttr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KeyFieldExpr {
    FieldNumber(u64),
    LastField,
    LastFieldMinus(u64),
}

#[derive(Debug, Eq, PartialEq)]
pub enum KeyParseError {
    Empty,
    InvalidSyntax,
    InvalidNumber,
    UnsupportedNumeric,
    UnsupportedDescending,
    UnsupportedZero,
    ZeroInRange,
    MixedRangeAttributes,
}

impl fmt::Display for KeyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty key is not allowed"),
            Self::InvalidSyntax => f.write_str("invalid key syntax"),
            Self::InvalidNumber => f.write_str("invalid key number"),
            Self::UnsupportedNumeric => {
                f.write_str("numeric comparison is not supported by this command")
            }
            Self::UnsupportedDescending => {
                f.write_str("descending sort order is not supported by this command")
            }
            Self::UnsupportedZero => f.write_str("selector 0 is not supported in key syntax"),
            Self::ZeroInRange => f.write_str("selector 0 must not appear inside a key range"),
            Self::MixedRangeAttributes => {
                f.write_str("range endpoints must use the same comparison attributes")
            }
        }
    }
}

impl StdError for KeyParseError {}

#[derive(Debug, Eq, PartialEq)]
pub enum KeyResolveError {
    MissingField,
    NfResolvedToZero,
}

impl fmt::Display for KeyResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField => f.write_str("key resolved to a non-existent field"),
            Self::NfResolvedToZero => f.write_str("key expression NF-<n> resolved to field 0"),
        }
    }
}

impl StdError for KeyResolveError {}

#[derive(Debug, Eq, PartialEq)]
pub enum KeyCompareError {
    InvalidNumericValue,
    KeyLengthMismatch,
}

impl fmt::Display for KeyCompareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNumericValue => {
                f.write_str("numeric key field is not a valid decimal number")
            }
            Self::KeyLengthMismatch => {
                f.write_str("resolved keys have different lengths and cannot be compared")
            }
        }
    }
}

impl StdError for KeyCompareError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedKeyPosition {
    pub position: FieldPosition,
    pub attr: KeyAttr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedKeyField<'a> {
    pub bytes: &'a [u8],
    pub position: FieldPosition,
    pub attr: KeyAttr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedKey<'a> {
    fields: Vec<ResolvedKeyField<'a>>,
}

impl<'a> ResolvedKey<'a> {
    pub fn new(fields: Vec<ResolvedKeyField<'a>>) -> Self {
        Self { fields }
    }

    pub fn fields(&self) -> &[ResolvedKeyField<'a>] {
        &self.fields
    }
}

pub fn parse_key_program(input: &[u8], options: KeyOptions) -> Result<KeyProgram, KeyParseError> {
    if input.is_empty() {
        return Err(KeyParseError::Empty);
    }

    let mut items = Vec::new();

    for part in input.split(|byte| *byte == b'@') {
        if part.is_empty() {
            return Err(KeyParseError::InvalidSyntax);
        }

        items.push(parse_key_item(part, options)?);
    }

    Ok(KeyProgram::new(items))
}

pub fn resolve_key_positions(
    program: &KeyProgram,
    record: &[u8],
) -> Result<Vec<ResolvedKeyPosition>, KeyResolveError> {
    let field_count = split_fields(record).count();
    let mut resolved = Vec::new();

    for item in program.items() {
        match item {
            KeyItem::Field(field) => {
                let number = resolve_expr_number(&field.expr, field_count)?;
                resolved.push(ResolvedKeyPosition {
                    position: FieldPosition::from_one_based(number),
                    attr: field.attr,
                });
            }
            KeyItem::Range(start, end, attr) => {
                let start_number = resolve_expr_number(start, field_count)?;
                let end_number = resolve_expr_number(end, field_count)?;

                if start_number <= end_number {
                    for number in start_number..=end_number {
                        resolved.push(ResolvedKeyPosition {
                            position: FieldPosition::from_one_based(number),
                            attr: *attr,
                        });
                    }
                } else {
                    for number in (end_number..=start_number).rev() {
                        resolved.push(ResolvedKeyPosition {
                            position: FieldPosition::from_one_based(number),
                            attr: *attr,
                        });
                    }
                }
            }
        }
    }

    Ok(resolved)
}

pub fn resolve_key<'a>(
    program: &KeyProgram,
    record: &'a [u8],
) -> Result<ResolvedKey<'a>, KeyResolveError> {
    let fields: Vec<&[u8]> = split_fields(record).collect();
    let mut resolved = Vec::new();

    for item in program.items() {
        match item {
            KeyItem::Field(field) => {
                let number = resolve_expr_number(&field.expr, fields.len())?;
                let index = position_to_index(number, fields.len())?;
                resolved.push(ResolvedKeyField {
                    bytes: fields[index],
                    position: FieldPosition::from_one_based(number),
                    attr: field.attr,
                });
            }
            KeyItem::Range(start, end, attr) => {
                let start_number = resolve_expr_number(start, fields.len())?;
                let end_number = resolve_expr_number(end, fields.len())?;

                if start_number <= end_number {
                    for number in start_number..=end_number {
                        let index = position_to_index(number, fields.len())?;
                        resolved.push(ResolvedKeyField {
                            bytes: fields[index],
                            position: FieldPosition::from_one_based(number),
                            attr: *attr,
                        });
                    }
                } else {
                    for number in (end_number..=start_number).rev() {
                        let index = position_to_index(number, fields.len())?;
                        resolved.push(ResolvedKeyField {
                            bytes: fields[index],
                            position: FieldPosition::from_one_based(number),
                            attr: *attr,
                        });
                    }
                }
            }
        }
    }

    Ok(ResolvedKey::new(resolved))
}

pub fn resolve_key_with_positions<'a>(
    positions: &[ResolvedKeyPosition],
    record: &'a [u8],
) -> Result<ResolvedKey<'a>, KeyResolveError> {
    let fields: Vec<&[u8]> = split_fields(record).collect();
    let mut resolved = Vec::with_capacity(positions.len());

    for position in positions {
        let number = position.position.get();
        let index = position_to_index(number, fields.len())?;
        resolved.push(ResolvedKeyField {
            bytes: fields[index],
            position: position.position,
            attr: position.attr,
        });
    }

    Ok(ResolvedKey::new(resolved))
}

pub fn normalize_key_positions_to_one(
    positions: &[ResolvedKeyPosition],
) -> Vec<ResolvedKeyPosition> {
    let Some(minimum) = positions
        .iter()
        .map(|position| position.position.get())
        .min()
    else {
        return Vec::new();
    };

    positions
        .iter()
        .map(|position| ResolvedKeyPosition {
            position: FieldPosition::from_one_based(position.position.get() - minimum + 1),
            attr: position.attr,
        })
        .collect()
}

pub fn compare_resolved_keys(
    left: &ResolvedKey<'_>,
    right: &ResolvedKey<'_>,
) -> Result<Ordering, KeyCompareError> {
    if left.fields().len() != right.fields().len() {
        return Err(KeyCompareError::KeyLengthMismatch);
    }

    for (left_field, right_field) in left.fields().iter().zip(right.fields().iter()) {
        let mut ordering = match left_field.attr.kind {
            KeyKind::Byte => left_field.bytes.cmp(right_field.bytes),
            KeyKind::Numeric => compare_numeric_bytes(left_field.bytes, right_field.bytes)?,
        };

        if left_field.attr.descending {
            ordering = ordering.reverse();
        }

        if ordering != Ordering::Equal {
            return Ok(ordering);
        }
    }

    Ok(Ordering::Equal)
}

fn resolve_expr_number(expr: &KeyFieldExpr, field_count: usize) -> Result<u64, KeyResolveError> {
    match expr {
        KeyFieldExpr::FieldNumber(number) => {
            resolve_field_index(*number, field_count)?;
            Ok(*number)
        }
        KeyFieldExpr::LastField => resolve_last_field_number(field_count),
        KeyFieldExpr::LastFieldMinus(offset) => {
            resolve_last_field_minus_number(*offset, field_count)
        }
    }
}

fn resolve_last_field_number(field_count: usize) -> Result<u64, KeyResolveError> {
    if field_count == 0 {
        return Err(KeyResolveError::MissingField);
    }

    u64::try_from(field_count).map_err(|_| KeyResolveError::MissingField)
}

fn resolve_field_index(number: u64, field_count: usize) -> Result<usize, KeyResolveError> {
    if number == 0 {
        return Err(KeyResolveError::MissingField);
    }

    let zero_based = usize::try_from(number - 1).map_err(|_| KeyResolveError::MissingField)?;
    if zero_based >= field_count {
        return Err(KeyResolveError::MissingField);
    }

    Ok(zero_based)
}

fn position_to_index(number: u64, field_count: usize) -> Result<usize, KeyResolveError> {
    resolve_field_index(number, field_count)
}

fn resolve_last_field_minus_number(
    offset: u64,
    field_count: usize,
) -> Result<u64, KeyResolveError> {
    let offset = usize::try_from(offset).map_err(|_| KeyResolveError::MissingField)?;

    if field_count == 0 {
        return Err(KeyResolveError::MissingField);
    }

    if offset == field_count {
        return Err(KeyResolveError::NfResolvedToZero);
    }

    if offset > field_count {
        return Err(KeyResolveError::MissingField);
    }

    u64::try_from(field_count - offset).map_err(|_| KeyResolveError::MissingField)
}

fn compare_numeric_bytes(left: &[u8], right: &[u8]) -> Result<Ordering, KeyCompareError> {
    let left = parse_decimal(left)?;
    let right = parse_decimal(right)?;

    if left.negative != right.negative {
        return Ok(if left.negative {
            Ordering::Less
        } else {
            Ordering::Greater
        });
    }

    let ordering = compare_unsigned_decimal_parts(&left, &right);
    if left.negative {
        Ok(ordering.reverse())
    } else {
        Ok(ordering)
    }
}

fn compare_unsigned_decimal_parts(left: &ParsedDecimal<'_>, right: &ParsedDecimal<'_>) -> Ordering {
    match compare_digit_slices(left.integer, right.integer) {
        Ordering::Equal => compare_fractional_slices(left.fraction, right.fraction),
        ordering => ordering,
    }
}

fn compare_digit_slices(left: &[u8], right: &[u8]) -> Ordering {
    let left = trim_leading_zero_digits(left);
    let right = trim_leading_zero_digits(right);

    match left.len().cmp(&right.len()) {
        Ordering::Equal => left.cmp(right),
        ordering => ordering,
    }
}

fn compare_fractional_slices(left: &[u8], right: &[u8]) -> Ordering {
    let max_len = left.len().max(right.len());

    for index in 0..max_len {
        let left_digit = left.get(index).copied().unwrap_or(b'0');
        let right_digit = right.get(index).copied().unwrap_or(b'0');

        match left_digit.cmp(&right_digit) {
            Ordering::Equal => continue,
            ordering => return ordering,
        }
    }

    Ordering::Equal
}

fn trim_leading_zero_digits(digits: &[u8]) -> &[u8] {
    let first_non_zero = digits.iter().position(|byte| *byte != b'0');
    match first_non_zero {
        Some(index) => &digits[index..],
        None => &digits[digits.len().saturating_sub(1)..],
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ParsedDecimal<'a> {
    negative: bool,
    integer: &'a [u8],
    fraction: &'a [u8],
}

fn parse_decimal(input: &[u8]) -> Result<ParsedDecimal<'_>, KeyCompareError> {
    if input.is_empty() {
        return Err(KeyCompareError::InvalidNumericValue);
    }

    let (negative, rest) = match input[0] {
        b'+' => (false, &input[1..]),
        b'-' => (true, &input[1..]),
        _ => (false, input),
    };

    if rest.is_empty() {
        return Err(KeyCompareError::InvalidNumericValue);
    }

    if let Some(dot) = rest.iter().position(|byte| *byte == b'.') {
        let integer = &rest[..dot];
        let fraction = &rest[dot + 1..];

        if integer.is_empty() && fraction.is_empty() {
            return Err(KeyCompareError::InvalidNumericValue);
        }

        if !integer.iter().all(|byte| byte.is_ascii_digit()) {
            return Err(KeyCompareError::InvalidNumericValue);
        }

        if !fraction.iter().all(|byte| byte.is_ascii_digit()) {
            return Err(KeyCompareError::InvalidNumericValue);
        }

        return Ok(ParsedDecimal {
            negative,
            integer: if integer.is_empty() { b"0" } else { integer },
            fraction,
        });
    }

    if !rest.iter().all(|byte| byte.is_ascii_digit()) {
        return Err(KeyCompareError::InvalidNumericValue);
    }

    Ok(ParsedDecimal {
        negative,
        integer: rest,
        fraction: b"",
    })
}

fn parse_key_item(input: &[u8], options: KeyOptions) -> Result<KeyItem, KeyParseError> {
    if let Some((left, right)) = split_range(input) {
        let start = parse_range_expr(left, options)?;
        let end = parse_range_expr(right, options)?;

        if start.attr != end.attr {
            return Err(KeyParseError::MixedRangeAttributes);
        }

        return Ok(KeyItem::Range(start.expr, end.expr, start.attr));
    }

    Ok(KeyItem::Field(parse_key_field(input, options)?))
}

fn split_range(input: &[u8]) -> Option<(&[u8], &[u8])> {
    let position = input.iter().position(|byte| *byte == b'/')?;
    let left = &input[..position];
    let right = &input[position + 1..];
    Some((left, right))
}

fn parse_range_expr(input: &[u8], options: KeyOptions) -> Result<KeyField, KeyParseError> {
    match parse_key_field(input, options) {
        Err(KeyParseError::UnsupportedZero) => Err(KeyParseError::ZeroInRange),
        result => result,
    }
}

fn parse_key_field(input: &[u8], options: KeyOptions) -> Result<KeyField, KeyParseError> {
    let (expr_input, attr) = split_attr(input, options)?;
    let expr = parse_key_expr(expr_input)?;
    Ok(KeyField { expr, attr })
}

fn split_attr(input: &[u8], options: KeyOptions) -> Result<(&[u8], KeyAttr), KeyParseError> {
    if input.is_empty() {
        return Err(KeyParseError::InvalidSyntax);
    }

    let (expr_input, attr) = if let Some(expr) = input.strip_suffix(b"nr") {
        (
            expr,
            KeyAttr {
                kind: KeyKind::Numeric,
                descending: true,
            },
        )
    } else if let Some(expr) = input.strip_suffix(b"n") {
        (
            expr,
            KeyAttr {
                kind: KeyKind::Numeric,
                descending: false,
            },
        )
    } else if let Some(expr) = input.strip_suffix(b"r") {
        (
            expr,
            KeyAttr {
                kind: KeyKind::Byte,
                descending: true,
            },
        )
    } else {
        (input, KeyAttr::default())
    };

    if expr_input.is_empty() {
        return Err(KeyParseError::InvalidSyntax);
    }

    if attr.kind == KeyKind::Numeric && !options.allow_numeric {
        return Err(KeyParseError::UnsupportedNumeric);
    }

    if attr.descending && !options.allow_descending {
        return Err(KeyParseError::UnsupportedDescending);
    }

    Ok((expr_input, attr))
}

fn parse_key_expr(input: &[u8]) -> Result<KeyFieldExpr, KeyParseError> {
    if input.is_empty() {
        return Err(KeyParseError::InvalidSyntax);
    }

    if input == b"0" {
        return Err(KeyParseError::UnsupportedZero);
    }

    if input == b"NF" {
        return Ok(KeyFieldExpr::LastField);
    }

    if let Some(rest) = input.strip_prefix(b"NF-") {
        if rest.is_empty() {
            return Err(KeyParseError::InvalidSyntax);
        }

        return Ok(KeyFieldExpr::LastFieldMinus(parse_number(rest)?));
    }

    if input.iter().all(|byte| byte.is_ascii_digit()) {
        return Ok(KeyFieldExpr::FieldNumber(parse_number(input)?));
    }

    Err(KeyParseError::InvalidSyntax)
}

fn parse_number(input: &[u8]) -> Result<u64, KeyParseError> {
    let mut value = 0_u64;

    for byte in input {
        if !byte.is_ascii_digit() {
            return Err(KeyParseError::InvalidSyntax);
        }

        value = value
            .checked_mul(10)
            .and_then(|current| current.checked_add(u64::from(byte - b'0')))
            .ok_or(KeyParseError::InvalidNumber)?;
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{
        KeyAttr, KeyField, KeyFieldExpr, KeyItem, KeyKind, KeyOptions, KeyParseError,
        KeyResolveError, ResolvedKey, ResolvedKeyField, ResolvedKeyPosition, compare_resolved_keys,
        normalize_key_positions_to_one, parse_key_program, resolve_key, resolve_key_positions,
        resolve_key_with_positions,
    };
    use crate::FieldPosition;
    use std::cmp::Ordering;

    #[test]
    fn parses_single_fields_with_attributes() {
        let program = parse_key_program(b"2@NFn@3r@NF-1nr", KeyOptions::default()).unwrap();

        assert_eq!(
            program.items(),
            &[
                KeyItem::Field(KeyField {
                    expr: KeyFieldExpr::FieldNumber(2),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                }),
                KeyItem::Field(KeyField {
                    expr: KeyFieldExpr::LastField,
                    attr: KeyAttr {
                        kind: KeyKind::Numeric,
                        descending: false,
                    },
                }),
                KeyItem::Field(KeyField {
                    expr: KeyFieldExpr::FieldNumber(3),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: true,
                    },
                }),
                KeyItem::Field(KeyField {
                    expr: KeyFieldExpr::LastFieldMinus(1),
                    attr: KeyAttr {
                        kind: KeyKind::Numeric,
                        descending: true,
                    },
                }),
            ]
        );
    }

    #[test]
    fn parses_ranges_and_composite_keys() {
        let program = parse_key_program(b"1/2@4/5", KeyOptions::default()).unwrap();

        assert_eq!(
            program.items(),
            &[
                KeyItem::Range(
                    KeyFieldExpr::FieldNumber(1),
                    KeyFieldExpr::FieldNumber(2),
                    KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                ),
                KeyItem::Range(
                    KeyFieldExpr::FieldNumber(4),
                    KeyFieldExpr::FieldNumber(5),
                    KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                ),
            ]
        );
    }

    #[test]
    fn parses_numeric_and_descending_ranges() {
        let program = parse_key_program(b"2n/NFn@4nr/2nr", KeyOptions::default()).unwrap();

        assert_eq!(
            program.items(),
            &[
                KeyItem::Range(
                    KeyFieldExpr::FieldNumber(2),
                    KeyFieldExpr::LastField,
                    KeyAttr {
                        kind: KeyKind::Numeric,
                        descending: false,
                    },
                ),
                KeyItem::Range(
                    KeyFieldExpr::FieldNumber(4),
                    KeyFieldExpr::FieldNumber(2),
                    KeyAttr {
                        kind: KeyKind::Numeric,
                        descending: true,
                    },
                ),
            ]
        );
    }

    #[test]
    fn rejects_mixed_range_attributes() {
        for input in [
            b"2/NFn".as_slice(),
            b"2/NFr".as_slice(),
            b"2n/NFr".as_slice(),
        ] {
            let error = parse_key_program(input, KeyOptions::default()).unwrap_err();
            assert_eq!(error, KeyParseError::MixedRangeAttributes);
        }
    }

    #[test]
    fn rejects_invalid_key_syntax() {
        for input in [
            b"".as_slice(),
            b"@".as_slice(),
            b"1@@2".as_slice(),
            b"/2".as_slice(),
            b"3/".as_slice(),
            b"nf".as_slice(),
            b"NF+1".as_slice(),
            b"+5".as_slice(),
            b"1@".as_slice(),
        ] {
            let error = parse_key_program(input, KeyOptions::default()).unwrap_err();
            if input.is_empty() {
                assert_eq!(error, KeyParseError::Empty);
            } else {
                assert_eq!(error, KeyParseError::InvalidSyntax);
            }
        }
    }

    #[test]
    fn rejects_zero_selectors() {
        let error = parse_key_program(b"0", KeyOptions::default()).unwrap_err();
        assert_eq!(error, KeyParseError::UnsupportedZero);

        let error = parse_key_program(b"0/3", KeyOptions::default()).unwrap_err();
        assert_eq!(error, KeyParseError::ZeroInRange);
    }

    #[test]
    fn rejects_numeric_when_disabled() {
        let error = parse_key_program(
            b"2n@NF",
            KeyOptions {
                allow_numeric: false,
                allow_descending: true,
            },
        )
        .unwrap_err();

        assert_eq!(error, KeyParseError::UnsupportedNumeric);
    }

    #[test]
    fn rejects_descending_when_disabled() {
        let error = parse_key_program(
            b"2r@NF",
            KeyOptions {
                allow_numeric: true,
                allow_descending: false,
            },
        )
        .unwrap_err();

        assert_eq!(error, KeyParseError::UnsupportedDescending);
    }

    #[test]
    fn resolves_key_positions_for_fields_and_ranges() {
        let program = parse_key_program(b"2@NF-1/NF", KeyOptions::default()).unwrap();

        let positions = resolve_key_positions(&program, b"a b c d").unwrap();

        assert_eq!(
            positions,
            vec![
                ResolvedKeyPosition {
                    position: FieldPosition::from_one_based(2),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
                ResolvedKeyPosition {
                    position: FieldPosition::from_one_based(3),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
                ResolvedKeyPosition {
                    position: FieldPosition::from_one_based(4),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
            ]
        );
    }

    #[test]
    fn resolves_reverse_ranges_in_key_order() {
        let program = parse_key_program(b"4/2@NFr", KeyOptions::default()).unwrap();

        let positions = resolve_key_positions(&program, b"a b c d e").unwrap();

        assert_eq!(
            positions,
            vec![
                ResolvedKeyPosition {
                    position: FieldPosition::from_one_based(4),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
                ResolvedKeyPosition {
                    position: FieldPosition::from_one_based(3),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
                ResolvedKeyPosition {
                    position: FieldPosition::from_one_based(2),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
                ResolvedKeyPosition {
                    position: FieldPosition::from_one_based(5),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: true,
                    },
                },
            ]
        );
    }

    #[test]
    fn rejects_missing_field_during_position_resolution() {
        let program = parse_key_program(b"3", KeyOptions::default()).unwrap();
        let error = resolve_key_positions(&program, b"a b").unwrap_err();
        assert_eq!(error, KeyResolveError::MissingField);
    }

    #[test]
    fn rejects_nf_minus_when_it_resolves_to_zero() {
        let program = parse_key_program(b"NF-3", KeyOptions::default()).unwrap();
        let error = resolve_key_positions(&program, b"a b c").unwrap_err();
        assert_eq!(error, KeyResolveError::NfResolvedToZero);
    }

    #[test]
    fn rejects_empty_record_for_key_resolution() {
        let program = parse_key_program(b"NF", KeyOptions::default()).unwrap();
        let error = resolve_key_positions(&program, b"").unwrap_err();
        assert_eq!(error, KeyResolveError::MissingField);
    }

    #[test]
    fn resolves_key_values_in_key_order() {
        let program = parse_key_program(b"2@NF-1/NF", KeyOptions::default()).unwrap();

        let resolved = resolve_key(&program, b"a b c d").unwrap();

        assert_eq!(
            resolved,
            ResolvedKey::new(vec![
                ResolvedKeyField {
                    bytes: &b"b"[..],
                    position: FieldPosition::from_one_based(2),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
                ResolvedKeyField {
                    bytes: &b"c"[..],
                    position: FieldPosition::from_one_based(3),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
                ResolvedKeyField {
                    bytes: &b"d"[..],
                    position: FieldPosition::from_one_based(4),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
            ])
        );
    }

    #[test]
    fn resolves_reverse_ranges_with_values() {
        let program = parse_key_program(b"4/2@NFr", KeyOptions::default()).unwrap();

        let resolved = resolve_key(&program, b"a b c d e").unwrap();

        assert_eq!(
            resolved,
            ResolvedKey::new(vec![
                ResolvedKeyField {
                    bytes: &b"d"[..],
                    position: FieldPosition::from_one_based(4),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
                ResolvedKeyField {
                    bytes: &b"c"[..],
                    position: FieldPosition::from_one_based(3),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
                ResolvedKeyField {
                    bytes: &b"b"[..],
                    position: FieldPosition::from_one_based(2),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
                ResolvedKeyField {
                    bytes: &b"e"[..],
                    position: FieldPosition::from_one_based(5),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: true,
                    },
                },
            ])
        );
    }

    #[test]
    fn resolves_key_values_from_precomputed_positions() {
        let positions = vec![
            ResolvedKeyPosition {
                position: FieldPosition::from_one_based(1),
                attr: KeyAttr {
                    kind: KeyKind::Byte,
                    descending: false,
                },
            },
            ResolvedKeyPosition {
                position: FieldPosition::from_one_based(3),
                attr: KeyAttr {
                    kind: KeyKind::Numeric,
                    descending: true,
                },
            },
        ];

        let resolved = resolve_key_with_positions(&positions, b"x y 10").unwrap();

        assert_eq!(
            resolved,
            ResolvedKey::new(vec![
                ResolvedKeyField {
                    bytes: &b"x"[..],
                    position: FieldPosition::from_one_based(1),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
                ResolvedKeyField {
                    bytes: &b"10"[..],
                    position: FieldPosition::from_one_based(3),
                    attr: KeyAttr {
                        kind: KeyKind::Numeric,
                        descending: true,
                    },
                },
            ])
        );
    }

    #[test]
    fn rejects_missing_field_during_key_value_resolution() {
        let program = parse_key_program(b"3", KeyOptions::default()).unwrap();
        let error = resolve_key(&program, b"a b").unwrap_err();
        assert_eq!(error, KeyResolveError::MissingField);
    }

    #[test]
    fn rejects_missing_field_during_position_based_resolution() {
        let positions = vec![ResolvedKeyPosition {
            position: FieldPosition::from_one_based(3),
            attr: KeyAttr {
                kind: KeyKind::Byte,
                descending: false,
            },
        }];

        let error = resolve_key_with_positions(&positions, b"a b").unwrap_err();
        assert_eq!(error, KeyResolveError::MissingField);
    }

    #[test]
    fn compares_byte_keys_lexicographically() {
        let left = ResolvedKey::new(vec![
            ResolvedKeyField {
                bytes: &b"a"[..],
                position: FieldPosition::from_one_based(1),
                attr: KeyAttr::default(),
            },
            ResolvedKeyField {
                bytes: &b"z"[..],
                position: FieldPosition::from_one_based(2),
                attr: KeyAttr::default(),
            },
        ]);
        let right = ResolvedKey::new(vec![
            ResolvedKeyField {
                bytes: &b"a"[..],
                position: FieldPosition::from_one_based(1),
                attr: KeyAttr::default(),
            },
            ResolvedKeyField {
                bytes: &b"zz"[..],
                position: FieldPosition::from_one_based(2),
                attr: KeyAttr::default(),
            },
        ]);

        assert_eq!(
            compare_resolved_keys(&left, &right).unwrap(),
            Ordering::Less
        );
    }

    #[test]
    fn compares_numeric_keys_by_value() {
        let left = ResolvedKey::new(vec![ResolvedKeyField {
            bytes: &b"001.20"[..],
            position: FieldPosition::from_one_based(1),
            attr: KeyAttr {
                kind: KeyKind::Numeric,
                descending: false,
            },
        }]);
        let right = ResolvedKey::new(vec![ResolvedKeyField {
            bytes: &b"1.2"[..],
            position: FieldPosition::from_one_based(1),
            attr: KeyAttr {
                kind: KeyKind::Numeric,
                descending: false,
            },
        }]);

        assert_eq!(
            compare_resolved_keys(&left, &right).unwrap(),
            Ordering::Equal
        );
    }

    #[test]
    fn compares_negative_numeric_keys() {
        let left = ResolvedKey::new(vec![ResolvedKeyField {
            bytes: &b"-2"[..],
            position: FieldPosition::from_one_based(1),
            attr: KeyAttr {
                kind: KeyKind::Numeric,
                descending: false,
            },
        }]);
        let right = ResolvedKey::new(vec![ResolvedKeyField {
            bytes: &b"-10"[..],
            position: FieldPosition::from_one_based(1),
            attr: KeyAttr {
                kind: KeyKind::Numeric,
                descending: false,
            },
        }]);

        assert_eq!(
            compare_resolved_keys(&left, &right).unwrap(),
            Ordering::Greater
        );
    }

    #[test]
    fn reverses_order_for_descending_keys() {
        let left = ResolvedKey::new(vec![ResolvedKeyField {
            bytes: &b"2"[..],
            position: FieldPosition::from_one_based(1),
            attr: KeyAttr {
                kind: KeyKind::Numeric,
                descending: true,
            },
        }]);
        let right = ResolvedKey::new(vec![ResolvedKeyField {
            bytes: &b"10"[..],
            position: FieldPosition::from_one_based(1),
            attr: KeyAttr {
                kind: KeyKind::Numeric,
                descending: true,
            },
        }]);

        assert_eq!(
            compare_resolved_keys(&left, &right).unwrap(),
            Ordering::Greater
        );
    }

    #[test]
    fn rejects_invalid_numeric_value_during_comparison() {
        let left = ResolvedKey::new(vec![ResolvedKeyField {
            bytes: &b"1e3"[..],
            position: FieldPosition::from_one_based(1),
            attr: KeyAttr {
                kind: KeyKind::Numeric,
                descending: false,
            },
        }]);
        let right = ResolvedKey::new(vec![ResolvedKeyField {
            bytes: &b"1000"[..],
            position: FieldPosition::from_one_based(1),
            attr: KeyAttr {
                kind: KeyKind::Numeric,
                descending: false,
            },
        }]);

        let error = compare_resolved_keys(&left, &right).unwrap_err();
        assert_eq!(error, super::KeyCompareError::InvalidNumericValue);
    }

    #[test]
    fn rejects_key_length_mismatch_during_comparison() {
        let left = ResolvedKey::new(vec![ResolvedKeyField {
            bytes: &b"a"[..],
            position: FieldPosition::from_one_based(1),
            attr: KeyAttr::default(),
        }]);
        let right = ResolvedKey::new(vec![
            ResolvedKeyField {
                bytes: &b"a"[..],
                position: FieldPosition::from_one_based(1),
                attr: KeyAttr::default(),
            },
            ResolvedKeyField {
                bytes: &b"b"[..],
                position: FieldPosition::from_one_based(2),
                attr: KeyAttr::default(),
            },
        ]);

        let error = compare_resolved_keys(&left, &right).unwrap_err();
        assert_eq!(error, super::KeyCompareError::KeyLengthMismatch);
    }

    #[test]
    fn normalizes_key_positions_to_one_based_origin() {
        let positions = vec![
            ResolvedKeyPosition {
                position: FieldPosition::from_one_based(3),
                attr: KeyAttr {
                    kind: KeyKind::Byte,
                    descending: false,
                },
            },
            ResolvedKeyPosition {
                position: FieldPosition::from_one_based(5),
                attr: KeyAttr {
                    kind: KeyKind::Numeric,
                    descending: true,
                },
            },
            ResolvedKeyPosition {
                position: FieldPosition::from_one_based(4),
                attr: KeyAttr {
                    kind: KeyKind::Byte,
                    descending: false,
                },
            },
        ];

        assert_eq!(
            normalize_key_positions_to_one(&positions),
            vec![
                ResolvedKeyPosition {
                    position: FieldPosition::from_one_based(1),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
                ResolvedKeyPosition {
                    position: FieldPosition::from_one_based(3),
                    attr: KeyAttr {
                        kind: KeyKind::Numeric,
                        descending: true,
                    },
                },
                ResolvedKeyPosition {
                    position: FieldPosition::from_one_based(2),
                    attr: KeyAttr {
                        kind: KeyKind::Byte,
                        descending: false,
                    },
                },
            ]
        );
    }

    #[test]
    fn normalizes_empty_position_list_to_empty() {
        assert!(normalize_key_positions_to_one(&[]).is_empty());
    }
}
