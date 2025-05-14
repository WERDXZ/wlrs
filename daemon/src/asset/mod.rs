pub mod animated;
pub mod damage;
pub mod image;

pub trait Asset {
    fn damage(&self) -> damage::Damage;
}
