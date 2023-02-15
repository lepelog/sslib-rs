enum FlowRef {
    Id(u16),
    Ref(&'static str),
}

pub enum PatchKind {
    TBox {
        stage: &'static str,
        room: u8,
        layer: u8,
        tbox_id: u8,
    },
    Item {
        stage: &'static str,
        room: u8,
        layer: u8,
        flag: u8,
    },
    Soil {
        stage: &'static str,
        room: u8,
        layer: u8,
        flag: u8,
    },
    chest {
        stage: &'static str,
        room: u8,
        layer: u8,
        flag: u8,
    },
    EBc {
        stage: &'static str,
        room: u8,
        layer: u8,
        id: u16,
    },
    Chandel {
        stage: &'static str,
        room: u8,
        layer: u8,
    },
    WarpObj {
        stage: &'static str,
        room: u8,
        layer: u8,
    },
    HeartCo {
        stage: &'static str,
        room: u8,
        layer: u8,
    },
    SwSB {
        stage: &'static str,
        room: u8,
        layer: u8,
        index: u8,
    },
    FlowById {
        file: &'static str,
        index: u16,
    },
    FlowByName {
        file: &'static str,
        name: &'static str,
    },
    Oarc {
        stage: &'static str,
        layer: u8,
    },
    ShpSmpl {
        index: u8,
    },
}
