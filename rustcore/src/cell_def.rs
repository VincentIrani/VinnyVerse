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

impl CellKind {
    pub fn from_input_string(input_string: &str) -> Option<Self> {
        match input_string {
            "Empty" => Some(CellKind::Empty),
            "Soul" => Some(CellKind::Soul),
            "Tissue" => Some(CellKind::Tissue),
            "Eyeball" => Some(CellKind::Eyeball),
            "Mouth" => Some(CellKind::Mouth),
            "Butt" => Some(CellKind::Butt),
            "Muscle" => Some(CellKind::Muscle),
            "Anchor" => Some(CellKind::Anchor),
            "Armor" => Some(CellKind::Armor),
            _ => None,
        }
    }
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

        pub fn valid_dir(direction: &str) -> bool {
            match direction {
                "N" | "S" | "E" | "W" | "C" => true,
                _ => false,
            }
        }
    }
