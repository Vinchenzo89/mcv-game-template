use crate::state::*;

pub trait IRenderer {
    fn render(x: i32, y: i32, width: i32, height: i32, ctx: &GameState);
}