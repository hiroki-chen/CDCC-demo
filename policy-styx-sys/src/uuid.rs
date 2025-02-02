#[repr(C, packed)]
#[derive(Default)]
pub struct Uuid {
    pub data: [u8; 16usize],
}
