use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
  pub enum CellKind {
        Empty,
        Soul,
        Tissue,
        Eyeball,
        Mouth,
        Butt,
        Muscle,
        Anchor,
        Armor,
    }

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
    pub struct Cell {
        pub id: String,
        pub kind: CellKind,
        pub energy: i16,
        pub orientation: String,
    }

    impl Cell {
        pub fn empty() -> Self {
            Self {
                id: "Null".to_string(),
                kind: CellKind::Empty,
                energy: 0,
                orientation: "C".to_string(),
            }
        }

        pub fn new(id: String, kind: CellKind, energy: i16, orientation: String) -> Self {
            Self {
                id,
                kind,
                energy,
                orientation,
            }
        }

        pub fn is_empty(&self) -> bool {
            self.kind == CellKind::Empty
        }
    }