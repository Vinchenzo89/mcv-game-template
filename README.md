# MCV Game Template

My template repo for learning game engine developement.

When starting a new game project, this repository is the starting template. As I make games and create more tooling/figure more things out, I add the improvements to this template so I have them for the next game.

This project also doubles as a learning basis to improve skills with Windows API, OpenGL, real-time applications, performance, and overall programming in Rust.

Most of what I learned is thanks to [Handmade Hero](https://handmadehero.org/)

## Code Style

As an experiment, I'm writing the code to match how I would code in C, being explicitly functional and verbose.  The style is influenced by Eskil Steenberg's C coding style.

I've found this style works really well in Rust and makes the code more fluid to read and write.

For instance, <code>impl</code> is only used in situations where structs require implementing a trait. In all other situations, the functions that operate on structs are implemented as static functions separate from the struct. In other words, as minimal OOP as possible.

The code has a much cleaner C-like feel; avoiding excess brackets and indentations required for the <code>impl</code> syntax.

This:

    struct GameState {
        pub game_prop: i32
    }

    impl GameState {
        fn update(&mut self) {
            self.game_prop += 1;
        }
    }

Is instead implemented as this:

    struct GameState {
        pub game_prop: i32
    }

    pub fn game_update(state: &mut GameState) {
        state.game_prop += 1;
    }





See the <code>game_lib/src/math.rs</code> for examples.