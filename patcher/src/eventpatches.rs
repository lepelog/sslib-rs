use msb::{FlowEntry, Msbf, Msbt};

fn give_item(itemid: u8) -> FlowEntry {
    FlowEntry::Flow {
        subtype: 1,
        param1: itemid.into(),
        param2: u16::MAX,
        next: -1,
        param3: 10,
    }
}

pub fn do_eventpatches(file: &str, msbf: &mut Msbf, msbt: &mut Msbt) {
    match file {
        "114-Friend" => {}
        _ => (),
    }
}
