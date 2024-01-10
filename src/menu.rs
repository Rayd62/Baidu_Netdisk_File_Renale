pub use crate::simple_user_input;
use termion::color;


pub fn greeting() {
    println!("{}
This is the Baidu Cloud Drive File Management Tool.

This program is designed to assist you with managing files in your Baidu Cloud Drive. Currently, it offers two main functions.{}", color::Fg(color::Red), color::Fg(color::Reset));
}

pub fn menu() {
    println!("
{}Main Menu:
1. Batch File Name Change:
   This function allows you to rename multiple files in your Baidu Cloud Drive simultaneously.

2. Batch File Extension Change:
   Use this function to modify the file extensions of multiple files at once.

Please choose the desired operation by entering the corresponding number.

To exit the program at any time, simply press 'Ctrl + C' or 'x'.
{}", color::Fg(color::Yellow), color::Fg(color::Reset));
}