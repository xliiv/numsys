//! A rust library converting number's base (AKA radix).
//!
//! For more type-centric solution see [radix](https://docs.rs/radix) crate
//!
//!
//! ### TODO:
//!
//! * works for u8, u16, u32, u64, optionally u128
//!     * [solution](https://doc.rust-lang.org/src/core/num/mod.rs.html#2272-2282)
//!


extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate lazy_static;


use std::collections::HashMap;


lazy_static! {
    /// `Vector` of `char` containing digits from `0` to `9` (included)
    pub static ref DIGITS: Vec<char> = (b'0'..b'9'+1).map(|x| x as char).collect();
    /// `Vector` of `char` containing upper ASCII letters
    pub static ref UPPER_AZ: Vec<char> = (b'A'..b'Z'+1).map(|x| x as char).collect();
    /// `Vector` of `char` containing sum of `DIGITS` and `UPPER_AZ`
    pub static ref DIGITS_UPPER_AZ: Vec<char> = [&DIGITS[..], &UPPER_AZ[..]].concat();
    /// Length of `DIGITS_UPPER_AZ` as `usize`
    pub static ref D_UAZ_LEN: usize = DIGITS_UPPER_AZ.len();
}


#[derive(Debug, PartialEq, Fail)]
pub enum NewError {
    #[fail(display = "{}", text)] BaseTooSmall { text: String },
    #[fail(display = "{}", text)] BaseTooBig { text: String },
    #[fail(display = "DictEmpty")] DictEmpty,
    #[fail(display = "{}", text)] MultipleChar { text: String },
    #[fail(display = "{}", text)] MissingChar { text: String },
}


/// Converts base of `decimal` to `base`.
///
/// Revert operation is defined in rust std [`usize::from_str_radix`](
/// https://doc.rust-lang.org/stable/std/primitive.usize.html#method.from_str_radix)
///
/// # Examples
///
/// Basic usage
///
/// ```
/// use numsys::switch_dec_base;
///
/// assert_eq!(switch_dec_base(10, 16), Ok("A".to_string()));
/// assert_eq!(switch_dec_base(10, 2), Ok("1010".to_string()));
/// assert_eq!(switch_dec_base(10, 10), Ok("10".to_string()));
/// assert_eq!(switch_dec_base(10, 3), Ok("101".to_string()));
/// ```
///
/// # Errors
///
/// * Returns `NewError::BaseTooSmall` when `base` is less then 2
///
/// ```
/// use numsys::switch_dec_base;
/// use numsys::NewError;
///
/// let msg = "Base MUST be 2 or higer, given 1".to_string();
/// assert_eq!(switch_dec_base(10, 1), Err(NewError::BaseTooSmall{ text: msg }));
/// ```
///
/// * Returns `NewError::BaseTooBig` when `base` is greater then 36
///
/// ```
/// use numsys::switch_dec_base;
/// use numsys::NewError;
///
/// let msg = "Base MUST be at most 36, given 37".to_string();
/// assert_eq!(switch_dec_base(10, 37), Err(NewError::BaseTooBig{ text: msg }));
/// ```
pub fn switch_dec_base(decimal: usize, base: usize) -> Result<String, NewError> {
    if base < 2 {
        return Err(NewError::BaseTooSmall {
            text: format!("Base MUST be 2 or higer, given {}", base),
        });
    };
    if base > *D_UAZ_LEN {
        return Err(NewError::BaseTooBig {
            text: format!("Base MUST be at most {}, given {}", *D_UAZ_LEN, base),
        });
    };
    if decimal == 0 {
        return Ok("0".into());
    }

    let result = match base {
        2 => format!("{:b}", decimal),
        8 => format!("{:o}", decimal),
        10 => format!("{}", decimal),
        16 => format!("{:X}", decimal),
        _ => {
            let char_set = &DIGITS_UPPER_AZ[0..base];
            dec2seq(decimal, char_set)?
        }
    };
    Ok(result)
}


/// Converts `sequence` to decimal using `char2val` translation.
///
/// # Examples
///
/// ```
/// use numsys::seq2dec;
///
/// assert_eq!(seq2dec("BABA", &['A', 'B']), Ok(10));
/// assert_eq!(seq2dec("1010", &['0', '1']), Ok(10));
/// assert_eq!(seq2dec("☆★☆★", &['★', '☆']), Ok(10));
/// ```
///
/// # Errors
///
/// * Returns `NewError::DictEmpty` when `char2val` length is 0.
///
/// ```
/// use numsys::seq2dec;
/// use numsys::NewError;
///
/// assert_eq!(seq2dec("1010", &[]), Err(NewError::DictEmpty));
/// ```
///
/// * Returns `NewError::MultipleChar` when `char2val` includes duplicated chars.
///
/// ```
/// use numsys::seq2dec;
/// use numsys::NewError;
///
/// let detailed_msg = "Chars MUST be unique, duplicated: \'A\' in [\'A\', \'A\']".to_string();
/// assert_eq!(seq2dec("1010", &['A', 'A']), Err(NewError::MultipleChar{ text: detailed_msg }));
/// ```
///
/// * Returns `NewError::MissingChar` when `char2val` missing a char or more.
///
/// ```
/// use numsys::seq2dec;
/// use numsys::NewError;
///
/// let detailed_msg = "Char \'2\' not found in: [\'0\']".to_string();
/// assert_eq!(seq2dec("20", &['0']), Err(NewError::MissingChar{ text: detailed_msg }));
/// ```
///
pub fn seq2dec<S: AsRef<str>>(sequence: S, char2val: &[char]) -> Result<usize, NewError> {
    let from_base = char2val.len();
    if from_base == 0 {
        return Err(NewError::DictEmpty);
    }
    let single_char_sequence = {
        let uniques: HashMap<_, _> = sequence.as_ref().chars().map(|c| (c, 0)).collect();
        uniques.len() == 1
    };
    if from_base == 1 && single_char_sequence {
        return Ok(sequence.as_ref().len());
    }
    let mut _char2val = {
        let mut hm: HashMap<char, usize> = HashMap::new();
        for (idx, elem) in char2val.iter().enumerate() {
            if hm.insert(*elem, idx).is_some() {
                let msg = format!(
                    "Chars MUST be unique, duplicated: {:?} in {:?}",
                    elem,
                    char2val
                );
                return Err(NewError::MultipleChar { text: msg });
            }
        }
        hm
    };
    let mut dec: usize = 0;
    for (idx, glyph) in sequence.as_ref().chars().rev().enumerate() {
        let value = _char2val.get(&glyph).ok_or_else(|| {
            NewError::MissingChar {
                text: format!("Char {:?} not found in: {:?}", glyph, char2val),
            }
        })?;
        dec += value * from_base.pow(idx as u32);
    }
    Ok(dec)
}


/// Converts `decimal` using `char2val` translation.
///
/// # Examples
///
/// ```
/// use numsys::dec2seq;
///
/// assert_eq!(dec2seq(10, &['0', '1']), Ok("1010".to_string()));
/// assert_eq!(dec2seq(10, &['A', 'B']), Ok("BABA".to_string()));
/// assert_eq!(dec2seq(10, &['★', '☆']), Ok("☆★☆★".to_string()));
/// ```
///
/// # Errors
///
/// * Returns `NewError::DictEmpty` when `char2val` length is 0
///
/// ```
/// use numsys::dec2seq;
/// use numsys::NewError;
///
/// assert_eq!(dec2seq(10, &[]), Err(NewError::DictEmpty));
/// ```
///
pub fn dec2seq(mut decimal: usize, char2val: &[char]) -> Result<String, NewError> {
    let base = char2val.len();
    if base == 0 {
        return Err(NewError::DictEmpty);
    }
    if base == 1 {
        return Ok(char2val[0].to_string().repeat(decimal));
    }
    let mut sequence = String::new();
    while decimal != 0 {
        let glyph = match char2val.get(decimal % base) {
            Some(x) => x,
            // base == char2val lenght, so always lands inside
            None => unreachable!(),
        };
        sequence.insert(0, *glyph);
        decimal /= base;
    }
    Ok(sequence)
}

#[cfg(test)]
mod tests {
    use ::*;

    // TODO: add tests which shows that places "as u32" are broken

    #[test]
    fn dec2seq_works_when_dict_has_single_element() {
        let result = dec2seq(10, &['a']);
        assert_eq!(result, Ok("aaaaaaaaaa".to_string()));
    }

    #[test]
    fn dec2seq_and_seq2dec_are_reversible_when_dict_len_1() {
        let number = 10;
        let dict = ['a'];
        let seq = dec2seq(number, &dict).expect("First conversion failed");
        assert_eq!(seq2dec(seq, &dict), Ok(number));
    }
}
