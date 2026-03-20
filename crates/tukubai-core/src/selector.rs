use std::error::Error as StdError;
use std::fmt;

use crate::split_fields;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SelectorOptions {
    pub allow_zero: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectorProgram {
    selectors: Vec<Selector>,
}

impl SelectorProgram {
    pub fn selectors(&self) -> &[Selector] {
        &self.selectors
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Selector {
    FieldNumber(u64),
    RawRecord,
    LastField,
    LastFieldMinus(u64),
    Range(SelectorExpr, SelectorExpr),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectorExpr {
    FieldNumber(u64),
    LastField,
    LastFieldMinus(u64),
}

#[derive(Debug, Eq, PartialEq)]
pub enum SelectorParseError {
    UnsupportedZero,
    ZeroInRange,
    InvalidSyntax,
    InvalidNumber,
}

impl fmt::Display for SelectorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedZero => f.write_str("selector 0 is not supported by this command"),
            Self::ZeroInRange => f.write_str("selector 0 must not appear inside a range"),
            Self::InvalidSyntax => f.write_str("invalid selector syntax"),
            Self::InvalidNumber => f.write_str("invalid selector number"),
        }
    }
}

impl StdError for SelectorParseError {}

#[derive(Debug, Eq, PartialEq)]
pub enum SelectorResolveError {
    MissingField,
    NfResolvedToZero,
}

impl fmt::Display for SelectorResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField => f.write_str("selector resolved to a non-existent field"),
            Self::NfResolvedToZero => f.write_str("selector NF-<n> resolved to field 0"),
        }
    }
}

impl StdError for SelectorResolveError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolvedItem<'a> {
    Field(&'a [u8]),
    RawRecord(&'a [u8]),
}

pub fn parse_selectors<I, S>(
    inputs: I,
    options: SelectorOptions,
) -> Result<SelectorProgram, SelectorParseError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<[u8]>,
{
    let mut selectors = Vec::new();

    for input in inputs {
        selectors.push(parse_selector(input.as_ref(), options)?);
    }

    Ok(SelectorProgram { selectors })
}

pub fn resolve_selectors<'a>(
    program: &'a SelectorProgram,
    record: &'a [u8],
) -> Result<Vec<ResolvedItem<'a>>, SelectorResolveError> {
    let fields: Vec<&[u8]> = split_fields(record).collect();
    let mut resolved = Vec::new();

    for selector in program.selectors() {
        match selector {
            Selector::FieldNumber(number) => {
                resolved.push(ResolvedItem::Field(resolve_field_number(*number, &fields)?));
            }
            Selector::RawRecord => resolved.push(ResolvedItem::RawRecord(record)),
            Selector::LastField => {
                resolved.push(ResolvedItem::Field(resolve_last_field(&fields)?));
            }
            Selector::LastFieldMinus(offset) => {
                let index = resolve_last_field_minus(*offset, fields.len())?;
                resolved.push(ResolvedItem::Field(fields[index]));
            }
            Selector::Range(start, end) => {
                let start_index = resolve_expr(start, &fields)?;
                let end_index = resolve_expr(end, &fields)?;

                if start_index <= end_index {
                    for field in fields.iter().take(end_index + 1).skip(start_index) {
                        resolved.push(ResolvedItem::Field(field));
                    }
                } else {
                    for field in fields.iter().take(start_index + 1).skip(end_index).rev() {
                        resolved.push(ResolvedItem::Field(field));
                    }
                }
            }
        }
    }

    Ok(resolved)
}

fn parse_selector(input: &[u8], options: SelectorOptions) -> Result<Selector, SelectorParseError> {
    if let Some((left, right)) = split_range(input) {
        let start = parse_expr(left, options)?;
        let end = parse_expr(right, options)?;
        return Ok(Selector::Range(start, end));
    }

    parse_term(input, options)
}

fn split_range(input: &[u8]) -> Option<(&[u8], &[u8])> {
    let position = input.iter().position(|byte| *byte == b'/')?;
    let left = &input[..position];
    let right = &input[position + 1..];
    Some((left, right))
}

fn parse_expr(input: &[u8], options: SelectorOptions) -> Result<SelectorExpr, SelectorParseError> {
    match parse_term(input, options)? {
        Selector::FieldNumber(number) => Ok(SelectorExpr::FieldNumber(number)),
        Selector::LastField => Ok(SelectorExpr::LastField),
        Selector::LastFieldMinus(offset) => Ok(SelectorExpr::LastFieldMinus(offset)),
        Selector::RawRecord => Err(SelectorParseError::ZeroInRange),
        Selector::Range(_, _) => Err(SelectorParseError::InvalidSyntax),
    }
}

fn parse_term(input: &[u8], options: SelectorOptions) -> Result<Selector, SelectorParseError> {
    if input.is_empty() {
        return Err(SelectorParseError::InvalidSyntax);
    }

    if input == b"0" {
        return if options.allow_zero {
            Ok(Selector::RawRecord)
        } else {
            Err(SelectorParseError::UnsupportedZero)
        };
    }

    if input == b"NF" {
        return Ok(Selector::LastField);
    }

    if let Some(rest) = input.strip_prefix(b"NF-") {
        if rest.is_empty() {
            return Err(SelectorParseError::InvalidSyntax);
        }
        return Ok(Selector::LastFieldMinus(parse_number(rest)?));
    }

    if input.iter().all(|byte| byte.is_ascii_digit()) {
        return Ok(Selector::FieldNumber(parse_number(input)?));
    }

    Err(SelectorParseError::InvalidSyntax)
}

fn parse_number(input: &[u8]) -> Result<u64, SelectorParseError> {
    let mut value = 0_u64;

    for byte in input {
        if !byte.is_ascii_digit() {
            return Err(SelectorParseError::InvalidSyntax);
        }

        value = value
            .checked_mul(10)
            .and_then(|current| current.checked_add(u64::from(byte - b'0')))
            .ok_or(SelectorParseError::InvalidNumber)?;
    }

    Ok(value)
}

fn resolve_expr(expr: &SelectorExpr, fields: &[&[u8]]) -> Result<usize, SelectorResolveError> {
    match expr {
        SelectorExpr::FieldNumber(number) => resolve_field_index(*number, fields.len()),
        SelectorExpr::LastField => {
            let index = fields
                .len()
                .checked_sub(1)
                .ok_or(SelectorResolveError::MissingField)?;
            Ok(index)
        }
        SelectorExpr::LastFieldMinus(offset) => resolve_last_field_minus(*offset, fields.len()),
    }
}

fn resolve_field_number<'a>(
    number: u64,
    fields: &[&'a [u8]],
) -> Result<&'a [u8], SelectorResolveError> {
    let index = resolve_field_index(number, fields.len())?;
    Ok(fields[index])
}

fn resolve_last_field<'a>(fields: &[&'a [u8]]) -> Result<&'a [u8], SelectorResolveError> {
    fields
        .last()
        .copied()
        .ok_or(SelectorResolveError::MissingField)
}

fn resolve_field_index(number: u64, field_count: usize) -> Result<usize, SelectorResolveError> {
    if number == 0 {
        return Err(SelectorResolveError::MissingField);
    }

    let zero_based = usize::try_from(number - 1).map_err(|_| SelectorResolveError::MissingField)?;
    if zero_based >= field_count {
        return Err(SelectorResolveError::MissingField);
    }

    Ok(zero_based)
}

fn resolve_last_field_minus(
    offset: u64,
    field_count: usize,
) -> Result<usize, SelectorResolveError> {
    let offset = usize::try_from(offset).map_err(|_| SelectorResolveError::MissingField)?;

    if field_count == 0 {
        return Err(SelectorResolveError::MissingField);
    }

    if offset == field_count {
        return Err(SelectorResolveError::NfResolvedToZero);
    }

    if offset > field_count {
        return Err(SelectorResolveError::MissingField);
    }

    Ok(field_count - offset - 1)
}

#[cfg(test)]
mod tests {
    use super::{
        ResolvedItem, Selector, SelectorExpr, SelectorOptions, SelectorParseError,
        SelectorResolveError, parse_selectors, resolve_selectors,
    };

    #[test]
    fn parses_basic_selectors() {
        let program = parse_selectors(
            [
                b"1".as_slice(),
                b"01".as_slice(),
                b"NF".as_slice(),
                b"NF-0".as_slice(),
            ],
            SelectorOptions { allow_zero: true },
        )
        .unwrap();

        assert_eq!(
            program.selectors(),
            &[
                Selector::FieldNumber(1),
                Selector::FieldNumber(1),
                Selector::LastField,
                Selector::LastFieldMinus(0),
            ]
        );
    }

    #[test]
    fn parses_ranges() {
        let program = parse_selectors(
            [b"NF-1/NF".as_slice(), b"5/2".as_slice()],
            SelectorOptions { allow_zero: true },
        )
        .unwrap();

        assert_eq!(
            program.selectors(),
            &[
                Selector::Range(SelectorExpr::LastFieldMinus(1), SelectorExpr::LastField),
                Selector::Range(SelectorExpr::FieldNumber(5), SelectorExpr::FieldNumber(2)),
            ]
        );
    }

    #[test]
    fn rejects_zero_when_disabled() {
        let error = parse_selectors([b"0".as_slice()], SelectorOptions::default()).unwrap_err();
        assert_eq!(error, SelectorParseError::UnsupportedZero);
    }

    #[test]
    fn rejects_zero_inside_range() {
        let error =
            parse_selectors([b"0/3".as_slice()], SelectorOptions { allow_zero: true }).unwrap_err();
        assert_eq!(error, SelectorParseError::ZeroInRange);
    }

    #[test]
    fn rejects_invalid_selector_syntax() {
        for input in [
            b"nf".as_slice(),
            b"NF+1".as_slice(),
            b"1+2".as_slice(),
            b"4-3".as_slice(),
            b"+5".as_slice(),
            b"/5".as_slice(),
            b"3/".as_slice(),
        ] {
            let error = parse_selectors([input], SelectorOptions { allow_zero: true }).unwrap_err();
            assert_eq!(error, SelectorParseError::InvalidSyntax);
        }
    }

    #[test]
    fn resolves_fields_and_raw_record_in_selector_order() {
        let program = parse_selectors(
            [b"2".as_slice(), b"0".as_slice(), b"NF".as_slice()],
            SelectorOptions { allow_zero: true },
        )
        .unwrap();

        let resolved = resolve_selectors(&program, b"alpha beta gamma").unwrap();

        assert_eq!(
            resolved,
            vec![
                ResolvedItem::Field(&b"beta"[..]),
                ResolvedItem::RawRecord(&b"alpha beta gamma"[..]),
                ResolvedItem::Field(&b"gamma"[..]),
            ]
        );
    }

    #[test]
    fn resolves_reverse_ranges() {
        let program =
            parse_selectors([b"5/2".as_slice()], SelectorOptions { allow_zero: true }).unwrap();

        let resolved = resolve_selectors(&program, b"a b c d e").unwrap();

        assert_eq!(
            resolved,
            vec![
                ResolvedItem::Field(&b"e"[..]),
                ResolvedItem::Field(&b"d"[..]),
                ResolvedItem::Field(&b"c"[..]),
                ResolvedItem::Field(&b"b"[..]),
            ]
        );
    }

    #[test]
    fn preserves_duplicate_fields() {
        let program = parse_selectors(
            [b"2".as_slice(), b"2/4".as_slice(), b"3".as_slice()],
            SelectorOptions { allow_zero: true },
        )
        .unwrap();

        let resolved = resolve_selectors(&program, b"a b c d").unwrap();

        assert_eq!(
            resolved,
            vec![
                ResolvedItem::Field(&b"b"[..]),
                ResolvedItem::Field(&b"b"[..]),
                ResolvedItem::Field(&b"c"[..]),
                ResolvedItem::Field(&b"d"[..]),
                ResolvedItem::Field(&b"c"[..]),
            ]
        );
    }

    #[test]
    fn allows_empty_record_only_for_zero() {
        let zero_program =
            parse_selectors([b"0".as_slice()], SelectorOptions { allow_zero: true }).unwrap();
        let resolved = resolve_selectors(&zero_program, b"").unwrap();
        assert_eq!(resolved, vec![ResolvedItem::RawRecord(&b""[..])]);

        let field_program =
            parse_selectors([b"1".as_slice()], SelectorOptions { allow_zero: true }).unwrap();
        let error = resolve_selectors(&field_program, b"").unwrap_err();
        assert_eq!(error, SelectorResolveError::MissingField);
    }

    #[test]
    fn rejects_nf_minus_when_it_resolves_to_zero() {
        let program =
            parse_selectors([b"NF-3".as_slice()], SelectorOptions { allow_zero: true }).unwrap();

        let error = resolve_selectors(&program, b"a b c").unwrap_err();
        assert_eq!(error, SelectorResolveError::NfResolvedToZero);
    }

    #[test]
    fn rejects_missing_fields() {
        let program = parse_selectors(
            [b"NF-4".as_slice(), b"4".as_slice()],
            SelectorOptions { allow_zero: true },
        )
        .unwrap();

        let error = resolve_selectors(&program, b"a b c").unwrap_err();
        assert_eq!(error, SelectorResolveError::MissingField);
    }
}
