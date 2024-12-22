use std::any::Any;

pub trait CustomComponent {
    fn as_any(&self) -> &(dyn Any);
    fn as_any_mut(&mut self) -> &mut (dyn Any);

    fn id(&self) -> &str;
    fn static_id() -> &'static str
    where
        Self: Sized;
}
