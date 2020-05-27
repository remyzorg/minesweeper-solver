use std::cmp::Ordering;
use std::collections::HashMap;

// A cell is a mine or empty with the amount of
// neighbours mine
#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    Mine,
    Empty(i32),
}

// The score computed from the gathering of
// mine probability of neighbours
#[derive(Debug, Clone, PartialEq)]
pub enum Score {
    NotEnough(i32),
    Mine,
    Val(i32),
}

// To sort cells depending on their score
impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Score) -> Option<Ordering> {
        match (self, other) {
            (Score::Mine, _) => Some(Ordering::Greater),
            (_, Score::Mine) => Some(Ordering::Less),
            (Score::NotEnough(n1), Score::NotEnough(n2)) => n1.partial_cmp(n2),
            (Score::NotEnough(_), _) => Some(Ordering::Greater),
            (_, Score::NotEnough(_)) => Some(Ordering::Less),
            (Score::Val(n1), Score::Val(n2)) => n1.partial_cmp(n2),
        }
    }
}

// A cell is a content, scores givent by neighbours
// and the actual score computed by the gathering of scores
#[derive(Debug, Clone)]
pub struct Cell {
    pub content: Content,
    pub scores: HashMap<(usize, usize), i32>,
    pub score: Score,
}

impl Cell {
    // Creates a cell from the content
    pub fn from(c: Content) -> Cell {
        Cell {
            content: c,
            scores: HashMap::new(),
            score: Score::NotEnough(0),
        }
    }

    // Creates an empty cell
    pub fn new() -> Cell {
        Cell::from(Content::Empty(0))
    }

    // Inserts a new score to a cell
    // also refreshes the actual score for synchronicity
    pub fn insert(&mut self, c: (usize, usize), score: i32) {
        self.scores.insert(c, score);
        self.refresh_score();
    }

    // Incrementes the value of the cell content
    // Only used to create the board
    pub fn incr(&mut self) -> () {
        match self.content {
            Content::Empty(n) => self.content = Content::Empty(n + 1),
            Content::Mine => (),
        }
    }

    // Computes the cell score using [scores]
    pub fn refresh_score(&mut self) {
        //numbers of neighbours attributing a score
        let nb = self.scores.iter().count() as i32;

        // we sum all the scores given by neighbours unless
        // there is a 0 or a 1000
        // 0 means the cell cannot be a mine
        // 1000 means the cell must be a mine
        let sum = self.scores.iter_mut().fold(1, |acc, (_, s)| {
            if *s == 0 || acc == 0 {
                0
            } else if *s == 1000 || acc == 1000 {
                1000
            } else {
                acc + *s
            }
        });

        // This info is translated to a Score value
        self.score = if sum == 1000 {
            Score::Mine
        } else {
            let score = sum / nb;

            if nb == 1 && sum != 1000 && sum != 0 {
                Score::NotEnough(score)
            } else {
                Score::Val(score)
            }
        };
    }
}

impl ToString for Cell {
    fn to_string(&self) -> String {
        match self.content {
            Content::Mine => String::from("x"),
            Content::Empty(i) => i.to_string(),
        }
    }
}
