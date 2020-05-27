use std::collections::HashSet;

pub mod cell;

use cell::Cell;
use cell::Content;
use cell::Score;

use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug)]
pub struct OpenError;

// The solver environment
pub struct Env {
    // matrix to store cells infos
    pub m: Vec<Vec<Cell>>,
    // opened cells
    pub opened: HashSet<(usize, usize)>,
    // cells left to process (not opened, not marked)
    pub left: HashSet<(usize, usize)>,
    // cells marked with a flag
    pub marked: HashSet<(usize, usize)>,
    // current solver cell stack
    pub stack: Vec<(usize, usize)>,
    // another way to access previous info
    pub stacked: HashSet<(usize, usize)>,
}

fn dim<T>(v: &Vec<Vec<T>>) -> (usize, usize) {
    (v.len(), v[0].len())
}

// Creates a set of cell neighbours
pub fn neighbours(
    v: &Vec<Vec<Cell>>,
    (i, j): (usize, usize),
) -> HashSet<(usize, usize)> {
    let (h, w) = dim(v);
    let mut neighbours = HashSet::new();

    if i > 0 {
        neighbours.insert((i - 1, j));
        if j > 0 {
            neighbours.insert((i - 1, j - 1));
        };
    };

    if j > 0 {
        neighbours.insert((i, j - 1));
        if i < h - 1 {
            neighbours.insert((i + 1, j - 1));
        };
    };

    if i < h - 1 {
        neighbours.insert((i + 1, j));
        if j < w - 1 {
            neighbours.insert((i + 1, j + 1));
        };
    };
    if j < w - 1 {
        neighbours.insert((i, j + 1));
        if i > 0 {
            neighbours.insert((i - 1, j + 1));
        };
    };

    neighbours
}

// Generates an initial cell matrix to represent the gameboard
fn gen(h: usize, w: usize, mine_nb: usize) -> Vec<Vec<Cell>> {
    // Create the matrix
    let mut m: Vec<Vec<Cell>> = (0..h)
        .map(|_| (0..w).map(|_| Cell::new()).collect())
        .collect();

    // Generates all mines by drawing from a shuffled
    // vector of all cells
    let mines = {
        let mut v = Vec::new();
        (0..h).for_each(|i| {
            (0..w).for_each(|j| {
                v.push((i, j));
            })
        });

        // remove the last corner to be sure this is not a mine
        // so it can be used as the starting cell

        v.retain(|c| {
            !neighbours(&m, (h - 1, w - 1)).contains(c) && *c != (h - 1, w - 1)
        });

        v.shuffle(&mut thread_rng());
        v.truncate(mine_nb);
        v
    };

    // Inserts mines and cell values in the board
    mines.iter().for_each(|mine| {
        m[mine.0][mine.1] = Cell::from(Content::Mine);
        neighbours(&m, *mine)
            .iter()
            .for_each(|n| m[n.0][n.1].incr());
    });
    m
}

impl Env {
    // Creates a new solver Env taking the dimensions of the board
    // and the number of mines
    pub fn new(h: usize, w: usize, nb_mines: usize) -> Env {
        let mut env = Env {
            stack: Vec::new(),
            opened: HashSet::new(),
            left: HashSet::new(),
            stacked: HashSet::new(),
            marked: HashSet::new(),
            m: gen(h, w, nb_mines),
        };

        (0..h).for_each(|i| {
            (0..h).for_each(|j| {
                env.left.insert((i, j));
            })
        });

        let init_c = (h - 1, w - 1);

        env.stack.push(init_c);
        env.left.remove(&init_c);
        env
    }

    // Refreshes scores given to neighbours cells by [c]
    fn update_neighbours_score(&mut self, c: (usize, usize)) {
        let nbrs = neighbours(&(self.m), c);

        let nb_marked =
            nbrs.iter().filter(|c| self.marked.contains(c)).count() as i32;

        // remove marked neighbours from the current
        // neighbours mine info
        let v = match self.get(c).content {
            Content::Empty(n) => n - nb_marked,
            _ => 0,
        };

        // The vector of unknown neighbours
        let covered: Vec<(usize, usize)> = nbrs
            .iter()
            .filter(|c| {
                !(self.get(**c).score == Score::Val(0))
                    && !self.marked.contains(c)
                    && !self.opened.contains(c)
            })
            .cloned()
            .collect();

        let nb_covered = covered.len() as i32;

        // The score givent to neighbours is
        // - 1000 if there is the same amount of hidden neighbours than the
        //   cell content info. It means every hidden neighbours is a mine
        // - 0 if the current cell content info is 0. It means no neighbour
        //   can be a mine
        // - 100 / covered * info_value is the probability of a
        //   neighbour to be a mine
        let score = if nb_covered <= v {
            1000
        } else {
            if v == 0 {
                0
            } else {
                100 / nb_covered * v
            }
        };

        // push the neighbours on the stack if they are worth it
        for (i, j) in covered {
            self.m[i][j].insert(c, score);
            if !self.stacked.contains(&(i, j)) {
                self.stacked.insert((i, j));
                self.stack.push((i, j));
            }
        }
    }

    // Markes a cell
    pub fn mark(&mut self, c: (usize, usize)) {
        self.stacked.remove(&c);
        self.marked.insert(c);
        self.left.remove(&c);

        // Refreshes all neighbours' neighbours scores after marking
        neighbours(&self.m, c).iter().for_each(|nbr| {
            if self.opened.contains(nbr) {
                self.update_neighbours_score(*nbr);
            }
        });
    }

    // Sorts the stack to show lowest score in the end,
    // mines at the beginning and cell with not much info in between
    pub fn sort(&mut self) {
        let m = &self.m;
        self.stack.sort_by(|a, b| {
            m[a.0][a.1]
                .score
                .partial_cmp(&m[b.0][b.1].score)
                .unwrap()
                .reverse()
        });
    }

    // Pops the stack if possible
    pub fn pop(&mut self) -> Option<(usize, usize)> {
        self.stack.pop().map(|c| {
            self.stacked.remove(&c);
            self.left.remove(&c);
            c
        })
    }

    // Opens a cell. Game is lost if it's a Mine
    pub fn open(&mut self, c: (usize, usize)) -> Result<(), OpenError> {
        // fails if the given cell is a mine
        if let Content::Mine = self.get(c).content {
            return Err(OpenError);
        }

        // local stack to recursively open 0 mines
        let mut stack0: Vec<(usize, usize)> = Vec::new();

        // neighbours that must refresh their neighbours score
        // after the opening
        let mut seen0: HashSet<(usize, usize)> = HashSet::new();

        stack0.push(c);
        seen0.insert(c);

        // Pops until the is no empty cell to open
        while let Some(c) = stack0.pop() {
            if let Content::Empty(v) = self.get(c).content.clone() {
                self.opened.insert(c);

                for nbr in neighbours(&self.m, c) {
                    if self.opened.contains(&nbr) {
                        seen0.insert(nbr);
                    }

                    // Recursively open neighbours if the mine info is 0
                    if v == 0
                        && !self.opened.contains(&nbr)
                        && !self.marked.contains(&nbr)
                    {
                        seen0.insert(nbr);
                        stack0.push(nbr)
                    };
                }
            }
        }

        // Refresh all the impacted scores
        seen0.iter().for_each(|c| self.update_neighbours_score(*c));

        // Sort the stack after refreshing
        self.sort();

        Ok(())
    }

    // Immutable access to a cell in the matrix
    pub fn get(&self, c: (usize, usize)) -> &cell::Cell {
        &(self.m)[c.0][c.1]
    }

    // Markes all obvious cells (the n first in the stack)
    pub fn mark_obvious(&mut self) -> usize {
        // Collects cells as long as they are obvious mines
        let drained: Vec<(usize, usize)> = self
            .stack
            .iter()
            .take_while(|c| match self.get(**c).score {
                Score::Mine => true,
                _ => false,
            })
            .copied()
            .collect();

        // Mark them
        drained.iter().for_each(|c| self.mark(*c));

        // Remove them from stack in one row
        self.stack.drain(0..drained.len());

        // Sort stack
        self.sort();

        drained.len()
    }

    // Showes the current state of the stack
    #[allow(dead_code)]
    pub fn print_stack(&self) {
        for c in &self.stack {
            print!("({:?}) = {:?}; ", c, self.get(*c).score);
        }
        println!("");
    }

    // Print the current state of the discovered board
    #[allow(dead_code)]
    pub fn print_hidden(&self) {
        for (i, row) in self.m.iter().enumerate() {
            for (j, c) in row.iter().enumerate() {
                if self.marked.contains(&(i, j)) {
                    print!(" {:^3}", "@");
                } else if self.opened.contains(&(i, j))
                    && c.content == Content::Empty(0)
                {
                    print!(" {:^3}", "  ");
                } else if self.opened.contains(&(i, j)) {
                    print!(" {:^3}", c.to_string());
                } else {
                    match c.score {
                        Score::Mine => print!(" {:^3}", "SU"),
                        Score::Val(n) => {
                            if c.content == Content::Mine {
                                print!(" {:^2}t", n);
                            } else {
                                print!(" {:^2}?", n);
                            }
                        }
                        _ => print!(" {:^3}", "-"),
                    }
                }
            }
            println!("");
        }
    }
}
