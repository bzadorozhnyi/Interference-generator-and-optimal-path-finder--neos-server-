use std::sync::LazyLock;

use regex::Regex;

pub static PATH_PARSER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\((?P<first_x>\d+),\s*(?P<first_y>\d+)\)\s*->\s*\((?P<second_x>\d+),\s*(?P<second_y>\d+)\)").unwrap());
