//! Cursor stepping arithmetic (ported from PR #26 `src/core/nav.rs`,
//! commit f4867e9b5).

/// Move a cursor by `delta` rows, clamping or wrapping at the edges.
pub fn step_index(cursor: usize, len: usize, delta: isize, wrap: bool) -> usize {
    if len == 0 {
        return 0;
    }
    let to = cursor as isize + delta;
    if wrap {
        to.rem_euclid(len as isize) as usize
    } else {
        to.clamp(0, len as isize - 1) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_clamp_or_wrap() {
        assert_eq!(step_index(0, 3, -1, false), 0);
        assert_eq!(step_index(2, 3, 1, false), 2);
        assert_eq!(step_index(0, 3, -1, true), 2);
        assert_eq!(step_index(2, 3, 1, true), 0);
        assert_eq!(step_index(1, 3, 1, true), 2);
        assert_eq!(step_index(0, 0, 1, true), 0);
    }
}
