
use std::collections::HashSet;

pub mod cell;

use cell::Score;
use cell::Cell;
use cell::Content;

use rand::thread_rng;
// use rand::Rng;
use rand::seq::SliceRandom;


#[derive (Debug)]
pub struct OpenError;

pub struct Env {
    pub m : Vec<Vec<Cell>>,
    pub opened : HashSet<(usize, usize)>,
    pub left : HashSet<(usize, usize)>,
    pub marked : HashSet<(usize, usize)>,
    pub stack : Vec<(usize, usize)>,
    pub stacked : HashSet<(usize, usize)>
}

fn dim<T>(v : &Vec<Vec<T>>) -> (usize, usize) {
    (v.len(), v[0].len())
}

pub fn neighbours
    (v : &Vec<Vec<Cell>>, (i, j) : (usize, usize))
     -> HashSet<(usize, usize)>
{
    let (h, w) = dim(v);
    let mut neighbours = HashSet::new();

    if i > 0 {
        neighbours.insert((i - 1, j));
        if j > 0 { neighbours.insert((i - 1, j - 1)); };
    };

    if j > 0 {
        neighbours.insert((i, j - 1));
        if i < h - 1 { neighbours.insert((i + 1, j - 1)); };
    };

    if i < h - 1 {
        neighbours.insert((i + 1, j));
        if j < w - 1 { neighbours.insert((i + 1, j + 1)); };
    };
    if j < w - 1 {
        neighbours.insert((i, j + 1));
        if i > 0 { neighbours.insert((i - 1, j + 1)); };
    };

    neighbours
}


fn gen (h : usize, w : usize, mine_nb : usize) -> Vec<Vec<Cell>> {

    let mines = {
        let mut v = Vec::new();
        (0..h).for_each(|i| (0..w).for_each(|j| {v.push((i, j));} ));
        v.pop();
        v.shuffle(&mut thread_rng());
        v.truncate(mine_nb);
        v
    };

    let mut v : Vec<Vec<Cell>> =
        (0..h).map(
            |_| (0..w).map(
                |_| Cell::new()
            ).collect()
        ).collect();

    mines.iter().for_each(|m| {
        v[m.0][m.1] = Cell::from(Content::Mine);
        neighbours(&v, *m).iter().for_each(
            |n| v[n.0][n.1].incr()
        );
    });
    v
}



impl Env {

    pub fn new (h : usize, w : usize, nb_mines : usize) -> Env {
        let mut env = Env {
            stack : Vec::new (),
            opened : HashSet::new (),
            left : HashSet::new (),
            stacked : HashSet::new (),
            marked : HashSet::new (),
            m : gen(h, w, nb_mines)
        };

        (0..h).for_each(|i| (0..h).for_each(
            |j| {env.left.insert((i, j));}
        ));

        // let mut rng = rand::thread_rng();
        let init_c = (h - 1, w - 1);
        // (rng.gen::<usize>() % h, rng.gen::<usize>() % w);

        env.stack.push(init_c);
        env.left.remove(&init_c);
        env
    }


    fn update_neighbours_score (&mut self, c : (usize, usize)) {
        let nbrs = neighbours (&(self.m), c) ;

        let nb_marked =
            nbrs.iter()
            .filter(|c|self.marked.contains(c))
            .count() as i32;

        let v = match self.get(c).content {
            Content::Empty(n) => n - nb_marked,
            _ => 0
        };

        let covered : Vec<(usize, usize)> = nbrs.iter().filter(
            |c| !self.marked.contains(c) && !self.opened.contains(c)
        ).cloned().collect();

        let nb_covered = covered.len() as i32;

        let score =
            if nb_covered <= v { 1000 } else {
                if v == 0 { 0 }
                else {
                    100 / nb_covered * v
                }
            };

        // println!("UPDATE_NBRS (c : {:?}) score:{} covered:{} marked:{} v:{}", c, score, nb_covered, nb_marked, v);
        for (i, j) in covered {
            self.m[i][j].insert(c, score);
            if !self.stacked.contains(&(i, j)) {
                self.stacked.insert((i, j));
                self.stack.push((i, j));
            }
        }
    }

    pub fn mark (&mut self, c : (usize, usize)) {

        self.stacked.remove(&c);
        self.marked.insert(c);
        self.left.remove(&c);

        let nbrs = neighbours(&self.m, c);

        for nbr in nbrs {
            if self.opened.contains(&nbr) {
                self.update_neighbours_score(nbr);
            }
        }
    }

    pub fn sort(&mut self) {
        let m = &self.m;
        self.stack.sort_by(|a, b|{
            m[a.0][a.1].score
                .partial_cmp(&m[b.0][b.1].score)
                .unwrap()
                .reverse()
        });
    }

    pub fn pop (&mut self) -> Option<(usize, usize)> {
        self.stack.pop().map(|c| {
            self.stacked.remove(&c);
            self.left.remove(&c);
            c
        }
        )
    }

    pub fn open (&mut self, c : (usize, usize)) -> Result<(), OpenError>
    {
        if let Content::Mine = self.get(c).content {
            return Err(OpenError)
        }

        let mut stack0 : Vec<(usize, usize)> = Vec::new();
        let mut seen0 : Vec<(usize, usize)> = Vec::new();

        stack0.push(c);
        self.opened.insert(c);
        seen0.push(c);

        while let Some (Content::Empty(v))
            = stack0.pop().map(|c| self.get(c).content.clone())
        {
            for nbr in neighbours(&self.m, c) {
                seen0.push(nbr);
                if v == 0 {
                    if let Content::Empty(nv) = self.get(nbr).content {
                        if !self.opened.contains(&nbr) {
                            self.opened.insert(nbr);
                            if nv == 0 { stack0.push(nbr) }
                        };
                    }
                }
            }
        }

        for nbr in seen0 {
            self.update_neighbours_score(nbr);
        }

        self.sort();
        Ok (())
    }

    pub fn get(&self, c : (usize, usize)) -> &cell::Cell {
        &(self.m)[c.0][c.1]
    }

    pub fn mark_obvious (&mut self)  {
        let drained : Vec<(usize, usize)> =
            self.stack.iter().take_while(
            |c| match self.get(**c).score {
                Score::Mine => true,
                _ => false
            }
        ).copied().collect();

        drained.iter().for_each(|c| self.mark(*c));
        self.stack.drain(0..drained.len());
        self.sort();
    }

    #[allow(dead_code)]
    pub fn print_stack (&self) {
        for c in &self.stack {
            print!("({:?}) = {:?}; ", c, self.get(*c).score);
        }
        println!("");
    }

    #[allow(dead_code)]
    pub fn print_hidden (&self) {
        for (i, row) in self.m.iter().enumerate()  {
            for (j, c) in row.iter().enumerate() {
                if self.marked.contains(&(i, j)) {
                    print!(" <{:^2}>", "M");
                }
                else if self.opened.contains(&(i, j)) {
                    print!(" ({:^2})", c.to_string());
                }
                else {
                    match c.score {
                        Score::Mine => print!(" {:^4}", "SU"),
                        Score::Val(n) =>
                            if c.content == Content::Mine {
                                print!(" {:^3}t", n);
                            } else {
                                print!(" {:^3}?", n);
                            }
                        _ => print!(" {:^4}", "-"),
                    }
                }
            }
            println!("");
        }
    }
}
