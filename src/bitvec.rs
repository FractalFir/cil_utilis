#[derive(Copy, Clone, Debug)]
pub(crate) struct BitVec64(u64);
impl From<u64> for BitVec64 {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
impl BitVec64 {
    pub fn bit(self, bit: u8) -> bool {
        (self.0 >> bit) & 1 != 0
    }
}
impl IntoIterator for BitVec64 {
    type Item = u8;

    type IntoIter = BitVec64Iter;

    fn into_iter(self) -> Self::IntoIter {
        BitVec64Iter { index: 0, bv: self }
    }
}
pub struct BitVec64Iter {
    bv: BitVec64,
    index: u8,
}
impl Iterator for BitVec64Iter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        for index in self.index..63 {
            if self.bv.bit(index) {
                self.index = index + 1;
                return Some(index);
            }
        }
        None
    }
}
#[test]
fn check_bits() {
    let bitset: BitVec64 = 0b1.into();
    assert!(bitset.bit(0));
    let bitset: BitVec64 = 0b1001.into();
    assert!(bitset.bit(0));
    assert!(bitset.bit(3));
}
#[test]
fn iterate() {
    let bitset: BitVec64 = 0b1001.into();
    assert!(bitset.bit(3));
    let mut iterator = bitset.into_iter();
    assert_eq!(Some(0), iterator.next());
    assert_eq!(Some(3), iterator.next());
    let bitset: BitVec64 = 0b1000.into();
    assert!(!bitset.bit(0));
    let mut iterator = bitset.into_iter();
    assert_eq!(Some(3), iterator.next());
}
