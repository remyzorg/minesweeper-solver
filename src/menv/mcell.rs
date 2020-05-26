use std::collections::HashMap;
use std::cmp::Ordering;

#[derive (Debug, Clone, PartialEq)]
pub enum Content {Mine, Empty (i32)}

#[derive (Debug, Clone, PartialEq)]
pub enum Score {
    NotEnough(i32),
    Mine,
    Val(i32)
}

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

#[derive (Debug, Clone)]
pub struct Cell {
    pub content : Content,
    pub scores : HashMap<(usize, usize), i32>,
    pub score : Score
}

impl Cell {
    pub fn from(c : Content) -> Cell {
        Cell { content : c, scores : HashMap::new(), score : Score::NotEnough(0)}
    }

    pub fn new() -> Cell {Cell::from (Content::Empty (0))}

    pub fn insert(&mut self, c : (usize, usize), score : i32) {
        self.scores.insert(c, score);
        self.refresh_score();
    }

    pub fn incr (&mut self) -> () {
        match self.content {
            Content::Empty(n) => self.content = Content::Empty(n + 1),
            Content::Mine => ()
        }
    }

    pub fn refresh_score (&mut self) {
        let nb = self.scores.iter().count() as i32;

        let sum = self.scores.iter_mut().fold(
            0, |acc, (_, s)|
            if *s == 1000 || acc == 1000 { 1000 }
            else { acc + *s }
        );

        self.score =
            if sum == 1000 { Score::Mine } else {
                let score = sum / nb;

                if nb <= 1 && (sum != 1000 || sum != 0) { Score::NotEnough(score) }
                else { Score::Val(score) }
            };
    }



}

impl ToString for Cell {
    fn to_string (&self) -> String {
        match self.content {
            Content::Mine => String::from("x"),
            Content::Empty(i) => i.to_string()
        }
    }
}
