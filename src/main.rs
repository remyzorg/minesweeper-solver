
pub mod menv;


// use std::io::stdin;
use menv::mcell::Cell;

#[allow(dead_code)]
fn print_mat (v : &Vec<Vec<Cell>>) {
    for row in v.iter()  {
        for c in row.iter() {
            print!("{}", c.to_string());
        }
        println!("");
    }
}

struct OpenError(i32);

fn play_game() -> Result<(), OpenError>{
    let h = 13;
    let w = 15;
    let nb_mines = 40;
    // print_mat(&env.m);
    let mut env = menv::Env::new(h, w, nb_mines);
    let mut nb_turns = 0;
    let mut _s = String::new();

    while let Some (c) = env.pop() {
        nb_turns = nb_turns + 1;
        env.open(c).map_err(|_| OpenError(nb_turns))?;
        env.mark_obvious();

        // if nb_turns < 1 {
            // let _ = stdin().read_line(&mut _s);
            // env.print_hidden();
            // env.print_stack();
            // break
        // }

        if env.stack.is_empty() {
            if let Some (c) = env.left.iter().next() {
                env.stack.push(*c);
            }
        }
    }

    println!("Solved: mines : {:?}\n", env.marked.iter().count());
    env.print_hidden();
    Ok(())
}


fn main() {
    let mut cnt = 0;
    while let Err(OpenError(nb_turns)) = play_game () {
        cnt = cnt + 1;
        println!("FAIL: after {}. Trying another game.", nb_turns);
    }
    println!("FAILED : {}", cnt);
}
