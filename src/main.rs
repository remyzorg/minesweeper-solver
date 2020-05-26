
pub mod solver;


use std::io::stdin;
use solver::cell::Cell;

#[allow(dead_code)]
fn print_mat (v : &Vec<Vec<Cell>>) {
    for row in v.iter()  {
        for c in row.iter() {
            print!("{}", c.to_string());
        }
        println!("");
    }
}

#[allow(dead_code)]
fn step (env : solver::Env) {
    let mut _s = String::new();
    let _ = stdin().read_line(&mut _s);
    env.print_hidden();
    env.print_stack();
}

struct OpenError(i32);

fn play_game() -> Result<(), OpenError>{
    let h = 13;
    let w = 15;
    let nb_mines = 40;
    let mut env = solver::Env::new(h, w, nb_mines);
    let mut nb_turns = 0;

    while let Some (c) = env.pop() {
        nb_turns = nb_turns + 1;
        env.open(c).map_err(|_| OpenError(nb_turns))?;
        env.mark_obvious();

        if env.stack.is_empty() {
            if let Some (c) = env.left.iter().next() {
                env.stack.push(*c);
            }
        }
    }

    // println!("Solved: mines : {:?}\n", env.marked.iter().count());
    // env.print_hidden();
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
