// baidu platform config
const APP_KEY: &str = "*"; // change it to yourself
const SECRET_KEY: &str = "*";  // change it to yourself
pub const USER_FILE: &str = "./user.txt";
const AUTH_BASE_URL: &str = "https://openapi.baidu.com/oauth/2.0/";
const USER_INFO_QUERY_URL: &str = "https://pan.baidu.com/rest/2.0/xpan/nas?method=uinfo";

use std::fs;
use std::error::Error;
use reqwest;
use serde::Deserialize;
use serde_json;
use std::process;
use termion::color;

// introduce simple user input mod
pub use crate::simple_user_input;

pub struct Config {
    pub access_token: String,
    pub refresh_token: String,
    pub user: String,
}


impl Config {
    pub async fn init() -> Self {
        // init program
        // 1. read config from local, and check access_token validity
        // 2. if err in front process, get info from platform
        // 3. if err in front process, exit program
        // 4. else return Config struct
        match Config::read_config_from_local() {
            Ok(content) if !content.trim().is_empty() && Config::access_token_is_valid(Config::extract_access_token(&content).unwrap()).await.unwrap() => {
                let access_token = Config::extract_access_token(&content).unwrap();
                let refresh_token = Config::extract_refresh_token(&content).unwrap();

                let user = Config::query_user_info(access_token).await.unwrap();

                return Config { access_token: access_token.to_string(), refresh_token: refresh_token.to_string(), user: user.baidu_name.to_string()};
            },
            _ => {
                match Config::query_config_from_platfrom().await {
                    Ok(auth_code) => {
                        let access_token = auth_code.access_token.clone();
                        let refresh_token = auth_code.refresh_token.clone();
                        Config::write_user_info(&auth_code).expect("Can not write user infomation into user.txt");
                        let user = Config::query_user_info(&access_token).await.unwrap();
                        return Config {access_token, refresh_token, user: user.baidu_name.to_string()};
                    },
                    Err(err) => {
                        println!("{}Program error: {err}{}", color::Fg(color::Red), color::Fg(color::Reset));
                        process::exit(1);
                    }
                }
            }
        }
    }

    fn read_config_from_local() -> Result<String, Box<dyn Error>> {
        // read user info from local user info
        if !fs::metadata(USER_FILE).is_ok() {
            // check the user info file is exists or not
            // if not create a new one and return Error
            fs::File::create(USER_FILE).expect("Can not create user.txt");
            let err_info = format!("{}Unable to locate the local file containing user information.{}\n", color::Fg(color::Red), color::Fg(color::Reset));
            return Err(err_info.into());
        }
        let content = fs::read_to_string(USER_FILE).expect("Unable to read user info file.\nTrying to query user infomation from baidu platform.");

        Ok(content)
    }

    fn extract_access_token(content: &str) -> Option<&str> {
        // extract access_token from local user info file
        let access_token = content.split("\n").nth(0)?.split(":").nth(1)?.trim();
        Some(access_token)
    }

    fn extract_refresh_token(content: &str) -> Option<&str> {
        // extract refresh_token from local user info file
        let refresh_token = content.split("\n").nth(1)?.split(":").nth(1)?.trim();
        Some(refresh_token)
    }

    async fn query_config_from_platfrom() -> Result<AuthCode, Box<dyn Error>> {
        // query user info from baidu platform
        // 1. request device code
        let device_codo_response: DeviceCodeResponse = Config::request_device_code().await.unwrap_or_else(|err| {
            println!("Program error: {err}");
            process::exit(1)
        });

        // 2. print verification url and user_code, ask user to login
        println!("Please navigate to the {}{}{} and log in. Once logged in, enter the user code \"{}{}{}\" 
        ", color::Fg(color::Red), &device_codo_response.verification_url, color::Fg(color::Reset), color::Fg(color::Red), &device_codo_response.user_code, color::Fg(color::Reset));
        // 3. waiting user verify to continue
        loop {
            let input = simple_user_input::get_input ("Please enter 'Y' to continue or 'N' to cancel: ");
            match input {
                Ok(ref value) if value.to_uppercase() == "Y" => break,
                Ok(ref value) if value.to_uppercase() == "N" => {
                    println!("Operation canceled.");
                    process::exit(1);
                }
                _ => {
                    println!("Invalid input. Please enter 'Y' or 'N'.");
                    continue;
                }
            }
        }
        // 4. request access code
        let resp: AuthCode = Config::request_access_code(&device_codo_response.device_code).await.unwrap_or_else(|err| {
            println!("Program error: {err}");
            process::exit(1);
        });

        // response auth code if everything works fine.
        Ok(resp)
        
    }

    async fn request_device_code() -> Result<DeviceCodeResponse, Box<dyn Error>> {
        // query verification_url, device_code, user_code from baidu platform 
        let url = format!("{AUTH_BASE_URL}device/code?response_type=device_code&client_id={APP_KEY}&scope=basic,netdisk");
        let response = reqwest::get(url).await?.text().await?;
        let body: DeviceCodeResponse = serde_json::from_str(&response)?;

        Ok(body)
    }

    async fn request_access_code(device_code: &str) -> Result<AuthCode, Box<dyn Error>>{
        let url = format!("{AUTH_BASE_URL}token?grant_type=device_token&code={device_code}&client_id={APP_KEY}&client_secret={SECRET_KEY}");
        let response = reqwest::get(url).await?.text().await?;
        let body:AuthCode = serde_json::from_str(&response)?;

        Ok(body)
    }

    fn write_user_info(auth_code: &AuthCode) -> Result<(), Box<dyn Error>> {
        // write user info to local file
        let content = format!("access_token: {}\nrefresh_token: {}", auth_code.access_token, auth_code.refresh_token);
        fs::write(USER_FILE, content)?;

        Ok(())
    }


    pub async fn access_token_is_valid(access_token: &str) -> Result<bool, Box<dyn Error>>{
        // using access_token to query user info to generate greeting info
        let url = format!("{USER_INFO_QUERY_URL}&access_token={access_token}");
        let response = reqwest::get(url).await?.text().await?;
        let body: UserInfoErr = serde_json::from_str(&response)?;
        if body.errno == 0 {
            return Ok(true);
        } else {
            println!("{}", body.errmsg);
            return Ok(false);
        }
    }

    pub async fn query_user_info(access_token: &str) -> Result<UserInfo, Box<dyn Error>> {
        // using access_token to query user info to generate greeting info
        let url = format!("{USER_INFO_QUERY_URL}&access_token={access_token}");
        let response = reqwest::get(url).await?.text().await?;
        let body: UserInfo = serde_json::from_str(&response)?;
        Ok(body)
    }
}

#[derive(Deserialize)]
struct DeviceCodeResponse {
device_code: String,
user_code: String,
verification_url: String,
}

#[derive(Deserialize)]
struct AuthCode {
access_token: String,
refresh_token: String,
}

#[derive(Deserialize)]
pub struct UserInfoErr {
    pub errno: i32,
    pub errmsg: String,
}

#[derive(Deserialize)]
pub struct UserInfo {
    pub baidu_name: String,
}
