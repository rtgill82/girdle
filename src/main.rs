//
// Copyright (c) 2022, Robert Gill <rtgill82@gmail.com>
//

mod dictionary;
use dictionary::Dictionary;

mod gtk;
use crate::gtk::UI;

const ID: &str = "com.github.rtgill82.girdle";

const DICTIONARIES: &[&str] = &[
    "/usr/share/dict/words",
    "/usr/dict/words"
];

fn main() {
    let result = Dictionary::new(DICTIONARIES);
    UI::run(ID, result);
}
