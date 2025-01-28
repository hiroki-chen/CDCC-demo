#[repr(C, packed)]
pub struct Uuid {
    pub data: [u8; 16usize],
}
