use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};
use tsify::Tsify;

pub use crate::geom::Coord;
use crate::geom::{
    compute_station_line_segment, distance_norm_square_point_line_segment,
    distance_norm_square_points, Rect,
};
use crate::sparse_array::{SparseArray, SparseArrayId};

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Station {
    name: String,
    level: u8,
    railways: Vec<RailwayIndex>,
}

impl Station {
    pub fn new(name: String, level: u8) -> Station {
        Station {
            name,
            level,
            railways: vec![],
        }
    }

    pub fn add_railway(&mut self, railway: RailwayIndex) -> bool {
        for i in 0..self.railways.len() {
            if self.railways[i] == railway {
                return false;
            }
        }
        self.railways.push(railway);
        true
    }

    pub fn remove_railway(&mut self, railway: RailwayIndex) -> bool {
        for i in 0..self.railways.len() {
            if self.railways[i] == railway {
                self.railways.remove(i);
                return true;
            }
        }
        false
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct RailwayPoint {
    coord: Coord,
    station: Option<StationIndex>,
}

#[derive(Serialize, Deserialize)]
pub struct Railway {
    name: String,
    color: Color,
    level: u8,
    points: Vec<RailwayPoint>,
}

impl Railway {
    pub fn add_point(&mut self, coord: Coord, station: Option<StationIndex>) {
        self.points.push(RailwayPoint { coord, station });
    }
}

#[derive(Serialize, Deserialize)]
pub struct BorderPoint {
    coord: Coord,
    neighbors: Vec<(BorderPointIndex, u8)>,
}

impl BorderPoint {
    pub fn new(coord: Coord) -> BorderPoint {
        BorderPoint {
            coord,
            neighbors: vec![],
        }
    }

    pub fn add_neighbor(&mut self, neighbor: BorderPointIndex, level: u8) {
        self.neighbors.push((neighbor, level));
    }

    pub fn has_neighbor(&mut self, neighbor: BorderPointIndex) -> bool {
        self.get_level(neighbor).is_some()
    }

    pub fn update_level(&mut self, neighbor: BorderPointIndex, level: u8) {
        for i in 0..self.neighbors.len() {
            if self.neighbors[i].0 == neighbor {
                self.neighbors[i].1 = level;
            }
        }
    }

    pub fn get_level(&self, neighbor: BorderPointIndex) -> Option<u8> {
        for &(j, level) in &self.neighbors {
            if neighbor == j {
                return Some(level);
            }
        }
        None
    }

    pub fn remove_neighbor(&mut self, neighbor: BorderPointIndex) {
        for i in 0..self.neighbors.len() {
            if self.neighbors[i].0 == neighbor {
                self.neighbors.remove(i);
                return;
            }
        }
    }
}

pub type StationIndex = SparseArrayId<Station>;
pub type RailwayIndex = SparseArrayId<Railway>;
pub type BorderPointIndex = SparseArrayId<BorderPoint>;

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct RerailMap {
    stations: SparseArray<Station>,
    railways: SparseArray<Railway>,
    border_points: SparseArray<BorderPoint>,
    railway_unique_id_last: usize,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct StationRenderingInfo {
    pub name: String,
    pub x: i32,
    pub y: i32,
}

#[wasm_bindgen(getter_with_clone)]
pub struct RenderingInfo {
    pub rail_colors: Vec<Color>,
    pub rail_width: Vec<i32>,
    pub rail_style: Vec<i32>,
    pub rail_points_num: Box<[i32]>,
    pub rail_points_x: Box<[i32]>,
    pub rail_points_y: Box<[i32]>,
    pub marker_points_x: Vec<i32>,
    pub marker_points_y: Vec<i32>,
    pub stations: Vec<StationRenderingInfo>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct ViewportRailwayList {
    pub rail_names: Vec<String>,
    pub rail_ids: Vec<usize>,
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ViewportSpec {
    #[serde(rename = "leftX")]
    left_x: i32,
    #[serde(rename = "topY")]
    top_y: i32,
    width: i32,
    height: i32,
    zoom: i32,
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TemporaryMovingPoint {
    index: IndexOnRailway,
    #[serde(rename = "pointAfterMove")]
    point_after_move: PhysicalCoord,
}

#[derive(Clone, Copy, Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TemporaryMovingBorderPoint {
    #[serde(rename = "pointOrSegment")]
    point_or_segment: BorderPointOrSegment,
    #[serde(rename = "pointAfterMove")]
    point_after_move: PhysicalCoord,
}

#[derive(Clone, Copy, Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ExtraBorderSegment {
    point: BorderPointIndex,
    #[serde(rename = "newPoint")]
    new_point: PhysicalCoord,
    level: u8,
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct RenderingOptions {
    #[tsify(optional)]
    #[serde(rename = "selectedRailId")]
    selected_rail_id: Option<usize>,
    #[tsify(optional)]
    #[serde(rename = "temporaryMovingPoint")]
    temporary_moving_point: Option<TemporaryMovingPoint>,
    #[tsify(optional)]
    #[serde(rename = "markerOnBorderPoints", default)]
    marker_on_border_points: bool,
    #[tsify(optional)]
    #[serde(rename = "temporaryMovingBorderPoint")]
    temporary_moving_border_point: Option<TemporaryMovingBorderPoint>,
    #[tsify(optional)]
    #[serde(rename = "extraBorderSegment")]
    extra_border_segment: Option<ExtraBorderSegment>,
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct StationInfo {
    name: String,
    level: u8,
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct RailwayInfo {
    name: String,
    level: u8,
    color: u32,
}

#[derive(Tsify, Clone, Copy, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PhysicalCoord {
    x: i32,
    y: i32,
}

impl PhysicalCoord {
    fn as_coord(&self) -> Coord {
        Coord {
            x: self.x,
            y: self.y,
        }
    }
}

fn split_into_x_and_y(points: &[PhysicalCoord]) -> (Vec<i32>, Vec<i32>) {
    let mut xs = vec![];
    let mut ys = vec![];
    for pt in points {
        xs.push(pt.x);
        ys.push(pt.y);
    }
    (xs, ys)
}

struct Viewport {
    left_x: i32,
    top_y: i32,
    zoom: i32,
    bounding_box: Rect,
}

impl Viewport {
    fn new(spec: ViewportSpec) -> Viewport {
        let bottom = spec.top_y + spec.height * spec.zoom;
        let right = spec.left_x + spec.width * spec.zoom;

        Viewport {
            left_x: spec.left_x,
            top_y: spec.top_y,
            zoom: spec.zoom,
            bounding_box: Rect::new(spec.top_y, bottom, spec.left_x, right),
        }
    }

    fn contains(&self, coord: Coord) -> bool {
        self.bounding_box.contains(coord)
    }

    fn crosses_with_line_segment(&self, a: Coord, b: Coord) -> bool {
        self.bounding_box.crosses_with_line_segment(a, b)
    }

    fn to_physical_point(&self, coord: Coord) -> PhysicalCoord {
        PhysicalCoord {
            x: (coord.x - self.left_x) / self.zoom,
            y: (coord.y - self.top_y) / self.zoom,
        }
    }

    fn from_physical_point(&self, coord: PhysicalCoord) -> Coord {
        Coord {
            x: coord.x * self.zoom + self.left_x,
            y: coord.y * self.zoom + self.top_y,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct IndexOnRailway {
    index: usize,
    inserting: bool,
}

#[derive(Clone, Copy, Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum BorderPointOrSegment {
    #[serde(rename = "point")]
    Point(BorderPointIndex),
    #[serde(rename = "segment")]
    Segment(BorderPointIndex, BorderPointIndex),
}

#[wasm_bindgen(getter_with_clone)]
pub struct StationListOnRailway {
    pub names: Vec<String>,
    pub distances: Vec<f64>,
}

const STATION_THRESHOLD: [[i32; 4]; 4] = [
    [20, 50, 50, 50],
    [50, 100, 200, 200],
    [50, 100, 200, 200],
    [100, 200, 500, 5000],
];

const RAILWAY_THRESHOLD: [i32; 4] = [100, 200, 200, 10000];

#[wasm_bindgen]
pub struct RerailMapAndRailwayIndex {
    map: Option<RerailMap>,
    index: RailwayIndex,
}

#[wasm_bindgen]
impl RerailMapAndRailwayIndex {
    #[wasm_bindgen(js_name = getMap)]
    pub fn get_map(&mut self) -> RerailMap {
        self.map.take().unwrap()
    }

    #[wasm_bindgen(js_name = getRailwayIndex)]
    pub fn get_railway_index(&self) -> RailwayIndex {
        self.index
    }
}

#[wasm_bindgen]
impl RerailMap {
    pub fn new() -> RerailMap {
        RerailMap {
            stations: SparseArray::new(),
            railways: SparseArray::new(),
            border_points: SparseArray::new(),
            railway_unique_id_last: 0,
        }
    }

    pub(crate) fn add_station(&mut self, station: Station) -> StationIndex {
        self.stations.push(station)
    }

    pub(crate) fn new_railway(&mut self, name: String, color: Color, level: u8) -> RailwayIndex {
        let railway = Railway {
            name,
            color,
            level,
            points: vec![],
        };
        self.railways.push(railway)
    }

    pub(crate) fn add_border_point(&mut self, border_point: BorderPoint) -> BorderPointIndex {
        self.border_points.push(border_point)
    }

    pub fn load(data: &[u8]) -> RerailMap {
        let mut data = data;
        assert!(data.len() >= 2);

        if data[0] == 'R' as u8 && data[1] == 'M' as u8 {
            crate::loader::load_legacy_railmap_file(&mut data).unwrap()
        } else if data[0] == 'R' as u8 && data[1] == 'L' as u8 {
            RerailMap::load_new_format(data)
        } else {
            panic!();
        }
    }

    fn load_new_format(data: &[u8]) -> RerailMap {
        let r = flexbuffers::Reader::get_root(data).unwrap();
        RerailMap::deserialize(r).unwrap()
    }

    pub fn save(&self) -> Box<[u8]> {
        let mut serializer = flexbuffers::FlexbufferSerializer::new();
        self.serialize(&mut serializer).unwrap();

        let mut ret = vec!['R' as u8, 'L' as u8];
        ret.extend(serializer.view());
        ret.into_boxed_slice()
    }

    #[wasm_bindgen(js_name = insertRailwayPoint)]
    pub fn insert_railway_point(
        mut self,
        railway_id: RailwayIndex,
        i: usize,
        x: i32,
        y: i32,
    ) -> RerailMap {
        let railway = &mut self.railways[railway_id];
        railway.points.insert(
            i,
            RailwayPoint {
                coord: Coord::new(x, y),
                station: None,
            },
        );
        self
    }

    #[wasm_bindgen(js_name = moveRailwayPoint)]
    pub fn move_railway_point(
        mut self,
        railway_id: RailwayIndex,
        i: usize,
        x: i32,
        y: i32,
    ) -> RerailMap {
        let railway = &mut self.railways[railway_id];
        railway.points[i].coord = Coord::new(x, y);
        self
    }

    #[wasm_bindgen(js_name = moveBorderPoint)]
    pub fn move_border_point(mut self, id: BorderPointIndex, x: i32, y: i32) -> RerailMap {
        self.border_points[id].coord = Coord::new(x, y);
        self
    }

    #[wasm_bindgen(js_name = insertBorderPointBetweenSegment)]
    pub fn insert_border_point_between_segment(
        mut self,
        i: BorderPointIndex,
        j: BorderPointIndex,
        x: i32,
        y: i32,
    ) -> RerailMap {
        let level = self.border_points[i].get_level(j).unwrap();
        self.border_points[i].remove_neighbor(j);
        self.border_points[j].remove_neighbor(i);

        let k = self.border_points.push(BorderPoint::new(Coord { x, y }));
        self.border_points[i].add_neighbor(k, level);
        self.border_points[k].add_neighbor(i, level);
        self.border_points[j].add_neighbor(k, level);
        self.border_points[k].add_neighbor(j, level);
        self
    }

    #[wasm_bindgen(js_name = connectToNewBorderPoint)]
    pub fn connect_to_new_border_point(
        mut self,
        i: BorderPointIndex,
        x: i32,
        y: i32,
        level: u8,
    ) -> RerailMap {
        let j = self.border_points.push(BorderPoint::new(Coord { x, y }));
        self.border_points[i].add_neighbor(j, level);
        self.border_points[j].add_neighbor(i, level);
        self
    }

    #[wasm_bindgen(js_name = connectExistingBorderPoints)]
    pub fn connect_existing_border_points(
        mut self,
        i: BorderPointIndex,
        j: BorderPointIndex,
        level: u8,
    ) -> RerailMap {
        if i == j {
            return self;
        }
        if self.border_points[i].has_neighbor(j) {
            self.border_points[i].update_level(j, level);
            self.border_points[j].update_level(i, level);
        } else {
            self.border_points[i].add_neighbor(j, level);
            self.border_points[j].add_neighbor(i, level);
        }
        self
    }

    #[wasm_bindgen(js_name = removeBorderPoint)]
    pub fn remove_border_point(mut self, i: BorderPointIndex) -> RerailMap {
        let n_adj = self.border_points[i].neighbors.len();
        if n_adj < 3 {
            if n_adj == 1 {
                let j = self.border_points[i].neighbors[0].0;
                self.border_points[i].remove_neighbor(j);
                self.border_points[j].remove_neighbor(i);
            } else if n_adj == 2 {
                let (j, jl) = self.border_points[i].neighbors[0];
                let (k, kl) = self.border_points[i].neighbors[1];
                self.border_points[i].remove_neighbor(j);
                self.border_points[j].remove_neighbor(i);
                self.border_points[i].remove_neighbor(k);
                self.border_points[k].remove_neighbor(i);

                if !self.border_points[j].has_neighbor(k) {
                    let level = jl.min(kl);
                    self.border_points[j].add_neighbor(k, level);
                    self.border_points[k].add_neighbor(j, level);
                }
            }
            self.border_points.delete(i);
        }

        self
    }

    #[wasm_bindgen(js_name = removeBorderEdge)]
    pub fn remove_border_edge(mut self, i: BorderPointIndex, j: BorderPointIndex) -> RerailMap {
        self.border_points[i].remove_neighbor(j);
        self.border_points[j].remove_neighbor(i);

        if self.border_points[i].neighbors.len() == 0 {
            self.border_points.delete(i);
        }
        if self.border_points[j].neighbors.len() == 0 {
            self.border_points.delete(j);
        }

        self
    }

    #[wasm_bindgen(js_name = removeRailwayPoint)]
    pub fn remove_railway_point(mut self, railway_id: RailwayIndex, i: usize) -> RerailMap {
        self = self.detach_station_on_railway(railway_id, i);
        let railway = &mut self.railways[railway_id];
        railway.points.remove(i);

        self
    }

    #[wasm_bindgen(js_name = detachStationOnRailway)]
    pub fn detach_station_on_railway(mut self, railway_id: RailwayIndex, i: usize) -> RerailMap {
        let railway = &mut self.railways[railway_id];
        if let Some(station_idx) = railway.points[i].station {
            railway.points[i].station = None;
            self[station_idx].remove_railway(railway_id);
            if self[station_idx].railways.is_empty() {
                self.stations.delete(station_idx);
            }
        }

        self
    }

    #[wasm_bindgen(js_name = linkToStation)]
    pub fn link_to_station(
        mut self,
        rail_id: RailwayIndex,
        index: usize,
        viewport: ViewportSpec,
        point: PhysicalCoord,
    ) -> RerailMap {
        if self.railways[rail_id].points[index].station.is_none() {
            let viewport = Viewport::new(viewport);
            let point = point.as_coord();

            let mut nearest_station = None;
            let mut nearst_distance_sq = 101; // TODO

            for (id, railway) in self.railways.enumerate() {
                if id == rail_id {
                    continue;
                }
                for j in 0..railway.points.len() {
                    if !railway.points[j].station.is_some() {
                        continue;
                    }

                    let dist_sq = distance_norm_square_points(
                        viewport
                            .to_physical_point(railway.points[j].coord)
                            .as_coord(),
                        point,
                    );
                    if dist_sq < nearst_distance_sq {
                        nearest_station =
                            Some((railway.points[j].coord, railway.points[j].station.unwrap()));
                        nearst_distance_sq = dist_sq;
                    }
                }
            }

            if let Some((coord, station_id)) = nearest_station {
                self.railways[rail_id].points[index] = RailwayPoint {
                    coord,
                    station: Some(station_id),
                };
                self[station_id].add_railway(rail_id);
            }
        }

        self
    }

    #[wasm_bindgen(js_name = railwaysInViewport)]
    pub fn railways_in_viewport(&self, viewport: ViewportSpec) -> ViewportRailwayList {
        let viewport = Viewport::new(viewport);

        let mut rail_names = vec![];
        let mut rail_ids = vec![];

        for (id, railway) in self.railways.enumerate() {
            if viewport.zoom > RAILWAY_THRESHOLD[railway.level as usize] {
                continue;
            }

            let mut is_displayed = false;
            for j in 1..railway.points.len() {
                if viewport
                    .crosses_with_line_segment(railway.points[j - 1].coord, railway.points[j].coord)
                {
                    is_displayed = true;
                    break;
                }
            }

            if is_displayed {
                rail_names.push(railway.name.clone());
                rail_ids.push(id.as_usize());
            }
        }

        ViewportRailwayList {
            rail_names,
            rail_ids,
        }
    }

    pub fn render(&self, viewport: ViewportSpec, opts: RenderingOptions) -> RenderingInfo {
        let viewport = Viewport::new(viewport);

        let mut rail_colors = vec![];
        let mut rail_width = vec![];
        let mut rail_style = vec![];
        let mut rail_points_num = vec![];
        let mut rail_points = vec![];
        let mut stations = vec![];

        let mut marker_points = vec![];

        let mut selected_railway_points = vec![];
        if let Some(id) = opts.selected_rail_id {
            let id = RailwayIndex::from_usize(id);
            if let Some(selected_railway) = self.railways.get(id) {
                selected_railway_points = selected_railway.points.clone();

                if let Some(temporary_moving_point) = &opts.temporary_moving_point {
                    let mouse_coord =
                        viewport.from_physical_point(temporary_moving_point.point_after_move);
                    if temporary_moving_point.index.inserting {
                        selected_railway_points.insert(
                            temporary_moving_point.index.index,
                            RailwayPoint {
                                coord: mouse_coord,
                                station: None,
                            },
                        );
                    } else {
                        selected_railway_points[temporary_moving_point.index.index].coord =
                            mouse_coord;
                    }
                }
            }
        }

        for pt in &selected_railway_points {
            let coord = pt.coord;
            if viewport.contains(coord) {
                marker_points.push(viewport.to_physical_point(coord))
            }
        }

        for (id, railway) in self.railways.enumerate() {
            if viewport.zoom > RAILWAY_THRESHOLD[railway.level as usize] {
                continue;
            }

            let railway_points = if Some(id.as_usize()) == opts.selected_rail_id {
                &selected_railway_points
            } else {
                &railway.points
            };

            let mut num = 0;
            for i in 1..railway_points.len() {
                if viewport
                    .crosses_with_line_segment(railway_points[i - 1].coord, railway_points[i].coord)
                {
                    num += 2;

                    rail_points.push(viewport.to_physical_point(railway_points[i - 1].coord));
                    rail_points.push(viewport.to_physical_point(railway_points[i].coord));
                }
            }

            if num > 0 {
                rail_colors.push(railway.color);
                rail_width.push(1);
                rail_style.push(0);
                rail_points_num.push(num);
            }
        }

        let mut station_points = vec![];
        let mut station_rendered = std::collections::BTreeSet::<StationIndex>::new();

        for (id, railway) in self.railways.enumerate() {
            let rail_level = railway.level as usize;
            if viewport.zoom > RAILWAY_THRESHOLD[rail_level] {
                continue;
            }
            let railway_points = if Some(id.as_usize()) == opts.selected_rail_id {
                &selected_railway_points
            } else {
                &railway.points
            };
            for i in 0..railway_points.len() {
                if let Some(station_idx) = railway_points[i].station {
                    if !railway_points[i].station.is_some() {
                        continue;
                    }
                    if !viewport.contains(railway_points[i].coord) {
                        continue;
                    }
                    let station_level = self[station_idx].level as usize;
                    if viewport.zoom > STATION_THRESHOLD[rail_level][station_level] {
                        continue;
                    }

                    let prev = if i == 0 {
                        None
                    } else {
                        Some(railway_points[i - 1].coord)
                    };
                    let cur = railway_points[i].coord;
                    let next = if i + 1 == railway_points.len() {
                        None
                    } else {
                        Some(railway_points[i + 1].coord)
                    };

                    let (c0, c1) = compute_station_line_segment(prev, cur, next, 200);

                    station_points.push(viewport.to_physical_point(c0));
                    station_points.push(viewport.to_physical_point(c1));

                    if station_rendered.contains(&station_idx) {
                        continue;
                    }
                    station_rendered.insert(station_idx);

                    let station = &self[station_idx];
                    let pt = viewport.to_physical_point(railway_points[i].coord);
                    stations.push(StationRenderingInfo {
                        name: station.name.clone(),
                        x: pt.x,
                        y: pt.y,
                    });
                }
            }
        }

        if station_points.len() > 0 {
            rail_colors.push(Color {
                r: 148,
                g: 148,
                b: 148,
            });
            rail_width.push(4);
            rail_style.push(0);
            rail_points_num.push(station_points.len() as i32);
            rail_points.extend(station_points);
        }

        let mut border_points = vec![vec![]; 3];

        let get_border_coord = |i: BorderPointIndex| {
            match opts.temporary_moving_border_point {
                None => (),
                Some(mv) => match mv.point_or_segment {
                    BorderPointOrSegment::Point(p) => {
                        if i == p {
                            return viewport.from_physical_point(mv.point_after_move);
                        }
                    }
                    _ => (),
                },
            }
            self.border_points[i].coord
        };

        let mut temporary_segment_level = None;
        for (i, pt) in self.border_points.enumerate() {
            if opts.marker_on_border_points {
                let coord = get_border_coord(i);
                if viewport.contains(coord) {
                    marker_points.push(viewport.to_physical_point(coord));
                }
            }
            for &(j, level) in &pt.neighbors {
                if i < j {
                    assert!(level < 3);

                    if let Some(TemporaryMovingBorderPoint {
                        point_or_segment: BorderPointOrSegment::Segment(x, y),
                        point_after_move: _,
                    }) = opts.temporary_moving_border_point
                    {
                        if (i, j) == (x, y) || (j, i) == (x, y) {
                            temporary_segment_level = Some(level);
                            continue;
                        }
                    }

                    let c1 = get_border_coord(i);
                    let c2 = get_border_coord(j);
                    if viewport.crosses_with_line_segment(c1, c2) {
                        border_points[level as usize].push(viewport.to_physical_point(c1));
                        border_points[level as usize].push(viewport.to_physical_point(c2));
                    }
                }
            }
        }
        if let Some(level) = temporary_segment_level {
            let temporary_moving_border_point = opts.temporary_moving_border_point.unwrap();
            if opts.marker_on_border_points {
                marker_points.push(temporary_moving_border_point.point_after_move);
            }
            let ps = temporary_moving_border_point.point_or_segment;
            if let BorderPointOrSegment::Segment(i, j) = ps {
                let c1 = get_border_coord(i);
                let c2 = get_border_coord(j);
                border_points[level as usize].push(viewport.to_physical_point(c1));
                border_points[level as usize].push(temporary_moving_border_point.point_after_move);
                border_points[level as usize].push(viewport.to_physical_point(c2));
                border_points[level as usize].push(temporary_moving_border_point.point_after_move);
            }
        }
        if let Some(extra) = opts.extra_border_segment {
            let level = extra.level as usize;
            assert!(level < 3);
            border_points[level]
                .push(viewport.to_physical_point(self.border_points[extra.point].coord));
            border_points[level].push(extra.new_point);
        }

        for level in 0..3 {
            if border_points[level].len() > 0 {
                rail_colors.push(Color { r: 0, g: 0, b: 0 });

                let (width, style) = match level {
                    0 => (1, 1),
                    1 => (1, 0),
                    2 => (2, 0),
                    _ => unreachable!(),
                };
                rail_width.push(width);
                rail_style.push(style);
                rail_points_num.push(border_points[level].len() as i32);
                rail_points.extend(&border_points[level]);
            }
        }

        let (rail_points_x, rail_points_y) = split_into_x_and_y(&rail_points);
        let (marker_points_x, marker_points_y) = split_into_x_and_y(&marker_points);

        assert_eq!(rail_width.len(), rail_style.len());

        RenderingInfo {
            rail_colors,
            rail_width,
            rail_style,
            rail_points_num: rail_points_num.into_boxed_slice(),
            rail_points_x: rail_points_x.into_boxed_slice(),
            rail_points_y: rail_points_y.into_boxed_slice(),
            marker_points_x,
            marker_points_y,
            stations,
        }
    }

    #[wasm_bindgen(js_name = findNearestSegment)]
    pub fn find_nearest_segment(
        &self,
        viewport: ViewportSpec,
        rail_id: RailwayIndex,
        x: i32,
        y: i32,
        max_dist: i32,
    ) -> Option<IndexOnRailway> {
        let viewport = Viewport::new(viewport);
        let p = Coord::new(x, y);

        let threshold = max_dist as i64 * max_dist as i64;

        let railway = self.railways.get(rail_id)?;

        let mut nearest = (threshold + 1, 0);

        for i in 0..railway.points.len() {
            let d = distance_norm_square_points(
                viewport
                    .to_physical_point(railway.points[i].coord)
                    .as_coord(),
                p,
            );
            if d < nearest.0 {
                nearest = (d, i);
            }
        }

        if nearest.0 <= threshold {
            return Some(IndexOnRailway {
                index: nearest.1,
                inserting: false,
            });
        }

        let mut nearest = (threshold + 1, 0);

        for i in 1..railway.points.len() {
            let c0 = viewport
                .to_physical_point(railway.points[i - 1].coord)
                .as_coord();
            let c1 = viewport
                .to_physical_point(railway.points[i].coord)
                .as_coord();
            let d = distance_norm_square_point_line_segment(c0, c1, p);
            if d < nearest.0 {
                nearest = (d, i);
            }
        }

        if nearest.0 <= threshold {
            Some(IndexOnRailway {
                index: nearest.1,
                inserting: true,
            })
        } else {
            None
        }
    }

    #[wasm_bindgen(js_name = findNearestBorder)]
    pub fn find_nearest_border(
        &self,
        viewport: ViewportSpec,
        x: i32,
        y: i32,
        max_dist: i32,
    ) -> Option<BorderPointOrSegment> {
        let viewport = Viewport::new(viewport);
        let p = Coord::new(x, y);

        let threshold = max_dist as i64 * max_dist as i64;

        let mut nearest = None;
        for (i, pt) in self.border_points.enumerate() {
            let d = distance_norm_square_points(viewport.to_physical_point(pt.coord).as_coord(), p);
            if d <= threshold {
                if nearest.map(|(d2, _)| d < d2).unwrap_or(true) {
                    nearest = Some((d, i));
                }
            }
        }
        if let Some((_, p)) = nearest {
            return Some(BorderPointOrSegment::Point(p));
        }

        let mut nearest = None;
        for (i, pt) in self.border_points.enumerate() {
            let c0 = viewport.to_physical_point(pt.coord).as_coord();
            for (j, _) in &pt.neighbors {
                let c1 = viewport
                    .to_physical_point(self.border_points[*j].coord)
                    .as_coord();
                let d = distance_norm_square_point_line_segment(c0, c1, p);
                if d <= threshold {
                    if nearest.map(|(d2, _)| d < d2).unwrap_or(true) {
                        nearest = Some((d, (i, *j)));
                    }
                }
            }
        }
        if let Some((_, (i, j))) = nearest {
            return Some(BorderPointOrSegment::Segment(i, j));
        }

        None
    }

    #[wasm_bindgen(js_name = getNumberOfPointsOnRailway)]
    pub fn get_number_of_points_on_railway(&self, rail_id: RailwayIndex) -> usize {
        self.railways[rail_id].points.len()
    }

    #[wasm_bindgen(js_name = getStationInfo)]
    pub fn get_station_info(&self, rail_id: RailwayIndex, point_idx: usize) -> Option<StationInfo> {
        let railway = self.railways.get(rail_id)?;
        if let Some(station_idx) = railway.points[point_idx].station {
            let station = &self[station_idx];
            return Some(StationInfo {
                name: station.name.clone(),
                level: station.level,
            });
        }
        None
    }

    #[wasm_bindgen(js_name = setStationInfo)]
    pub fn set_station_info(
        mut self,
        rail_id: RailwayIndex,
        point_idx: usize,
        info: StationInfo,
    ) -> RerailMap {
        let railway = self.railways.get_mut(rail_id);
        if let Some(railway) = railway {
            if let Some(station_idx) = railway.points[point_idx].station {
                self[station_idx].name = info.name;
                self[station_idx].level = info.level;
            } else {
                let station_idx = self.stations.push(Station::new(info.name, info.level));
                railway.points[point_idx].station = Some(station_idx);
                self[station_idx].add_railway(rail_id);
            }
        }
        self
    }

    #[wasm_bindgen(js_name = getRailwayInfo)]
    pub fn get_railway_info(&self, rail_id: RailwayIndex) -> RailwayInfo {
        let railway = &self.railways[rail_id];
        RailwayInfo {
            name: railway.name.clone(),
            level: railway.level,
            color: ((railway.color.r as u32) << 16)
                | ((railway.color.g as u32) << 8)
                | ((railway.color.b as u32) << 0),
        }
    }

    #[wasm_bindgen(js_name = setRailwayInfo)]
    pub fn set_railway_info(mut self, rail_id: RailwayIndex, info: RailwayInfo) -> RerailMap {
        let railway = &mut self.railways[rail_id];
        railway.name = info.name;
        railway.level = info.level;
        railway.color = Color {
            r: ((info.color >> 16) & 255) as u8,
            g: ((info.color >> 8) & 255) as u8,
            b: ((info.color >> 0) & 255) as u8,
        };
        self
    }

    #[wasm_bindgen(js_name = newRailwayFromInfo)]
    pub fn new_railway_from_info(
        mut self,
        info: RailwayInfo,
        x: i32,
        y: i32,
    ) -> RerailMapAndRailwayIndex {
        let railway = Railway {
            name: info.name,
            level: info.level,
            color: Color {
                r: ((info.color >> 16) & 255) as u8,
                g: ((info.color >> 8) & 255) as u8,
                b: ((info.color >> 0) & 255) as u8,
            },
            points: vec![RailwayPoint {
                coord: Coord::new(x, y),
                station: None,
            }],
        };
        let index = self.railways.push(railway);
        RerailMapAndRailwayIndex {
            map: Some(self),
            index,
        }
    }

    #[wasm_bindgen(js_name = removeRailway)]
    pub fn remove_railway(mut self, rail_id: RailwayIndex) -> RerailMap {
        let railway = &self.railways[rail_id];
        for pt in &railway.points {
            if let Some(station_idx) = pt.station {
                self.stations[station_idx].remove_railway(rail_id);
                if self.stations[station_idx].railways.is_empty() {
                    self.stations.delete(station_idx);
                }
            }
        }
        self.railways.delete(rail_id);
        self
    }

    #[wasm_bindgen(js_name = stationListOnRailway)]
    pub fn station_list_on_railway(&self, rail_id: RailwayIndex) -> StationListOnRailway {
        let railway = &self.railways[rail_id];
        let mut cur_distance = 0.0f64;
        let mut names = vec![];
        let mut distances = vec![];

        for i in 0..railway.points.len() {
            if i > 0 {
                let dist_sq = distance_norm_square_points(
                    railway.points[i - 1].coord,
                    railway.points[i].coord,
                );
                cur_distance += (dist_sq as f64).sqrt();
            }
            if let Some(id) = railway.points[i].station {
                let station = &self[id];
                names.push(station.name.clone());
                distances.push(cur_distance);
            }
        }

        StationListOnRailway { names, distances }
    }
}

impl Index<StationIndex> for RerailMap {
    type Output = Station;

    fn index(&self, index: StationIndex) -> &Self::Output {
        &self.stations[index]
    }
}

impl Index<RailwayIndex> for RerailMap {
    type Output = Railway;

    fn index(&self, index: RailwayIndex) -> &Self::Output {
        &self.railways[index]
    }
}

impl Index<BorderPointIndex> for RerailMap {
    type Output = BorderPoint;

    fn index(&self, index: BorderPointIndex) -> &Self::Output {
        &self.border_points[index]
    }
}

impl IndexMut<StationIndex> for RerailMap {
    fn index_mut(&mut self, index: StationIndex) -> &mut Self::Output {
        &mut self.stations[index]
    }
}

impl IndexMut<RailwayIndex> for RerailMap {
    fn index_mut(&mut self, index: RailwayIndex) -> &mut Self::Output {
        &mut self.railways[index]
    }
}

impl IndexMut<BorderPointIndex> for RerailMap {
    fn index_mut(&mut self, index: BorderPointIndex) -> &mut Self::Output {
        &mut self.border_points[index]
    }
}
