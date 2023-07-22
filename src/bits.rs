//! Convenient bit-manipulation operations

use std::ops::RangeInclusive;

pub trait Bits {
    const BIT_COUNT: usize;

    fn bit(&self, index: usize) -> bool;
    fn bits(&self, range: RangeInclusive<usize>) -> Self;
    fn set_bit(&mut self, index: usize);
    fn reset_bit(&mut self, index: usize);
}

impl Bits for u8 {
    const BIT_COUNT: usize = 8;

    fn bit(&self, index: usize) -> bool {
        assert!(index < Self::BIT_COUNT);
        (self >> index) & 1 == 1
    }

    fn bits(&self, range: RangeInclusive<usize>) -> Self {
        if range.is_empty() {
            Self::default()
        } else {
            assert!(*range.end() < Self::BIT_COUNT);
            (self << (Self::BIT_COUNT - 1 - range.end()))
                >> (Self::BIT_COUNT - 1 - range.end() + range.start())
        }
    }

    fn set_bit(&mut self, index: usize) {
        assert!(index < Self::BIT_COUNT);
        *self |= 1 << index;
    }

    fn reset_bit(&mut self, index: usize) {
        assert!(index < Self::BIT_COUNT);
        *self &= !(1 << index);
    }
}

#[cfg(test)]
mod test {
    use super::Bits;

    #[test]
    fn bit() {
        let byte: u8 = 0b1010_1010;
        assert_eq!(byte.bit(0), false);
        assert_eq!(byte.bit(1), true);
        assert_eq!(byte.bit(2), false);
        assert_eq!(byte.bit(3), true);
        assert_eq!(byte.bit(4), false);
        assert_eq!(byte.bit(5), true);
        assert_eq!(byte.bit(6), false);
        assert_eq!(byte.bit(7), true);

        assert_eq!(!byte.bit(0), true);
        assert_eq!(!byte.bit(1), false);
        assert_eq!(!byte.bit(2), true);
        assert_eq!(!byte.bit(3), false);
        assert_eq!(!byte.bit(4), true);
        assert_eq!(!byte.bit(5), false);
        assert_eq!(!byte.bit(6), true);
        assert_eq!(!byte.bit(7), false);
    }

    #[test]
    fn bits() {
        let byte: u8 = 0b1010_1100;
        assert_eq!(byte.bits(0..=3), 0b1100);
        assert_eq!(byte.bits(4..=7), 0b1010);
        assert_eq!(byte.bits(2..=5), 0b1011);
    }

    #[test]
    fn set_bit() {
        let mut byte: u8 = 0b1100_1010;
        byte.set_bit(5);
        assert_eq!(byte, 0b1110_1010);
        byte.set_bit(4);
        assert_eq!(byte, 0b1111_1010);
        byte.set_bit(2);
        assert_eq!(byte, 0b1111_1110);
        byte.set_bit(0);
        assert_eq!(byte, 0b1111_1111);
    }

    #[test]
    fn reset_bit() {
        let mut byte: u8 = 0b1100_1010;
        byte.reset_bit(7);
        assert_eq!(byte, 0b0100_1010);
        byte.reset_bit(6);
        assert_eq!(byte, 0b0000_1010);
        byte.reset_bit(3);
        assert_eq!(byte, 0b0000_0010);
        byte.reset_bit(1);
        assert_eq!(byte, 0b0000_0000);
    }
}
