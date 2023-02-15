use std::borrow::Cow;

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

struct Flow3;

struct Switch;

fn speak_text(t: &str) -> FlowEntry {
    todo!()
}

pub enum Flowref {
    Index(i16),
    Name(Cow<'static, str>),
}

impl From<&'static str> for Flowref {
    fn from(s: &'static str) -> Self {
        Self::Name(s.into())
    }
}

impl From<String> for Flowref {
    fn from(s: String) -> Self {
        Self::Name(s.into())
    }
}

impl From<i16> for Flowref {
    fn from(s: i16) -> Self {
        Self::Index(s)
    }
}

struct FlowpatchManager;
struct FlowEdit<'a>(&'a mut FlowpatchManager);

impl FlowpatchManager {
    pub fn add_flow(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        flow: FlowEntry,
        next: impl Into<Flowref>,
    ) {
    }

    pub fn add_switch(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        flow: Switch,
        branches: &[Flowref],
    ) {
    }

    pub fn get_flow(&mut self, index: i16) -> FlowEdit {
        FlowEdit(self)
    }
}

impl<'a> FlowEdit<'a> {
    // for type3
    pub fn set_action(&mut self, flow: FlowEntry) -> &mut Self {
        self
    }
    // for type1
    pub fn set_text(&mut self, text: impl Into<Cow<'static, str>>) {}
    pub fn set_next(&mut self, next: impl Into<Flowref>) -> &mut Self {
        self
    }
}

pub fn do_eventpatches(file: &str, msbf: &mut Msbf, msbt: &mut Msbt) {
    let mut mgr = FlowpatchManager;
    match file {
        "114-Friend" => {
            mgr.get_flow(14).set_next("Fledge Item 1");
            mgr.add_flow("Fledge Item 1", give_item(0xba), "Fledge Item 2");
            mgr.add_flow("Fledge Item 2", give_item(0xba), -1);
        }
        _ => (),
    }
}
