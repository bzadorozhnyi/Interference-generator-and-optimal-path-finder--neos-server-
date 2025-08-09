use std::collections::HashSet;

use crate::field::cell::Cell;

#[derive(serde::Serialize)]
pub struct PinkPairParam {
    name: String,
    values: Vec<usize>,
}

impl PinkPairParam {
    pub fn new(pink_pairs: HashSet<(&Cell, &Cell)>) -> Vec<PinkPairParam> {
        let mut result = Vec::new();
        let mut counter = 1;

        for (a, b) in pink_pairs {
            let values = vec![1, a.x, 2, a.y, 3, b.x, 4, b.y];
            result.push(PinkPairParam {
                name: format!("pink_pair{}", counter),
                values,
            });
            counter += 1;
        }

        result
    }
}
