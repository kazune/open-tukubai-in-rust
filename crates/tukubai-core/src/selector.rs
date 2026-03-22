use std::error::Error as StdError;
use std::fmt;

use crate::split_fields;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectorOptions {
    pub allow_zero: bool,
    pub allow_range: bool,
}

impl SelectorOptions {
    pub const fn single_field(allow_zero: bool) -> Self {
        Self {
            allow_zero,
            allow_range: false,
        }
    }

    pub const fn multi_field(allow_zero: bool) -> Self {
        Self {
            allow_zero,
            allow_range: true,
        }
    }
}

impl Default for SelectorOptions {
    fn default() -> Self {
        Self::multi_field(false)
    }
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
    UnsupportedRange,
    ZeroInRange,
    InvalidSyntax,
    InvalidNumber,
}

impl fmt::Display for SelectorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedZero => f.write_str("selector 0 is not supported by this command"),
            Self::UnsupportedRange => {
                f.write_str("range selectors are not supported by this command")
            }
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
    UnsupportedZero,
}

impl fmt::Display for SelectorResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField => f.write_str("selector resolved to a non-existent field"),
            Self::NfResolvedToZero => f.write_str("selector NF-<n> resolved to field 0"),
            Self::UnsupportedZero => {
                f.write_str("selector 0 cannot be resolved to a field position")
            }
        }
    }
}

impl StdError for SelectorResolveError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolvedItem<'a> {
    Field(&'a [u8]),
    RawRecord(&'a [u8]),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FieldPosition(u64);

impl FieldPosition {
    fn from_one_based(value: u64) -> Self {
        debug_assert!(value > 0);
        Self(value)
    }

    pub fn get(self) -> u64 {
        self.0
    }

    pub fn to_zero_based(self) -> Option<usize> {
        usize::try_from(self.0.checked_sub(1)?).ok()
    }
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

pub fn resolve_selector_positions(
    program: &SelectorProgram,
    record: &[u8],
) -> Result<Vec<FieldPosition>, SelectorResolveError> {
    let fields: Vec<&[u8]> = split_fields(record).collect();
    let mut resolved = Vec::new();

    for selector in program.selectors() {
        match selector {
            Selector::FieldNumber(number) => {
                resolve_field_index(*number, fields.len())?;
                resolved.push(FieldPosition::from_one_based(*number));
            }
            Selector::RawRecord => return Err(SelectorResolveError::UnsupportedZero),
            Selector::LastField => {
                let number = resolve_last_field_number(fields.len())?;
                resolved.push(FieldPosition::from_one_based(number));
            }
            Selector::LastFieldMinus(offset) => {
                let number = resolve_last_field_minus_number(*offset, fields.len())?;
                resolved.push(FieldPosition::from_one_based(number));
            }
            Selector::Range(start, end) => {
                let start_number = resolve_expr_number(start, fields.len())?;
                let end_number = resolve_expr_number(end, fields.len())?;

                if start_number <= end_number {
                    for number in start_number..=end_number {
                        resolved.push(FieldPosition::from_one_based(number));
                    }
                } else {
                    for number in (end_number..=start_number).rev() {
                        resolved.push(FieldPosition::from_one_based(number));
                    }
                }
            }
        }
    }

    Ok(resolved)
}

fn parse_selector(input: &[u8], options: SelectorOptions) -> Result<Selector, SelectorParseError> {
    if let Some((left, right)) = split_range(input) {
        if !options.allow_range {
            return Err(SelectorParseError::UnsupportedRange);
        }

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

fn resolve_expr_number(
    expr: &SelectorExpr,
    field_count: usize,
) -> Result<u64, SelectorResolveError> {
    match expr {
        SelectorExpr::FieldNumber(number) => {
            resolve_field_index(*number, field_count)?;
            Ok(*number)
        }
        SelectorExpr::LastField => resolve_last_field_number(field_count),
        SelectorExpr::LastFieldMinus(offset) => {
            resolve_last_field_minus_number(*offset, field_count)
        }
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

fn resolve_last_field_number(field_count: usize) -> Result<u64, SelectorResolveError> {
    if field_count == 0 {
        return Err(SelectorResolveError::MissingField);
    }

    u64::try_from(field_count).map_err(|_| SelectorResolveError::MissingField)
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
    let number = resolve_last_field_minus_number(offset, field_count)?;
    resolve_field_index(number, field_count)
}

fn resolve_last_field_minus_number(
    offset: u64,
    field_count: usize,
) -> Result<u64, SelectorResolveError> {
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

    u64::try_from(field_count - offset).map_err(|_| SelectorResolveError::MissingField)
}

#[cfg(test)]
mod tests {
    use super::{
        FieldPosition, ResolvedItem, Selector, SelectorExpr, SelectorOptions, SelectorParseError,
        SelectorResolveError, parse_selectors, resolve_selector_positions, resolve_selectors,
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
            SelectorOptions::multi_field(true),
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
            SelectorOptions::multi_field(true),
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
    fn rejects_ranges_when_disabled() {
        let error =
            parse_selectors([b"1/2".as_slice()], SelectorOptions::single_field(false)).unwrap_err();
        assert_eq!(error, SelectorParseError::UnsupportedRange);
    }

    #[test]
    fn rejects_zero_inside_range() {
        let error =
            parse_selectors([b"0/3".as_slice()], SelectorOptions::multi_field(true)).unwrap_err();
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
            let error = parse_selectors([input], SelectorOptions::multi_field(true)).unwrap_err();
            assert_eq!(error, SelectorParseError::InvalidSyntax);
        }
    }

    #[test]
    fn resolves_fields_and_raw_record_in_selector_order() {
        let program = parse_selectors(
            [b"2".as_slice(), b"0".as_slice(), b"NF".as_slice()],
            SelectorOptions::multi_field(true),
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
            parse_selectors([b"5/2".as_slice()], SelectorOptions::multi_field(true)).unwrap();

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
            SelectorOptions::multi_field(true),
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
    fn resolves_positions_in_selector_order() {
        let program = parse_selectors(
            [b"2".as_slice(), b"NF-1/NF".as_slice(), b"2".as_slice()],
            SelectorOptions::default(),
        )
        .unwrap();

        let resolved = resolve_selector_positions(&program, b"a b c d").unwrap();

        assert_eq!(
            resolved,
            vec![
                FieldPosition::from_one_based(2),
                FieldPosition::from_one_based(3),
                FieldPosition::from_one_based(4),
                FieldPosition::from_one_based(2),
            ]
        );
    }

    #[test]
    fn resolves_reverse_range_positions() {
        let program = parse_selectors([b"5/2".as_slice()], SelectorOptions::default()).unwrap();

        let resolved = resolve_selector_positions(&program, b"a b c d e").unwrap();

        assert_eq!(
            resolved,
            vec![
                FieldPosition::from_one_based(5),
                FieldPosition::from_one_based(4),
                FieldPosition::from_one_based(3),
                FieldPosition::from_one_based(2),
            ]
        );
    }

    #[test]
    fn rejects_zero_for_position_resolution() {
        let program =
            parse_selectors([b"0".as_slice()], SelectorOptions::multi_field(true)).unwrap();

        let error = resolve_selector_positions(&program, b"a b").unwrap_err();
        assert_eq!(error, SelectorResolveError::UnsupportedZero);
    }

    #[test]
    fn field_position_converts_to_zero_based() {
        let position = FieldPosition::from_one_based(3);

        assert_eq!(position.get(), 3);
        assert_eq!(position.to_zero_based(), Some(2));
    }

    #[test]
    fn allows_empty_record_only_for_zero() {
        let zero_program =
            parse_selectors([b"0".as_slice()], SelectorOptions::multi_field(true)).unwrap();
        let resolved = resolve_selectors(&zero_program, b"").unwrap();
        assert_eq!(resolved, vec![ResolvedItem::RawRecord(&b""[..])]);

        let field_program =
            parse_selectors([b"1".as_slice()], SelectorOptions::multi_field(true)).unwrap();
        let error = resolve_selectors(&field_program, b"").unwrap_err();
        assert_eq!(error, SelectorResolveError::MissingField);
    }

    #[test]
    fn rejects_nf_minus_when_it_resolves_to_zero() {
        let program =
            parse_selectors([b"NF-3".as_slice()], SelectorOptions::multi_field(true)).unwrap();

        let error = resolve_selectors(&program, b"a b c").unwrap_err();
        assert_eq!(error, SelectorResolveError::NfResolvedToZero);
    }

    #[test]
    fn rejects_missing_fields() {
        let program = parse_selectors(
            [b"NF-4".as_slice(), b"4".as_slice()],
            SelectorOptions::multi_field(true),
        )
        .unwrap();

        let error = resolve_selectors(&program, b"a b c").unwrap_err();
        assert_eq!(error, SelectorResolveError::MissingField);
    }
}
