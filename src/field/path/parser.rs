use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, digit1, line_ending, space0},
    combinator::{eof, map, map_res, opt},
    multi::many0,
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult, Parser,
};

use crate::field::cell::Cell;

type Link = (Cell, Cell);

const PATH_START_PATTERN: &str = "--- Path ";
const PATH_END_PATTERN: &str = " ---";

fn number(input: &str) -> IResult<&str, i32> {
    map_res(digit1, str::parse).parse(input)
}

fn cell(input: &str) -> IResult<&str, Cell> {
    map(
        delimited(
            tag("("),
            separated_pair(number, char(','), number),
            tag(")"),
        ),
        |(x, y)| Cell::new(x as usize, y as usize),
    )
    .parse(input)
}

fn link(input: &str) -> IResult<&str, Link> {
    map(separated_pair(cell, tag(" -> "), cell), |(a, b)| (a, b)).parse(input)
}

fn link_line(input: &str) -> IResult<&str, Link> {
    preceded(space0, terminated(link, alt((line_ending, eof)))).parse(input)
}

fn path_header(input: &str) -> IResult<&str, ()> {
    map(
        (
            opt(line_ending),
            space0,
            tag(PATH_START_PATTERN),
            digit1,
            tag(PATH_END_PATTERN),
            line_ending,
        ),
        |_| (),
    )
    .parse(input)
}

fn path_block(input: &str) -> IResult<&str, Vec<Link>> {
    preceded(path_header, many0(link_line)).parse(input)
}

fn skip_until_path(input: &str) -> IResult<&str, &str> {
    take_until(PATH_START_PATTERN).parse(input)
}

pub fn parse_neos_output(mut input: &str) -> IResult<&str, Vec<Vec<Link>>> {
    let mut paths = Vec::new();

    while let Ok((i, _)) = skip_until_path(input) {
        let (new_input, path) = path_block(i)?;
        paths.push(path);
        input = new_input;
    }

    Ok((input, paths))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number() {
        assert_eq!(number("123"), Ok(("", 123)));
        assert_eq!(number("7abc"), Ok(("abc", 7)));
    }

    #[test]
    fn test_cell() {
        let result = cell("(7,5)");
        println!("Cell parse result: {:?}", result);
        assert_eq!(result, Ok(("", Cell::new(7, 5))));
    }

    #[test]
    fn test_link() {
        let result = link("(7,5) -> (8,5)");
        println!("Link parse result: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_link_line() {
        let result = link_line(" (7,5) -> (8,5)\n");
        println!("Link line parse result: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_header() {
        let result = path_header("--- Path 1 ---\n");
        println!("Path header parse result: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_simple_path() {
        let input = "--- Path 1 ---\n (7,5) -> (8,5)\n (8,5) -> (9,5)\n";
        let result = path_block(input);
        println!("Path block parse result: {:?}", result);
        assert!(result.is_ok());
    }
}
