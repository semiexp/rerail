use wasm_bindgen::prelude::*;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub use crate::geom::Coord;
use crate::geom::{Rect, compute_station_line_segment};

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Station {
    name: String,
    railways: Vec<Weak<RefCell<Railway>>>,
}

impl Station {
    pub fn new(name: String) -> Station {
        Station { name, railways: vec![] }
    }

    pub fn add_railway(&mut self, railway: Weak<RefCell<Railway>>) -> bool {
        for i in 0..self.railways.len() {
            if self.railways[i].ptr_eq(&railway) {
                return false;
            }
        }
        self.railways.push(railway);
        true
    }
}

struct RailwayPoint {
    coord: Coord,
    station: Option<Weak<RefCell<Station>>>,
}

pub struct Railway {
    name: String,
    color: Color,
    points: Vec<RailwayPoint>,
}

impl Railway {
    pub fn new(name: String, color: Color) -> Railway {
        Railway { name, color, points: vec![] }
    }

    pub fn add_point(&mut self, coord: Coord, station: Option<Weak<RefCell<Station>>>) {
        self.points.push(RailwayPoint { coord, station });
    }
}

#[wasm_bindgen]
pub struct RailwayMap {
    stations: Vec<Rc<RefCell<Station>>>,
    railways: Vec<Rc<RefCell<Railway>>>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct RenderingInfo {
    pub rail_colors: Vec<Color>,
    pub rail_width: Vec<i32>,
    pub rail_points_num: Box<[i32]>,
    pub rail_points_x: Box<[i32]>,
    pub rail_points_y: Box<[i32]>,
}

#[wasm_bindgen]
impl RailwayMap {
    pub fn new() -> RailwayMap {
        RailwayMap { stations: vec![], railways: vec![] }
    }

    pub(crate) fn add_station(&mut self, station: Rc<RefCell<Station>>) {
        self.stations.push(station);
    }

    pub(crate) fn add_railway(&mut self, railway: Rc<RefCell<Railway>>) {
        self.railways.push(railway);
    }

    pub fn load(data: &[u8]) -> RailwayMap {
        let mut data = data;
        crate::loader::load_legacy_railmap_file(&mut data).unwrap()
    }

    pub fn render(&self, left_x: i32, top_y: i32, view_height: i32, view_width: i32, zoom_level: i32) -> RenderingInfo {
        let right_x = left_x + view_width * zoom_level;
        let bottom_y = top_y + view_height * zoom_level;
        let viewport = Rect::new(top_y, bottom_y, left_x, right_x);

        let mut rail_colors = vec![];
        let mut rail_width = vec![];
        let mut rail_points_num = vec![];
        let mut rail_points_x = vec![];
        let mut rail_points_y = vec![];

        for railway in &self.railways {
            let railway = railway.borrow();

            let mut num = 0;
            for i in 1..railway.points.len() {
                if viewport.crosses_with_line_segment(railway.points[i - 1].coord, railway.points[i].coord) {
                    num += 2;

                    let c0 = railway.points[i - 1].coord;
                    let c1 = railway.points[i].coord;

                    rail_points_x.push((c0.x - left_x) / zoom_level);
                    rail_points_x.push((c1.x - left_x) / zoom_level);
                    rail_points_y.push((c0.y - top_y) / zoom_level);
                    rail_points_y.push((c1.y - top_y) / zoom_level);
                }
            }

            if num > 0 {
                rail_colors.push(railway.color);
                rail_width.push(1);
                rail_points_num.push(num);
            }
        }

        let mut station_points_x = vec![];
        let mut station_points_y = vec![];
        for railway in &self.railways {
            let railway = railway.borrow();

            for i in 0..railway.points.len() {
                if !railway.points[i].station.is_some() {
                    continue;
                }
                if !viewport.contains(railway.points[i].coord) {
                    continue;
                }

                let prev = if i == 0 { None } else { Some(railway.points[i - 1].coord) };
                let next = if i + 1 == railway.points.len() { None } else { Some(railway.points[i + 1].coord) };

                let (c0, c1) = compute_station_line_segment(prev, railway.points[i].coord, next, 200);

                station_points_x.push((c0.x - left_x) / zoom_level);
                station_points_x.push((c1.x - left_x) / zoom_level);
                station_points_y.push((c0.y - top_y) / zoom_level);
                station_points_y.push((c1.y - top_y) / zoom_level);
            }
        }

        if station_points_x.len() > 0 {
            rail_colors.push(Color { r: 148, g: 148, b: 148 });
            rail_width.push(4);
            rail_points_num.push(station_points_x.len() as i32);
            rail_points_x.extend(station_points_x);
            rail_points_y.extend(station_points_y);
        }

        RenderingInfo {
            rail_colors,
            rail_width,
            rail_points_num: rail_points_num.into_boxed_slice(),
            rail_points_x: rail_points_x.into_boxed_slice(),
            rail_points_y: rail_points_y.into_boxed_slice(),
        }
    }
}
