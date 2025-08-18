use rand::Rng;

use crate::cell_def;

use cell_def::{Cell, CellKind};
use std::io::{self, Write};


use crate::UserInput;
use crate::WorldData;
use crate::BalancingParams;


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
                CellKind::Soul => 'S',
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

pub fn generate_souls(world_data: &mut WorldData, soul_que: & Vec<UserInput>, starting_energy: i16){
    for input in soul_que.iter() {
        
        let mut soul_id_to_find = String::new();

        if let UserInput::GenerateSoul { soul_id } = input {
            soul_id_to_find = soul_id.clone();
        } else {
            // input was something else, handle or ignore
        }

        if let Some((soul, x, y)) = world_data.soul_locations.iter().find(|(soul, _, _)| *soul == soul_id_to_find) {
            println!("Soul {} already exists!", soul);
            continue; // Skip if soul already exists
        }

        let mut x_spawn = (rand::thread_rng().gen_range(0..world_data.world.len())) as usize;
        let mut y_spawn = (rand::thread_rng().gen_range(0..world_data.world[0].len())) as usize;

        let mut i = 0; // Counter to prevent infinite loop
        while !is_empty_cell(&world_data, x_spawn.try_into().unwrap(), y_spawn.try_into().unwrap(), 3) && i < 100 {
            // If the cell is not empty, find a new random position
            x_spawn = rand::thread_rng().gen_range(0..world_data.world.len());
            y_spawn = rand::thread_rng().gen_range(0..world_data.world[0].len());
            i += 1; // Increment counter
        }

        if i >= 100 {
            println!("Could not find an empty cell for soul {} after 100 attempts", soul_id_to_find);
            continue; // Skip if no empty cell found after 100 attempts
        }

        // Create a new soul cell
        let new_soul_cell = Cell::new(soul_id_to_find.clone(), CellKind::Soul, starting_energy, "C".to_string());
        world_data.critter_layer[y_spawn][x_spawn] = new_soul_cell; // Place the soul in the critter layer
        world_data.soul_locations.push((soul_id_to_find.clone(), x_spawn.try_into().unwrap(), y_spawn.try_into().unwrap())); // Add to soul locations
        println!("Generated soul {} at ({}, {})", soul_id_to_find, x_spawn, y_spawn);
    }
    
}

pub fn is_empty_cell(world_data: &WorldData, x: usize, y: usize, r:usize) -> bool {
    // Check if the coordinates are within bounds

    println!("Checking if cell at ({}, {}) is empty within radius {}", x, y, r);

    if x > world_data.world.len() || y > world_data.world[0].len() {
        return false;
    } else if x <= 0 || y <= 0 {
        return false; 
    }

    let rows = world_data.world.len();
    if rows == 0 {
        return false; // Empty world
    }

    let cols = world_data.world[0].len();

    // Calculate the boundaries for the radius search, clamp to world size
    let start_x = x.saturating_sub(r); // avoid underflow
    let end_x = (x + r).min(rows - 1);
    let start_y = y.saturating_sub(r);
    let end_y = (y + r).min(cols - 1);

    for i in start_x..=end_x {
        for j in start_y..=end_y {
            if !world_data.critter_layer[i][j].is_empty() {
                return false; // Found a non-empty cell
            }
        }
    }

    true // All cells in radius are empty
}

pub fn do_actions(world_data: &mut WorldData, action_que: & Vec<UserInput>, balancing_params: &BalancingParams){
    for action in action_que{
        let UserInput::Activate { soul_id, delay, X, Y, power } = action else {
          continue;
        };

        //Checks befor activating cell
        if world_data.critter_layer[*Y as usize][*X as usize].is_empty() {
            println!("Cell at ({}, {}) is empty", X, Y); //eventually this should be returned to user!!
            continue;
        } else if world_data.critter_layer[*Y as usize][*X as usize].id != *soul_id {
            println!("Cell at ({}, {}) is not owned by you!", X, Y);
            continue;
        }

        match world_data.critter_layer[*Y as usize][*X as usize].kind {
            CellKind::Soul => {
                println!("Cell at ({}, {}) is a soul, not a valid target", X, Y);
            },
            CellKind::Tissue => {
                println!("Cell at ({}, {}) is a tissue, not a valid target", X, Y);
            },
            CellKind::Eyeball => {
                println!("Cell at ({}, {}) is an eyeball", X, Y);
            },
            CellKind::Mouth => {
                println!("Cell at ({}, {}) is a mouth", X, Y);
            },
            CellKind::Butt => {
                println!("Cell at ({}, {}) is a butt", X, Y);
            },
            CellKind::Muscle => {
                println!("Cell at ({}, {}) is a muscle", X, Y);
            },
            CellKind::Armor => {
                println!("Cell at ({}, {}) is an armor, not a valid target", X, Y);
            },
            CellKind::Anchor => {
                println!("Cell at ({}, {}) is an anchor", X, Y);
            },
            _ => {
                println!("Cell at ({}, {}) is not a valid target", X, Y);
            }
        }
    }
}
