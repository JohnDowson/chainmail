use fnv::FnvHashMap;
pub struct InternedString(usize);
#[derive(Default)]
pub struct Object(FnvHashMap<InternedString, Object>);
pub struct Evaluator {}

impl Evaluator {
    fn _eval(&mut self) -> Object {
        Object::default()
    }
}
