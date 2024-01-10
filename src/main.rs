use mylib::{init, menu, file_rename};
use mylib::simple_user_input::get_input;

use std::process;
use reqwest;
extern crate termion;
use termion::color;

#[tokio::main]
async fn main() {

    println!("{}Program Initing...", color::Fg(color::Blue));

    // Get Permission for Baidu Netdisk.
    let config = init::Config::init().await;

    // Login Success
    println!("Welcome {}{}{}{}, profile Successfully Loaded!{}", color::Fg(color::Yellow),config.user, color::Fg(color::Reset), color::Fg(color::Blue),color::Fg(color::Reset));
    menu::greeting();
    'main_menu: loop {
        // into main menu
        menu::menu();
        match get_input("Enter the number corresponding to the operation you'd like to perform:") {
            Ok(option) => { match option.as_str() {
                "1" => {
                    // setup delimiter
                    let delimiter = "*-*-*-*-*".repeat(5);
                    // 1. ask user input manipulat dir in Baidu Netdisk
                    let dir = match get_input("Please provide the directory name: ") {
                        Ok(x) if x == "x" => continue 'main_menu,
                        Ok(content) => content,
                        Err(e) => {
                            println!("Error: {e}, back to main menu");
                            continue 'main_menu;
                        },
                    };
                    println!("{}", &delimiter);

                    // 2. create a reqwest 
                    let client = reqwest::Client::new();

                    // 3. get files in that dir
                    let direcotry_content = match file_rename::BaiduDirectory::get_files(&client, &dir, &config.access_token).await {
                        Ok(files) => files,
                        Err(e) => {
                            println!("Error: {e}, back to main menu");
                            continue 'main_menu;
                        },
                    };


                    // 4. ask user to tag the episode with '$'
                    let tagged_episode = match get_input("Please tag the episode in the filename using the '$' character. For example, use 'aabbccS01$01$.mp4'") {
                        Ok(x) if x == "x" => continue 'main_menu,
                        Ok(content) => content,
                        Err(e) => {
                            println!("Error: {e}, back to main menu");
                            continue 'main_menu;
                        }
                    };
                    println!("{}", &delimiter);

                    // 5. ask user to identify season no.
                    let season_no = match get_input("Please enter the season number in the format 'S01'. This will be used to construct filenames like 'S01E01.mp4'.") {
                        Ok(x) if x == "x" => continue 'main_menu,
                        Ok(content) => content,
                        Err(e) => {
                            println!("Error: {e}, back to main menu");
                            continue 'main_menu;
                        }
                    };
                    println!("{}", &delimiter);

                    // 6. output all new files, and make user to decide continue or not
                    direcotry_content.get_all_new_files(&season_no, &tagged_episode);

                    'inner_loop: loop {
                        let _ = match get_input("Please confirm the new filenames above. Use 'y' to proceed or 'n' to cancel.") {
                            Ok(x) if x == "x" => continue 'main_menu,
                            Ok(ref value) if value.to_uppercase() == "Y" => break,
                            Ok(ref value) if value.to_uppercase() == "N" => {
                                println!("Operation canceled by user, back to main menu.");
                                continue 'main_menu;
                            },
                            Ok(_) => {
                                println!("Unkonw input, please enter Y/N.");
                                continue 'inner_loop;
                            },
                            Err(e) => {
                                println!("Program Error: {e}, back to main menu.");
                                continue 'main_menu;
                            }
                        };
                    }
                    println!("{}", &delimiter);
                    // 7. try to rename all files under this dir
                    direcotry_content.rename_files(&client, &season_no, &config.access_token, &tagged_episode).await;
                },
                "2" => {
                    // setup delimiter
                    let delimiter = "*-*-*-*-*".repeat(5);
                    // 1. ask user input manipulat dir in Baidu Netdisk
                    let dir = match get_input("Please provide the directory name: ") {
                        Ok(x) if x == "x" => continue 'main_menu,
                        Ok(content) => content,
                        Err(e) => {
                            println!("Error: {e}, back to main menu");
                            continue 'main_menu;
                        },
                    };
                    println!("{}", &delimiter);

                    // 2. create a reqwest 
                    let client = reqwest::Client::new();

                    // 3. get files in that dir
                    let direcotry_content = match file_rename::BaiduDirectory::get_files(&client, &dir, &config.access_token).await {
                        Ok(files) => files,
                        Err(e) => {
                            println!("Error: {e}, back to main menu");
                            continue 'main_menu;
                        },
                    };
                    // 4. ask user to provide new extension
                    let extension = match get_input("Please enter the new file extension:") {
                        Ok(x) if x == "x" => continue 'main_menu,
                        Ok(content) => content,
                        Err(e) => {
                            println!("Error: {e}, back to main menu");
                            continue 'main_menu;
                        }
                    };
                    println!("{}", &delimiter);

                    // 5. output old extension and new extentsion, and make user to decide continue or not
                    direcotry_content.compare_new_old_extension(&extension);

                    'inner_loop: loop {
                        let _ = match get_input("Please confirm the change above. Use 'y' to proceed or 'n' to cancel.") {
                            Ok(x) if x == "x" => continue 'main_menu,
                            Ok(ref value) if value.to_uppercase() == "Y" => break,
                            Ok(ref value) if value.to_uppercase() == "N" => {
                                println!("Operation canceled by user, back to main menu.");
                                continue 'main_menu;
                            },
                            Ok(_) => {
                                println!("Unkonw input, please enter Y/N.");
                                continue 'inner_loop;
                            },
                            Err(e) => {
                                println!("Program Error: {e}, back to main menu.");
                                continue 'main_menu;
                            }
                        };
                    }
                    println!("{}", &delimiter);
                    
                    // 6. try to rename extensions
                    direcotry_content.rename_extension(&client, &config.access_token, &extension).await;

                },
                "x" => {
                    process::exit(1)
                },
                _ => {println!("Invalid Option. Please choose a valid option and try again.");}
            } },
            Err(error) => {println!("{}", error);},
        }
    }
}
