pub enum OpenThunderhead {
    Open,
    Ballad,
}

pub enum StartingSword {
    Swordless,
    PracticeSword,
    GoddessSword,
    GoddessLongsword,
    GoddessWhiteSword,
    MasterSword,
    TrueMasterSword,
}

struct BitsReader;
struct BitsWriter;

trait BitsReaderRW: Sized {
    fn from_bits(reader: &mut BitsReader) -> Option<Self>;
    fn to_bits(reader: &mut BitsReader);
}

pub struct LogicOptions {
    /// description here
    pub start_tablet_count: u8,
    pub open_thunderhead: OpenThunderhead,
    pub starting_sword: StartingSword,
    pub required_dungeon_count: u8,
    pub skip_imp2: bool,
    pub empty_unrequired_dungeons: bool,
    pub triforce_required: bool,
}
