use std::error::Error;

use reqwest;
use reqwest::header;
use serde::Deserialize;
use serde_json;
use urlencoding::encode;


use crate::file_mgmt_operation;

const FILE_INFO_URL: &str = "https://pan.baidu.com/rest/2.0/xpan/file?method=list";
const FILE_MGMT_URL: &str = "https://pan.baidu.com/rest/2.0/xpan/file?method=filemanager";

#[derive(Deserialize, Debug)]
pub struct BaiduDirectory {
    errno: i32,
    list: Vec<File>,
}

#[derive(Deserialize, Debug)]
struct File {
    server_filename: String,
    path: String,
}


#[derive(Deserialize)]
struct ErrNo{
    errno: i32,
}

// impl fmt::Display for ErrNo {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "There is an error: {}", self.errno)
//     }
// }

// impl Error for ErrNo {}



impl BaiduDirectory {
    pub async fn get_files(client: &reqwest::Client, dir: &str, access_token: &str) -> Result<BaiduDirectory, Box<dyn Error>> {
        // get files in dir which user specified
        
        // test dir, remenber delete after function work well
        let dir_name = encode(dir);


        let url = format!("{FILE_INFO_URL}&dir={dir_name}&web=0&access_token={access_token}");
        let response = client
            .get(url)
            .header(header::USER_AGENT, "pan.baidu.com")
            .send()
            .await?
            .text()
            .await?;

        let body: ErrNo = serde_json::from_str(&response)?;
        match body.errno {
            0 => {
                let body: BaiduDirectory = serde_json::from_str(&response)?;
                println!("Files in {dir}");
                for file in &body.list {
                    println!("{}", file.server_filename);
                }
                return Ok(body);
            },
            -7 => {
                return Err("文件或目录无权访问".into());
            },
            -9 => {
                return Err("文件或目录不存在".into());
            },
            _ => {
                return Err("Unknown Error Occurs.".into());
            }
        }
    }

    // rename files

    fn find_episodes(tagged_episode: &str) -> (usize, usize) {
        let begin = tagged_episode.find("$").unwrap();
        let end = tagged_episode.rfind("$").unwrap() - 1;
        return (begin, end)
    }

    fn new_filename(file: &File, season: &str, begin: usize, end: usize) -> String {
        let (_, extension) = file
                            .server_filename
                            .rsplit_once(".").unwrap();
        let episode = file.server_filename
                            .get(begin..end).unwrap();
        format!("{season}E{episode}.{extension}")
    }


    pub fn get_all_new_files(&self, season: &str, tagged_episode: &str) {
        // find the episode location
        let (begin, end) = BaiduDirectory::find_episodes(tagged_episode);

        // check the destination info is correct
        for file in &self.list {
            let new_name = BaiduDirectory::new_filename(file, season, begin, end);
            println!("{:?}", new_name);
        }
    }

    pub async fn rename_files(&self, client: &reqwest::Client, season: &str, access_token: &str, tagged_episode: &str) {
        // format rename url
        let url = format!("{FILE_MGMT_URL}&access_token={access_token}&opera=rename");
        
        // find the episode location
        let (begin, end) = BaiduDirectory::find_episodes(tagged_episode);

        let result_string: String = self.list
                .iter()
                .map(|file| {
                    let newname = BaiduDirectory::new_filename(file, season, begin, end);
                    format!("{{\"path\":\"{}\",\"newname\":\"{}\"}}", &file.path, &newname)
                })
                .collect::<Vec<_>>()
                .join(",");

        // construct post data in x-www-form-urlencoded
        let form = reqwest::multipart::Form::new()
                .text("async", "2")
                .text("filelist", format!("[{}]", &result_string))
                .text("ondup", "fail");

        // make post and read the errno to check.
        match file_mgmt_operation::post_request(client, &url, form).await {
                Ok( file_mgmt_operation::ErrNo { errno: 0 } ) => println!("success!"),
                Ok( file_mgmt_operation::ErrNo { errno: -9 } ) => println!("文件不存在."),
                Ok( file_mgmt_operation::ErrNo { errno: 111 } ) => println!("有其他异步任务正在执行"),
                Ok( file_mgmt_operation::ErrNo { errno: -7 } ) => println!("文件名非法"),
                Ok( file_mgmt_operation::ErrNo { errno: 2 } ) => println!("参数错误"),
                Ok(_) => println!("Unkownn Error Occusr.",),
                Err(e) => eprintln!("{e}"),

            }
        
    }


    // rename extension
    fn new_extension(file: &File, extension: &str) -> String {
        let (filename, _) = file
                            .server_filename
                            .rsplit_once(".").unwrap();
        format!("{filename}.{extension}")
    }


    pub fn compare_new_old_extension(&self, extension: &str) {
        let (_, old_extension) = self.list[0].server_filename.rsplit_once(".").unwrap();
        println!("{old_extension} -> {extension}")
    }


    pub async fn rename_extension(&self, client: &reqwest::Client, access_token: &str, extension: &str) {
        let url = format!("{FILE_MGMT_URL}&access_token={access_token}&opera=rename");

        let result_string: String = self.list
                .iter()
                .map(|file| {
                    let newname = BaiduDirectory::new_extension(file, extension);
                    format!("{{\"path\":\"{}\",\"newname\":\"{}\"}}", &file.path, &newname)
                })
                .collect::<Vec<_>>()
                .join(",");

        let form = reqwest::multipart::Form::new()
                .text("async", "2")
                .text("filelist", format!("[{}]", &result_string))
                .text("ondup", "fail");

        // make post and read the errno to check.
        match file_mgmt_operation::post_request(client, &url, form).await {
                Ok( file_mgmt_operation::ErrNo { errno: 0 } ) => println!("success!"),
                Ok( file_mgmt_operation::ErrNo { errno: -9 } ) => println!("文件不存在."),
                Ok( file_mgmt_operation::ErrNo { errno: 111 } ) => println!("有其他异步任务正在执行"),
                Ok( file_mgmt_operation::ErrNo { errno: -7 } ) => println!("文件名非法"),
                Ok( file_mgmt_operation::ErrNo { errno: 2 } ) => println!("参数错误"),
                Ok(_) => println!("Unkownn Error Occusr.",),
                Err(e) => eprintln!("{e}"),

            }
    }

}