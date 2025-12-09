#[derive(Copy, Clone)]
pub struct IrisNetworkConfig {
    pub endian: Endian,
    pub size: SizeType,
}

impl Default for IrisNetworkConfig {
    fn default() -> Self {
        Self {
            endian: Default::default(),
            size: Default::default(),
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum Endian {
    #[default]
    Little,
    Big,
}
#[derive(Default, PartialEq, Clone, Copy)]
pub enum SizeType {
    U16,
    #[default]
    U32,
    U64,
}