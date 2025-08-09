use rand::Rng;

use crate::cell_def;

use cell_def::{Cell, CellKind};
use std::io::{self, Write};


use crate::UserInput;


pub fn generate_world(size: usize) -> Vec<Vec<u8>> {
    let mut rng = rand::thread_rng();

    // Step 1: Initialize world with random values 0..=255
    let mut world: Vec<Vec<u8>> = (0..size)
        .map(|_| (0..size).map(|_| rng.gen_range(0..=255)).collect())
        .collect();

    // Step 2: Smooth the world multiple times to reduce sharp jumps
    let smoothing_passes = 2;
    for _ in 0..smoothing_passes {
        world = smooth(&world, size);
    }

    world
}

// Helper function: smooth by averaging neighbors (3x3 kernel)
fn smooth(world: &Vec<Vec<u8>>, size: usize) -> Vec<Vec<u8>> {
    let mut new_world = vec![vec![0u8; size]; size];

    for i in 0..size {
        for j in 0..size {
            let mut sum = 0usize;
            let mut count = 0usize;

            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let y = i as i32 + dy;
                    let x = j as i32 + dx;

                    if x >= 0 && x < size as i32 && y >= 0 && y < size as i32 {
                        sum += world[y as usize][x as usize] as usize;
                        count += 1;
                    }
                }
            }

            new_world[i][j] = (sum / count) as u8;
        }
    }

    new_world
}

pub fn visualize_world_console(world: &Vec<Vec<u8>>) {
    for row in world {
        for &val in row {
            let c = match val {
                0..=51 => ' ',
                52..=102 => '░',
                103..=153 => '▒',
                154..=204 => '▓',
                _ => '█',
            };
            print!("{}", c);
        }
        println!();
    }
}

pub fn the_sun(world: &mut Vec<Vec<u8>>) {

    for i in 0..world.len() {
        for j in 0..world[i].len() {
            if world[i][j] < 255 {
                world[i][j] += 1;
            }
        }
    }
}

pub fn build_critters(critter_layer: &mut Vec<Vec<Cell>>, build_que: &mut Vec<UserInput>) {
    for input in build_que.iter() {
        if let UserInput::Build {
            soul_id,
            block_type,
            X,
            Y,
            dir,
            power,
        } = input
        {
            // Bounds check
            if *Y as usize >= critter_layer.len() || *X as usize >= critter_layer[0].len() {
                println!("Build request out of bounds: ({}, {})", X, Y);
                continue;
            }

            if critter_layer[*Y as usize][*X as usize].kind != CellKind::Empty {
                println!("Cell at ({}, {}) is not empty, cannot build", X, Y);
                continue;
            }

            let mut can_build = false;
            let size = critter_layer.len();
            let x = *X as isize;
            let y = *Y as isize;
            let directions = [(0, 1), (1, 0), (0, -1), (-1, 0), (1, 1), (1, -1), (-1, 1), (-1, -1)];

            for (dx, dy) in directions.iter() {
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < size as isize && ny >= 0 && ny < size as isize {
                    let neighbor = &critter_layer[ny as usize][nx as usize];
                    if neighbor.kind == CellKind::Tissue
                        || (neighbor.kind == CellKind::Soul && neighbor.id == *soul_id)
                    {
                        can_build = true;
                        break;
                    }
                }
            }

            if !can_build {
                println!(
                    "Cannot build at ({}, {}): no adjacent Tissue or matching Soul cell",
                    X, Y
                );
                continue;
            }

            // Match block_type to CellKind
            let cell_kind = match block_type.as_str() {
                "Tissue" => CellKind::Tissue,
                "Eyeball" => CellKind::Eyeball,
                "Mouth" => CellKind::Mouth,
                "Butt" => CellKind::Butt,
                "Muscle" => CellKind::Muscle,
                "Anchor" => CellKind::Anchor,
                "Armor" => CellKind::Armor,
                _ => {
                    println!("Unknown block type: {}", block_type);
                    continue;
                }
            };

            let direction = match dir.as_str() {
                "N" => "N".to_string(),
                "S" => "S".to_string(),
                "E" => "E".to_string(),
                "W" => "W".to_string(),
                "C" => "C".to_string(),
                _ => {
                    println!("Unknown direction: {}", dir);
                    continue;
                }
            };

            // Place the cell
            critter_layer[*Y as usize][*X as usize] =
                Cell::new(soul_id.clone(), cell_kind, *power, direction.clone());
        }
    }
}

pub fn visualize_critter_layer(critter_layer: &Vec<Vec<Cell>>) {
    for row in critter_layer {
        for cell in row {
            let c = match cell.kind {
                CellKind::Tissue => 'T',
                CellKind::Eyeball => 'O',
                CellKind::Mouth => 'M',
                CellKind::Butt => 'B',
                CellKind::Muscle => 'U',
                CellKind::Anchor => 'A',
                CellKind::Armor => '#',
                CellKind::Empty => '.', // For empty cells
                _ => '.', // For empty or unknown cells
            };
            print!("{}", c);
        }
        println!();
    }
}

pub fn activate_soul(){

}

pub fn mount_soul(){

}