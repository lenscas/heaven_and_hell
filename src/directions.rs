use quicksilver::{geom::Vector, lifecycle::Key};
use std::convert::TryFrom;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum Directions {
    Left,
    Up,
    Right,
    Down,
}

impl Directions {
    pub fn change_x(self) -> bool {
        match self {
            Self::Up | Self::Down => false,
            _ => true,
        }
    }
    pub fn change_y(self) -> bool {
        !self.change_x()
    }
}

impl From<Directions> for Vector {
    fn from(from: Directions) -> Vector {
        match from {
            Directions::Up => (0, -1).into(),
            Directions::Down => (0, 1).into(),
            Directions::Right => (1, 0).into(),
            Directions::Left => (-1, 0).into(),
        }
    }
}
impl TryFrom<Key> for Directions {
    type Error = ();
    fn try_from(k: Key) -> Result<Self, Self::Error> {
        match k {
            //Key::W => Ok(Directions::Up),
            Key::A => Ok(Directions::Left),
            //Key::S => Ok(Directions::Down),
            Key::D => Ok(Directions::Right),
            _ => Err(()),
        }
    }
}
