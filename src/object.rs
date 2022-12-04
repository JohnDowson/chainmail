use internment::Intern;
use std::rc::Rc;

pub struct Class {
    fields: Fields,
    responders: Responders,
}

pub struct ClassId(Intern<String>);

struct FieldId(Intern<String>);
pub struct Fields(Vec<(FieldId, Object)>);
pub struct Responders(Vec<(ClassId, Object)>);

pub enum Value {
    Object(Rc<Object>),
    I64(i64),
    F64(f64),
    Bool(bool),
    Null,
}

pub struct Object {
    fields: Fields,
    class_id: ClassId,
}
