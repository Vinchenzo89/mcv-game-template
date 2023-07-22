use crate::math::*;

#[derive(Default)]
pub struct GameInput {
    pub screen_width: i32,
    pub screen_height: i32,
    pub frame_dt_sec: f32,
    pub turn_left: bool,
    pub turn_right: bool,
    pub accelerate: bool,
    pub decelerate: bool,
    pub launch_down: bool,
}

#[derive(Default)]
pub struct GameState {
    pub player: Player,
    pub ship: Ship,
    pub sun: Sun,
    pub planets: Option<Vec<Planet>>,
    pub space_stars: Option<Vec<Star>>,

    pub title_fade: f32,
    pub nav_path: Option<NavPath>,

    pub debug_player_forces: Option<Vec<Vector2f>>,
}

pub struct Star {
    pub pos: Vector2f,
    pub size: f32,
}

#[derive(Default)]
pub struct Sun {
    pub pos: Vector2f,
    pub g_force: f32,
}

#[derive(Default)]
pub struct Player {
    pub rot: f32,
    pub pos:    Vector2f,
    pub d_pos:  Vector2f,
    pub dd_pos: f32,
    pub landed: bool,
}

#[derive(Default)]
pub struct Ship {
    pub fuel_level: f32,
    pub fuel_burn_rate: f32,
}

const MAX_PLANET_ITEMS: usize = 5;
#[derive(Default)]
pub struct Planet {
    pub radius: f32,
    pub pos: Vector2f,
    pub color: Vector4f,

    pub g_radius: f32,
    pub g_force: f32,
    
    pub surface_radius: f32,
    pub lz_rel_pos: Vector2f,
    pub lz_color: Vector4f,

    pub item: PlanetItem,
}

#[derive(Default)]
pub struct PlanetItem {
    pub itype: PlanetItemType,
    pub pos: Vector2f,
}

#[derive(Default)]
pub enum PlanetItemType {
    #[default] 
    None,
    Fuel
}

pub struct NavPath {
    pub points: Vec<NavPoint>
}

pub struct NavPoint {
    pub p: Vector2f,
    pub c: Vector3f,
}
impl NavPoint {
    pub fn new(p: Vector2f, c: Vector3f) -> Self {
        Self { p, c }
    }
}