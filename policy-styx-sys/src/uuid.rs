#[repr(C, packed)]
#[derive(Default, Eq, PartialEq, PartialOrd, Ord, Debug, Clone)]
pub struct Uuid {
    pub data: [u8; 16usize],
}
