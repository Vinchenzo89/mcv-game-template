pub mod math;
pub mod render;
pub mod state;

use math::*;
use state::*;

pub type GameUpdateAndRenderFunc = extern "C" fn (input: &GameInput, ctx: &mut GameState); 

#[no_mangle]
pub extern "C" fn update_and_render(input: &GameInput, ctx: &mut GameState) {

    let screen_width = input.screen_width;
    let screen_height = input.screen_height;
    let world_width  = 2.0 * screen_width as f32;
    let world_height = 2.0 * screen_height as f32;

    // title fade based on movement
    if ctx.player.pos.x != 0.0 || ctx.player.pos.y != 0.0 {
        ctx.title_fade += 0.02;
        ctx.title_fade = ctx.title_fade.min(1.0);
    }

    if ctx.ship.fuel_burn_rate == 0.0 {
        ctx.ship.fuel_level = 1.0;
        ctx.ship.fuel_burn_rate = 0.001;
    }

    // init planets
    if let None = ctx.planets {
        ctx.sun = Sun {
            pos: vector_2f(0.0, 0.0),
            g_force: 0.002
        };

        let mut planets = Vec::new();
        let pos = [
            vector_2f(-600.0, -400.0),
            vector_2f(-600.0,  400.0),
            vector_2f( 600.0,  400.0),
            vector_2f( 600.0, -400.0),
        ]; 
        for p in pos {
            planets.push(Planet {
                pos: p,
                radius: 100.0,
                surface_radius: 100.0,
                g_radius: 250.0,
                g_force: 9.8, // 8 meters/s
                color: vector_4f(0.2, 0.5, 0.5, 1.0),
                lz_rel_pos: vector_2f(0.0, 0.0),
                lz_color: vector_4f(0.2, 0.2, 0.2, 1.0),
                item: PlanetItem { 
                    itype: PlanetItemType::Fuel , 
                    pos: vector_2f(10.0, 10.0) 
                },
            });
        } 
        ctx.planets = Some(planets);
    }

    // init stars
    if let None = ctx.space_stars {
        let star_density = 150;
        let star_offset_dist = 10.0;
        let star_x_step = world_width/star_density as f32;
        let star_y_step = world_height/star_density as f32;
        let star_start_x = -world_width/2.0;
        let star_start_y = -world_height/2.0;
        let mut stars = Vec::new();
        for y in 0..star_density {
            for x in 0..star_density {
                let point = vector_2f(
                    star_start_x + (x as f32 * star_x_step),
                    star_start_y + (y as f32 * star_y_step),
                );
                let ox = -star_offset_dist + (2.0 * star_offset_dist * rand::random::<f32>());
                let oy = -star_offset_dist + (2.0 * star_offset_dist * rand::random::<f32>());
                let offset = vector_2f(ox, oy);
                let point = vector_2f_add(point, offset);
                let size = 1.0;
                stars.push(Star { pos: point, size });
            }
        }
        ctx.space_stars = Some(stars);
    };
    
    let mut acceleration = 0.0;
    if input.accelerate {
        acceleration = 0.3;
    }
    if input.decelerate {
        acceleration = -0.3;
    }

    // Handle space flight burn
    if !ctx.player.landed {
        // acceleration is only allowed if we have fuel
        ctx.ship.fuel_level = if acceleration != 0.0 {
            (ctx.ship.fuel_level - ctx.ship.fuel_burn_rate).max(0.0)
        } else {
            ctx.ship.fuel_level
        };
        if ctx.ship.fuel_level <= 0.0 {
            acceleration = 0.0;
        }
    }

    let mut rotation_speed: f32 = 0.0;
    if input.turn_left {
        rotation_speed = 0.1;
    }
    if input.turn_right {
        rotation_speed = -0.1;
    }

    ctx.player.rot += rotation_speed;
    let direction = vector_2f(ctx.player.rot.cos(), ctx.player.rot.sin());
    ctx.player.dd_pos = acceleration;

    let mut forces = match ctx.debug_player_forces {
        Some(_) => ctx.debug_player_forces.take().unwrap(),
        None => Vec::new()
    };
    forces.clear();

    let player_velocity = vector_2f_scale(direction, acceleration);
    forces.push(player_velocity);

    // Accumulate gravity from all space bodies
    let mut planet_g_accum = vector_2f(0.0, 0.0);

    // then the planets
    if let Some(planets) = &ctx.planets {
        let mut player = &mut ctx.player;

        for p in planets.iter() {
            let p_dir = vector_2f_sub(p.pos, player.pos);
            let p_dist = vector_2f_length(p_dir);
            if p_dist < p.surface_radius {
                player.landed = true;
                let planet_surface_friction = vector_2f_scale(player.d_pos, -0.1);
                forces.push(planet_surface_friction);
                planet_g_accum = vector_2f_add(planet_g_accum, planet_surface_friction);
            }
            else if p_dist.between(p.radius, p.g_radius) {
                player.landed = false;
                let planet_g_force = p.g_force * input.frame_dt_sec;
                let planet_g_velocity = vector_2f_scale(vector_2f_normalize(p_dir), planet_g_force);
                forces.push(planet_g_velocity);
                planet_g_accum = vector_2f_add(planet_g_accum, planet_g_velocity);
            }
        }
    }

    // Sun gravity only felt when in space
    if ctx.player.landed == false {
        let sun_dir = vector_2f_sub(ctx.sun.pos, ctx.player.pos);
        let sun_g_velocity = vector_2f_scale(sun_dir, ctx.sun.g_force * input.frame_dt_sec);
        planet_g_accum = vector_2f_add(planet_g_accum, sun_g_velocity);
    }

    ctx.debug_player_forces = Some(forces);

    // Apply the final player velocity after all forces have been calculated
    let player_velocity = vector_2f_add(player_velocity, planet_g_accum);
    // cap velocity
    let max_velocity:f32 = 20.0;
    let new_d_pos = vector_2f_add(ctx.player.d_pos, player_velocity);
    let new_d_pos_len = vector_2f_length(new_d_pos); 
    if new_d_pos_len <= max_velocity {
        ctx.player.d_pos = new_d_pos;
    }
    ctx.player.pos = vector_2f_add(ctx.player.pos, ctx.player.d_pos);
    ctx.player.pos.x = ctx.player.pos.x.wrap(-world_width as f32 / 2.0, world_width as f32 / 2.0);
    ctx.player.pos.y = ctx.player.pos.y.wrap(-world_height as f32 / 2.0, world_height as f32 / 2.0);

}
