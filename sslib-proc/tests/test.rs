use bzs::{SetByName, Datatype, DatatypeSetable, ContextSetError, DatatypeSetError};
use sslib_proc::SetByName;

struct H;

impl DatatypeSetable for H {
    fn set(&mut self, data: &Datatype<'_>) -> Result<(), DatatypeSetError> {
        Ok(())
    }
}

#[derive(SetByName)]
struct Test {
    cool: u32,
    asdf: u32,
    h: H,
}
