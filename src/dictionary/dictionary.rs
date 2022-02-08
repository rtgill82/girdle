// 
// Copyright (c) 2022, Robert Gill <rtgill82@gmail.com>
// 

use std::fs;
use std::io;

use std::cell::{Ref,RefCell};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead,BufReader};

use crate::dictionary::Error;
use crate::dictionary::Result;

pub struct Dictionary
{
    words: Vec<String>,
    include: RefCell<HashSet<char>>,
    exclude: RefCell<HashSet<char>>,
    positions: RefCell<[char; 5]>,
    matches: RefCell<Option<Vec<String>>>
}

#[derive(Clone,Copy)]
pub enum SetType
{
    Excluded,
    Included
}

fn find_dictionary<'a>(dictionaries: &'a [&str]) -> Result<&'a str> {
    for path in dictionaries {
        if let Ok(_) = fs::metadata(path) {
            return Ok(path);
        }
    }

    Err(Error::new("Unable to find a word database."))
}

impl Dictionary {
    pub fn new<'a>(dictionaries: &'a [&str]) -> Result<Dictionary>
    {
        let database = find_dictionary(dictionaries)?;
        let dictionary = Dictionary {
            words: read_words(database)?,
            include: RefCell::new(HashSet::new()),
            exclude: RefCell::new(HashSet::new()),
            positions: RefCell::new(['.'; 5]),
            matches: RefCell::new(None)
        };
        Ok(dictionary)
    }

    pub fn reset(&self) {
        (*self.include.borrow_mut()).clear();
        (*self.exclude.borrow_mut()).clear();
        *self.positions.borrow_mut() = ['.'; 5];
        *self.matches.borrow_mut() = None;
    }

    pub fn add_char(&self, set_type: SetType, ch: char) {
        match set_type {
            SetType::Excluded => self.exclude_char(ch),
            SetType::Included => self.include_char(ch)
        }
    }

    pub fn remove_char(&self, set_type: SetType, ch: char) {
        match set_type {
            SetType::Excluded => self.remove_excluded_char(ch),
            SetType::Included => self.remove_included_char(ch)
        }
    }

    pub fn clear_set(&self, set_type: SetType) {
        match set_type {
            SetType::Excluded => self.clear_excluded_chars(),
            SetType::Included => self.clear_included_chars()
        }
    }

    pub fn excluded_chars(&self) -> Vec<char> {
        let exclude = self.exclude.borrow();
        let mut vec = exclude.iter()
            .map(|ch| *ch).collect::<Vec<_>>();
        vec.sort();
        vec
    }

    pub fn included_chars(&self) -> Vec<char> {
        let include = self.include.borrow();
        let mut vec = include.iter()
            .map(|ch| *ch).collect::<Vec<_>>();
        vec.sort();
        vec
    }

    pub fn set_char_position(&self, pos: usize, ch: char) {
        if pos < 1 || pos > 5 {
            panic!("`pos` must be between 1 and 5.")
        }

        if ch == '.' {
            *self.matches.borrow_mut() = None;
        }

        (*self.include.borrow_mut()).remove(&ch);
        (*self.exclude.borrow_mut()).remove(&ch);
        (*self.positions.borrow_mut())[pos-1] = ch;
    }

    pub fn unset_char_position(&self, pos: usize) {
        self.set_char_position(pos, '.');
    }

    pub fn matches(&self) -> Ref<Option<Vec<String>>> {
        let mut matches = self.matches.borrow_mut();
        *matches = match &*matches {
            Some(matches) => Some(self.filter_matches(&matches)),
            None          => Some(self.filter_matches(&self.words)),
        };
        drop(matches);

        self.matches.borrow()
    }

    fn filter_matches(&self, matches: &Vec<String>) -> Vec<String> {
        let matches: Vec<String> = matches.into_iter().filter(|s| {
            if self.match_excluded(&s) {
                return false;
            }

            if self.match_included(&s) &&
                self.match_positions(&s)
            {
                return true;
            }

            return false;
        }).map(|s| String::from(s)).collect();

        matches
    }

    fn exclude_char(&self, ch: char) {
        (*self.include.borrow_mut()).remove(&ch);
        (*self.exclude.borrow_mut()).insert(ch);
    }

    fn include_char(&self, ch: char) {
        (*self.exclude.borrow_mut()).remove(&ch);
        (*self.include.borrow_mut()).insert(ch);
    }

    fn remove_excluded_char(&self, ch: char) {
        (*self.exclude.borrow_mut()).remove(&ch);
        *self.matches.borrow_mut() = None;
    }

    fn remove_included_char(&self, ch: char) {
        (*self.include.borrow_mut()).remove(&ch);
        *self.matches.borrow_mut() = None;
    }

    fn clear_excluded_chars(&self) {
        (*self.exclude.borrow_mut()).clear();
    }

    fn clear_included_chars(&self) {
        (*self.include.borrow_mut()).clear();
    }

    fn match_excluded(&self, s: &str) -> bool {
        let exclude = self.exclude.borrow();

        for ch in &*exclude {
            if s.contains(*ch) {
                return true;
            }
        }
        return false;
    }

    fn match_included(&self, s: &str) -> bool {
        let include = self.include.borrow();

        for ch in &*include {
            if s.contains(*ch) { continue; }
            return false;
        }
        return true;
    }

    fn match_positions(&self, s: &str) -> bool {
        let positions = self.positions.borrow();

        for (i, ch) in s.char_indices() {
            if positions[i] == ch { continue; }
            if positions[i] == '.' { continue; }
            return false;
        }
        return true;
    }
}

fn read_words(database: &str) -> io::Result<Vec<String>> {
    let file = File::open(database)?;
    let reader = BufReader::new(file);
    let mut matches = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.len() == 5 {
            matches.push(line.to_lowercase());
        }
    }

    Ok(matches)
}
