#[multiversx_sc::proxy]
pub trait TemplateProxy {
    #[init]
    fn init(&self) {}
}
