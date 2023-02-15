//! make it easier to edit MSBF flows
//!
//!

use std::{borrow::Cow, collections::HashMap, mem::swap};

use crate::{FlowEntry, Msbf, Msbt, TextSegment};

#[derive(Debug, thiserror::Error)]
pub enum FlowPatchError {
    #[error("out of range: {0}")]
    OutOfRange(i16),
    #[error("not a flow")]
    NotFlow,
    #[error("a switch was not expected here")]
    UnexpectedSwitch,
    #[error("a switch was expected here")]
    ExpectedSwitch,
    #[error("name not found: {0}")]
    NameNotFound(Cow<'static, str>),
}

#[derive(Debug, Clone)]
pub enum Flowref {
    Index(i16),
    Name(Cow<'static, str>),
}

impl Flowref {
    pub fn end() -> Self {
        Self::Index(-1)
    }
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


#[derive(Debug, Clone)]
pub enum RefFlowEntry {
    NonDiverging(RefFlowEntryNonDiverging),
    Diverging(RefFlowEntryDiverging),
}

#[derive(Debug, Clone)]
pub enum RefFlowEntryNonDiverging {
    Start {
        next: Flowref,
    },
    Text {
        file: u16,
        line: u16,
        next: Flowref,
    },
    Flow {
        subtype: u8,
        param1: u16,
        param2: u16,
        next: Flowref,
        param3: i16,
    },
}

#[derive(Debug, Clone)]
pub struct RefFlowEntryDiverging {
    pub subtype: u8,
    pub param1: u16,
    pub param2: u16,
    pub param3: i16,
    pub branches: Vec<Flowref>,
}

impl From<FlowEntry> for RefFlowEntry {
    fn from(value: FlowEntry) -> Self {
        match value {
            FlowEntry::Start { next } => {
                RefFlowEntry::NonDiverging(RefFlowEntryNonDiverging::Start { next: next.into() })
            }
            FlowEntry::Text { file, line, next } => {
                RefFlowEntry::NonDiverging(RefFlowEntryNonDiverging::Text {
                    file,
                    line,
                    next: next.into(),
                })
            }
            FlowEntry::Flow {
                subtype,
                param1,
                param2,
                next,
                param3,
            } => RefFlowEntry::NonDiverging(RefFlowEntryNonDiverging::Flow {
                subtype,
                param1,
                param2,
                next: next.into(),
                param3,
            }),
            FlowEntry::Switch {
                subtype,
                param1,
                param2,
                param3,
                branches,
            } => RefFlowEntry::Diverging(RefFlowEntryDiverging {
                subtype,
                param1,
                param2,
                param3,
                branches: branches.into_iter().map(Into::into).collect(),
            }),
        }
    }
}

impl From<RefFlowEntryDiverging> for RefFlowEntry {
    fn from(value: RefFlowEntryDiverging) -> Self {
        Self::Diverging(value)
    }
}

impl From<RefFlowEntryNonDiverging> for RefFlowEntry {
    fn from(value: RefFlowEntryNonDiverging) -> Self {
        Self::NonDiverging(value)
    }
}

impl RefFlowEntryNonDiverging {
    pub fn with_next(mut self, new_next: impl Into<Flowref>) -> Self {
        *self.get_next_mut() = new_next.into();
        self
    }

    pub fn get_next(&self) -> &Flowref {
        match self {
            Self::Start { next } | Self::Flow { next, .. } | Self::Text { next, .. } => {
                next
            }
        }
    }

    pub fn get_next_mut(&mut self) -> &mut Flowref {
        match self {
            Self::Start { next } | Self::Flow { next, .. } | Self::Text { next, .. } => {
                next
            }
        }
    }
}

pub mod flow {
    use super::{Flowref, RefFlowEntry, RefFlowEntryNonDiverging};

    pub fn start() -> RefFlowEntryNonDiverging {
        RefFlowEntryNonDiverging::Start {
            next: Flowref::end(),
        }
    }

    pub fn set_storyflag(flag: u16) -> RefFlowEntryNonDiverging {
        RefFlowEntryNonDiverging::Flow { subtype: 0, param1: 0, param2: flag, next: Flowref::end(), param3: 0 }
    }

    pub fn unset_storyflag(flag: u16) -> RefFlowEntryNonDiverging {
        RefFlowEntryNonDiverging::Flow { subtype: 0, param1: 0, param2: flag, next: Flowref::end(), param3: 1 }
    }

    pub fn add_rupees(rupees: i16) -> RefFlowEntryNonDiverging {
        RefFlowEntryNonDiverging::Flow { subtype: 0, param1: 0, param2: rupees as u16, next: Flowref::end(), param3: 8 }
    }

    pub fn give_item(item: u16) -> RefFlowEntryNonDiverging {
        RefFlowEntryNonDiverging::Flow { subtype: 0, param1: 0, param2: item, next: Flowref::end(), param3: 9 }
    }

    pub fn give_gratitude_crystals() -> RefFlowEntryNonDiverging {
        RefFlowEntryNonDiverging::Flow { subtype: 0, param1: 0, param2: 0, next: Flowref::end(), param3: 43 }
    }
}

pub struct EventPatcher {
    flow: Msbf,
    text: Msbt,
    ref_flows: Vec<RefFlowEntry>,
    // store here when adding new flow entries
    index_names: HashMap<Cow<'static, str>, i16>,
}

pub struct Flow;
pub struct Flowend;

impl From<Flow> for Flowend {
    fn from(value: Flow) -> Self {
        Flowend
    }
}

impl Flow {
    pub fn with_label(self, label: impl Into<Cow<'static, str>>) -> Flow {
        self
    }

    pub fn give_item(self, item: u16) -> Flow {
        Flow
    }

    pub fn check_itemflag(self, itemflag: u16, count: u16) -> Flow {
        Flow
    }

    pub fn speak_text(self, text: &str) -> Flow {
        Flow
    }

    pub fn trigger_exit(self, roomid: u16, exit: u16) -> Flow {
        Flow
    }

    pub fn set_storyflag(self, storyflag: u16) -> Flow {
        Flow
    }

    pub fn check_storyflag(self, flag: u16, setflows: &[Flow], notsetflows: &[Flow]) -> Flowend {
        Flowend
    }

    pub fn check_answer2(self, flows1: &[Flow], flows2: &[Flow]) -> Flowend {
        Flowend
    }

    pub fn jump(self, dest: impl Into<Flowref>) -> Flowend {
        Flowend
    }
}

// pub fn test() {
//     create_entrypoint("117_99", &[
//         speak_text("hi, this is a non functional test"),
//         give_item(0),
//         speak_text("choice:\n[1] yes\nor no[2-]").with_label("choice"),
//         check_answer2(&[
//             speak_text("cool, you chose yes, don't"),
//             set_storyflag(899),
//             jump("choice"),
//         ], &[
//             check_storyflag(899, &[
//                 trigger_exit(0, 0)
//             ], &[
//                 trigger_exit(0, 1)
//             ])
//         ])
//     ])
// }

impl EventPatcher {
    pub fn new(mut flow: Msbf, text: Msbt) -> Self {
        // first, take flows out of the Msbf and make their "next" pointers also able
        // to reference names (same with lines of text)
        // will be put back after all is done
        let mut flows = vec![];
        swap(&mut flow.flows, &mut flows);
        let ref_flows: Vec<RefFlowEntry> = flows.into_iter().map(Into::into).collect();
        Self {
            flow,
            text,
            ref_flows,
            index_names: Default::default(),
        }
    }
    pub fn get_flow(
        &mut self,
        index: impl Into<Flowref>,
    ) -> Result<&mut RefFlowEntry, FlowPatchError> {
        let i16index: i16 = match index.into() {
            Flowref::Index(index) => index,
            Flowref::Name(name) => *self
                .index_names
                .get(&name)
                .ok_or(FlowPatchError::NameNotFound(name))?,
        };
        let uindex: usize = i16index
            .try_into()
            .map_err(|_| FlowPatchError::OutOfRange(i16index))?;
        self.ref_flows
            .get_mut(uindex)
            .ok_or(FlowPatchError::OutOfRange(i16index))
    }
    pub fn set_next(
        &mut self,
        index: impl Into<Flowref>,
        new_next: impl Into<Flowref>,
    ) -> Result<(), FlowPatchError> {
        match self.get_flow(index)? {
            RefFlowEntry::NonDiverging(entry) => {
                *entry.get_next_mut() = new_next.into();
                Ok(())
            }
            _ => Err(FlowPatchError::UnexpectedSwitch),
        }
    }
    pub fn set_branches(
        &mut self,
        index: impl Into<Flowref>,
        new_branches: Vec<Flowref>,
    ) -> Result<(), FlowPatchError> {
        match self.get_flow(index)? {
            RefFlowEntry::Diverging(RefFlowEntryDiverging { branches, .. }) => {
                *branches = new_branches;
                Ok(())
            }
            _ => Err(FlowPatchError::ExpectedSwitch),
        }
    }
    pub fn add_show_text(&mut self, text: &str, next_flow: impl Into<Flowref>) -> i16 {
        let text_line = self.text.text.len() as u16;
        let flow_index = self.ref_flows.len();

        self.text.text.push(TextSegment {
            atr: vec![0, 0],
            text: text.encode_utf16().collect(),
        });
        self.text
            .lbl
            .insert(format!("{text_line}"), text_line.into());
        self.ref_flows.push(RefFlowEntryNonDiverging::Text {
            file: 0,
            line: text_line,
            next: next_flow.into(),
        }.into());

        return flow_index as i16;
    }
    pub fn add_text_with_label(&mut self, text: &str, label: impl Into<String>) -> u16 {
        let label = label.into();
        let text_line = self.text.text.len() as u16;

        self.text.text.push(TextSegment {
            atr: vec![0, 0],
            text: text.encode_utf16().collect(),
        });
        self.text.lbl.insert(label, text_line.into());

        return text_line;
    }
    pub fn add_flow(&mut self, flow: RefFlowEntry, label: Option<impl Into<Cow<'static, str>>>) -> i16 {
        let next_index: i16 = self.ref_flows.len().try_into().unwrap();
        if let Some(label) = label {
            self.index_names.insert(label.into(), next_index);
        }
        self.ref_flows.push(flow);
        next_index
    }
}
