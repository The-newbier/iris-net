#[derive(Copy, Clone)]
pub struct IrisConfig {
    pub endian: Endian,
}

impl Default for IrisConfig {
    fn default() -> Self {
        IrisConfig {
            endian: Default::default(),
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum Endian {
    #[default]
    Little,
    Big,
}

pub enum HeaderType {
    U16,
    U32,
    U64
}

/*fn get_bincode_config(config: IrisConfig) -> impl bincode::config::Config {
    let cfg = bincode::config::standard();

    match config.endian {
        Endian::Big => cfg.with_big_endian(),
        Endian::Little => cfg.with_little_endian(),
    }
}*/
