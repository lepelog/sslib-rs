enum FlowRef {
    Id(u16),
    Ref(&'static str),
    SRef(String),
}

enum PatckKind {
    Tbox {
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
    FlowById {
        file: &'static str,
        id: u16,
    },
    FlowByName {
        file: &'static str,
        name: &'static str,
    },
    Oarc {
        stage: &'static str,
        layer: u8,
    },
}

use PatckKind::*;

const patches: &[(&'static str, &'static [PatckKind])] = &[(
    "Fledge",
    &[
        FlowById {
            file: "frend",
            id: 123,
        },
        Oarc {
            stage: "F001r",
            layer: 4,
        },
        Oarc {
            stage: "F001r",
            layer: 3,
        },
    ],
)];
