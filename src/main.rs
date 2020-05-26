pub mod solver;

use solver::cell::Cell;
use std::io::stdin;

#[allow(dead_code)]
fn print_mat(v: &Vec<Vec<Cell>>) {
    for row in v.iter() {
        for c in row.iter() {
            print!("{}", c.to_string());
        }
        println!("");
    }
}

#[allow(dead_code)]
fn step(env: solver::Env) {
    let mut _s = String::new();
    let _ = stdin().read_line(&mut _s);
    env.print_hidden();
    env.print_stack();
}

// Error value for a lost game
// Carries the number of turns before losing
struct OpenError(i32);

// Runs a game.
fn play_game() -> Result<(), OpenError> {
    let h = 13;
    let w = 15;
    let nb_mines = 40;

    // Inits the solver
    let mut env = solver::Env::new(h, w, nb_mines);
    let mut nb_turns = 0;

    // Pops until there is nothing left to process, meaning the
    // game is over and won
    while let Some(c) = env.pop() {
        nb_turns = nb_turns + 1;

        // Opens current cell and possibly fails the game
        env.open(c).map_err(|_| OpenError(nb_turns))?;
        env.mark_obvious();

        // Refill stack with a random cell if no more cells are accessible
        // by visiting neighbours
        if env.stack.is_empty() {
            if let Some(c) = env.left.iter().next() {
                env.stack.push(*c);
            }
        }
    }

    Ok(())
}

fn main() {
    let mut cnt = 0;
    let mut win = 0;
    let qty = 1000;

    while cnt < qty {
        if play_game().is_ok() {
            win = win + 1;
        }
        cnt = cnt + 1;
    }
    println!("Win rate : {}%", win * 100 / qty);
}
