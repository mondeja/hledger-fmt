pub(crate) fn leading_commodity_len_from_units(units: &str) -> usize {
    let mut len = 0;
    for c in units.chars() {
        if c.is_ascii_digit() {
            break;
        }
        len += 1;
    }
    len
}

pub(crate) fn trailing_commodity_len_from_units(units: &str) -> usize {
    let mut len = 0;
    for c in units.chars().rev() {
        if c.is_ascii_digit() {
            break;
        }
        len += 1;
    }
    len
}
