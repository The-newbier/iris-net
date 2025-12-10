/// This is are Settings how the Message will be encoded, decoded and send
#[derive(Copy, Clone)]
pub struct IrisNetworkConfig {
    /// This says how the message is formated
    pub endian: Endian,
    /// This says how big the Buffer-Limit is
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

pub enum Protocol {
    TCP,
    UDP
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