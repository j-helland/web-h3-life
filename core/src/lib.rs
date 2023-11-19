use std::collections::HashMap;

use geojson::{Value, Geometry, GeoJson, Feature, FeatureCollection};
use h3o::{LatLng, Resolution, CellIndex};
use rand::{rngs::StdRng, Rng, distributions::{Distribution, Uniform}, SeedableRng};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellState {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    cells: HashMap<CellIndex, CellState>,
}

#[wasm_bindgen]
impl Universe {
    pub fn new(num_init: usize, resolution: u8) -> Self {
        let h3_resolution = unsafe { Resolution::try_from(resolution).unwrap_unchecked() };
        let mut rng = rand::thread_rng();
        let coords = (0..num_init).map(|_| UniformSampler::sample_coord(&mut rng));
        let cells = coords.map(|c| c.to_cell(h3_resolution)).collect::<Vec<_>>();
        let mut cells_map = HashMap::new();
        for cell in cells {
            cells_map.insert(cell, CellState::Alive);
        }
        Universe { 
            cells: cells_map, 
        }
    }

    pub fn render(&mut self) -> String {
        let mut features: Vec<Feature> = Vec::new();
        let mut tombstones: Vec<CellIndex> = Vec::new();
        for (&index, &state) in self.cells.iter() {
            if state == CellState::Dead {
                tombstones.push(index);
                continue;
            }
            features.push(to_feature(Index(index).into()));
        }
        tombstones.into_iter().for_each(|index| {
            self.cells.remove(&index);
        });

        let geojson = GeoJson::FeatureCollection(FeatureCollection {
            features,
            bbox: None,
            foreign_members: None,
        });
        geojson.to_string()
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();
        for (index, &state) in self.cells.iter() {
            let neighbors = index
                .grid_disk::<Vec<_>>(1)
                .into_iter()
                .filter(|n| n != index)
                .collect::<Vec<_>>();

            // Cell reproduction.
            // Handle separately because dead cells are not stored in the map.
            for neighbor in &neighbors {
                if self.cells.contains_key(neighbor) {
                    continue;
                }
                let nc = self.live_neighbor_count(neighbor);
                if nc == 2 {
                    next.insert(*neighbor, CellState::Alive);
                }
            }

            let num_live_neighbors = self.live_neighbor_count(index);
            let next_state = match (state, num_live_neighbors) {
                (CellState::Alive, x) if x < 2 => CellState::Dead,
                (CellState::Alive, 2) | (CellState::Alive, 3) => CellState::Alive,
                (CellState::Alive, x) if x > 3 => CellState::Dead,
                (otherwise, _) => otherwise,
            };

            next.insert(*index, next_state);
        }
        self.cells = next;
    }

    fn live_neighbor_count(&self, index: &CellIndex) -> u8 {
        let neighbors = index.grid_disk::<Vec<_>>(1);
        neighbors
            .iter()
            .filter(|c| match self.cells.get(c) {
                Some(&state) => state == CellState::Alive,
                None => false,
            })
            .count() as u8
    }
}

/**
 * Creates a polygon from the vertices of an H3 cell. This will be a hexagon in most cases, except
 * for the pentagons on icosahedron vertices.
 */
struct Index(CellIndex);
impl Into<Geometry> for Index {
    fn into(self) -> Geometry {
        let boundary = self.0.boundary();
        let mut vertices = boundary
            .iter()
            .map(|v| vec![v.lng(), v.lat()])
            .collect::<Vec<_>>();

        // Handle edge case where polygon extends over the 180/-180 longitude.
        let mut is_swap_sign = false;
        for i in 1..vertices.len() {
            let lng1 = vertices[i-1][0];
            let lng2 = vertices[i][0];
            is_swap_sign |= (f64::max(lng1, lng2) - f64::min(lng1, lng2)).abs() > MAX_LNG;
            if is_swap_sign {
                break;
            }
        }
        if is_swap_sign {
            for v in &mut vertices {
                if v[0].is_sign_negative() {
                    v[0] += 2.0 * MAX_LNG;
                }
            }
        }
        vertices.push(vertices[0].clone());
        let vertices = vertices;

        Geometry::new(Value::Polygon(vec![vertices]))   
    }
}

fn to_feature(geom: Geometry) -> Feature {
    Feature{
        geometry: Some(geom),
        bbox: None,
        id: None,
        properties: None,
        foreign_members: None,
    }
}
    
const MIN_LAT: f64 = -90.0;
const MAX_LAT: f64 = 90.0;
const MIN_LNG: f64 = -180.0;
const MAX_LNG: f64 = 180.0;

pub fn create_rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

pub trait GeoSampler<R> {
    fn sample_coord(rng: &mut R) -> LatLng;
}

pub struct UniformSampler;
impl<R: Rng> GeoSampler<R> for UniformSampler {
    fn sample_coord(rng: &mut R) -> LatLng {
        let dist_lat = Uniform::new(MIN_LAT, MAX_LAT);
        let dist_lng = Uniform::new(MIN_LNG, MAX_LNG);
        unsafe { LatLng::new(dist_lng.sample(rng), dist_lat.sample(rng)).unwrap_unchecked() }
    }
}

