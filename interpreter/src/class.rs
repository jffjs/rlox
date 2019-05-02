use snowflake::ProcessUniqueId;
use std::fmt;

#[derive(Clone, Debug)]
pub struct LoxClass {
    pub name: String,
    pub id: ProcessUniqueId,
}

impl LoxClass {
    pub fn new(name: String) -> LoxClass {
        LoxClass {
            name,
            id: ProcessUniqueId::new(),
        }
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}

impl PartialEq for LoxClass {
    fn eq(&self, other: &LoxClass) -> bool {
        self.id == other.id
    }
}
