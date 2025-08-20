// This file houses the function used to generate world packages given a center point, power level, and a few other key parameters
use serde::Serialize;
use std::f32::consts::PI;
use bincode;

use crate::cell_def;
use crate::WorldData;
use cell_def::{Cell, CellKind};

#[derive(Serialize, Clone)]
pub struct Square {
    pub x: i32,
    pub y: i32,
    pub content: SquareKind,
}

#[derive(Serialize, Clone)]
pub enum SquareKind {
    CritterCell(Cell),
    WorldCell(u8),
}

#[derive(Debug, Serialize, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub fn generate_visual_pkg(world_data: &WorldData, soul_id: &String, x: &i32, y: &i32, radius: i32, direction: &String, angle_deg: &i16) -> Vec<u8> {

    let mut visual_pkg = Vec::new();
    let points = circle_slice((x, y), radius, direction, angle_deg);

    for point in points {
        //If the generated point is not in the world bounds dont generate a square pkg for it.
        if !world_data.is_in_bounds(point.x, point.y) {
            continue;
        }

        let (local_x, local_y) = world_data.global_to_local(soul_id, point.x, point.y);

        let mut square = Square {x: local_x, y: local_y, content: SquareKind::WorldCell(0)};
        if world_data.is_critter_at(point.x, point.y) {
            square.content = SquareKind::CritterCell(world_data.critter_layer[point.x as usize][point.y as usize].clone());
        } else {
            square.content = SquareKind::WorldCell(world_data.world[point.x as usize][point.y as usize].clone());
        }

        visual_pkg.push(square);
    }

    let json_visual_pkg: Vec<u8> = serde_json::to_vec(&visual_pkg)
        .expect("Failed to serialize visual_pkg");

    json_visual_pkg
}

pub fn circle_slice(
    center: (&i32, &i32),
    radius: i32,
    direction: &String,
    angle_deg: &i16,
) -> Vec<Point> {
    let (cx, cy) = center;
    let mut points = Vec::new();
    let full_circle = *angle_deg >= 360;

    // Convert direction to radians for centerline
    let dir_rad = match direction.as_str() {
        "N" | "n" => PI / 2.0,
        "S" | "s" => -PI / 2.0,
        "E" | "e" => 0.0,
        "W" | "w" => PI,
        _ => 0.0,
    };

    let half_angle_rad = (*angle_deg as f32).to_radians() / 2.0;

    for x in (cx - radius)..=(cx + radius) {
        for y in (cy - radius)..=(cy + radius) {
            let dx = (x - cx) as f32;
            let dy = (y - cy) as f32;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= radius as f32 {
                if full_circle {
                    points.push(Point { x, y });
                    continue;
                }

                let mut angle = dy.atan2(dx);
                let mut diff = angle - dir_rad;

                while diff > PI {
                    diff -= 2.0 * PI;
                }
                while diff < -PI {
                    diff += 2.0 * PI;
                }

                if diff.abs() <= half_angle_rad {
                    points.push(Point { x, y });
                }
            }
        }
    }

    points
}