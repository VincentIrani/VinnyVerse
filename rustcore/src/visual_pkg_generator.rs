// This file houses the function used to generate world packages given a center point, power level, and a few other key parameters
use serde::Serialize;
use std::f32::consts::PI;
use bincode;

use crate::cell_def;
use crate::WorldData;
use cell_def::{Cell, CellKind};

#[derive(Serialize, Clone)]
pub struct Square {
    pub x: usize,
    pub y: usize,
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

pub fn generate_visual_pkg(world_data: &WorldData, soul_id: &String, x: &i32, y: &i32, radius: i32, direction: char, angle_deg: f32) -> Vec<u8> {

    let mut visual_pkg = Vec::new();
    let points = circle_slice((x, y), radius, direction, angle_deg);

    for point in points {
        let (local_x, local_y) = world_data.global_to_local(soul_id, point.x, point.y);
        let square = Square {
            x: local_x as usize,
            y: local_y as usize,
            content: SquareKind::WorldCell(0),
        };

        visual_pkg.push(square);
    }

    let bin_visual_pkg: Vec<u8> = bincode::serialize(&visual_pkg)
        .expect("Failed to serialize visual_pkg");
    
    bin_visual_pkg
}

pub fn circle_slice(
    center: (&i32, &i32),
    radius: i32,
    direction: char,
    angle_deg: f32,
) -> Vec<Point> {
    let (cx, cy) = center;
    let mut points = Vec::new();
    let full_circle = angle_deg >= 360.0;

    // Convert direction to radians for centerline
    let dir_rad = match direction {
        'N' | 'n' => PI / 2.0,
        'S' | 's' => -PI / 2.0,
        'E' | 'e' => 0.0,
        'W' | 'w' => PI,
        _ => 0.0,
    };

    let half_angle_rad = angle_deg.to_radians() / 2.0;

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