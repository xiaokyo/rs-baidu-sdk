#![allow(dead_code)]

use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::io::Read;

use crate::lib::cmd::open_url;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pan {
    app_id: String,
    // appKey
    client_id: String,
    // SecretKey
    client_secret: String,
    device_info: DeviceInfo,
    access_token: AccessToken,
}

// 设备码信息
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeviceInfo {
    device_code: String,
    user_code: String,
    expires_in: u64,
    interval: u64,
    verification_url: String,
    qrcode_url: String,
}

// token数据集合
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccessToken {
    expires_in: u64,
    access_token: String,
    refresh_token: String,
    scope: String,
    session_key: String,
    session_secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserInfo {
    baidu_name: String,
    netdisk_name: String,
    errmsg: String,
    errno: u64,
    avatar_url: String,
    vip_type: u64,
    uk: u64,
    request_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum VipType {
    Normal = 0,
    Vip = 1,
    Super = 2,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CloundFile {
    server_filename: String,
    fs_id: u64,
    from_type: u64, // 0 文件、1 目录
    path: String,
    oper_id: u64,
    category: u64, // 1 视频、2 音频、3 图片、4 文档、5 应用、6 其他、7 种子
}

#[derive(Debug, Deserialize, Serialize)]
pub enum FileType {
    File = 0,
    Dir = 1,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum FileCategory {
    Video = 1,
    Audio = 2,
    Image = 3,
    Document = 4,
    Application = 5,
    Other = 6,
    Torrent = 7,
}

#[derive(Debug, Deserialize, Serialize)]
struct GetFileListRes {
    list: Vec<CloundFile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PanCapacity {
    used: u64,
    total: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Md5Chunks {
    chunk: Vec<u8>,
    md5: String,
}

impl Pan {
    pub fn new() -> Self {
        Pan {
            app_id: "26703329".to_string(),
            client_id: "LPfDNWUhfBsqk808mbPQpS0T6MUjO6xz".to_string(),
            client_secret: "jfc6WTyiVRliKyxGaGCbef69zVl1dHD8".to_string(),
            device_info: DeviceInfo {
                device_code: "6fc5e399ac9d1ff81bf425d6a8ccce49".to_string(),
                user_code: "6k7dnewv".to_string(),
                expires_in: 1800,
                interval: 5,
                verification_url: "https://openapi.baidu.com/device".to_string(),
                qrcode_url: "https://openapi.baidu.com/device/qrcode/5161d1ff637493364ef43b0d193da4d1/6k7dnewv".to_string(),
            },
            access_token: AccessToken {
                expires_in: 0,
                access_token: "126.00f95130de7bc0f9b30d450ccec52f40.YCjJewhvbzaS7hYvVUaSckwmoKH64BlrZJRfWcn.gaLbDA".to_string(),
                refresh_token: "127.1df467b0f2a657d39ac53cc6ca8f4ce2.YgzfaWGEDRKQIGVZ7Q4VsW80XnzCdJKHFUoYmHx.LD81CA".to_string(),
                scope: "".to_string(),
                session_key: "".to_string(),
                session_secret: "".to_string(),
            },
        }
    }

    /**
     * 获取设备码信息
     */
    pub async fn get_device_code(&mut self) -> &mut Self {
        let client = reqwest::Client::new();
        let url = format!("https://openapi.baidu.com/oauth/2.0/device/code?response_type=device_code&client_id={}&scope=basic,netdisk", self.client_id);
        println!("get_device_code url: {}", url);
        let res: DeviceInfo = client.post(url).send().await.unwrap().json().await.unwrap();
        println!("{:#?}", res);
        if res.qrcode_url != "" {
            // 打开二维码操作
            open_url(res.qrcode_url.as_str());
        }

        self.device_info = res;
        self
    }

    /**
     * 设备授权成功后需要定时获取token
     */
    pub async fn get_access_token(&mut self) -> &mut Self {
        println!("开始获取access_token");

        async fn request(
            device_code: String,
            client_id: String,
            client_secret: String,
        ) -> Result<AccessToken, Box<dyn std::error::Error>> {
            let client = reqwest::Client::new();
            let url = format!(
                "https://openapi.baidu.com/oauth/2.0/token?grant_type=device_token&code={}&client_id={}&client_secret={}",
                device_code, client_id, client_secret
            );
            println!("get_access_token url: {}", url);
            let res: AccessToken = client.post(url).body("").send().await?.json().await?;
            Ok(res)
        }

        let res = request(
            self.device_info.device_code.clone(),
            self.client_id.clone(),
            self.client_secret.clone(),
        )
        .await;

        match res {
            Ok(res) => {
                self.access_token = res;
                println!("获取access_token成功: {:#?}", self.access_token);
            }
            Err(e) => {
                println!("获取access_token失败，{}, 正在重新获取", e);
            }
        }

        self
    }

    /**
     * 获取用户信息
     */
    pub async fn get_user_info(&self) -> UserInfo {
        let client = reqwest::Client::new();
        let url = format!(
            "https://pan.baidu.com/rest/2.0/xpan/nas?method=uinfo&access_token={}",
            self.access_token.access_token
        );
        println!("{}", url);
        let res = client
            .get(url)
            .send()
            .await
            .unwrap()
            .json::<UserInfo>()
            .await
            .unwrap();

        println!("{:#?}", res);

        res
    }

    pub async fn get_file_list(&self) -> Vec<CloundFile> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://pan.baidu.com/rest/2.0/xpan/file?method=list&access_token={}",
            self.access_token.access_token
        );
        println!("{}", url);
        let res = client
            .get(url)
            .send()
            .await
            .unwrap()
            .json::<GetFileListRes>()
            .await
            .unwrap();

        println!("{:#?}", res.list);
        res.list
    }

    /**
     * 获取网盘容量
     */
    pub async fn get_capacity(&self) -> PanCapacity {
        let url = format!(
            "https://pan.baidu.com/api/quota?access_token={}",
            self.access_token.access_token
        );
        println!("{}", url);
        let res = reqwest::get(url.as_str())
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        println!("{:#?}", res);
        res
    }

    pub async fn upload_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = std::fs::File::open(file_path)?;

        #[derive(Serialize, Deserialize, Debug)]
        struct CreateFileInfo {
            size: u64,
            category: u32,
            isdir: u32,
            request_id: u64,
            path: String,
            fs_id: u64,
            md5: String,
            ctime: u64,
            mtime: u64,
        }

        #[derive(Serialize, Deserialize, Debug)]
        struct PreCreateResponse {
            return_type: Option<u32>,
            errno: Option<i32>,
            info: Option<CreateFileInfo>,
            block_list: Option<Vec<u32>>,
            request_id: Option<u64>,
            uploadid: Option<String>,
            path: Option<String>,
        }

        let md = file.metadata()?;
        let file_size = md.len();
        let file_type = md.file_type();
        let isdir = match file_type.is_dir() {
            true => "1".to_string(),
            false => "0".to_string(),
        };

        let (file_name, remote_path) = self.get_remote_path(file_path);
        let encode_path = self.url_encode("path", &remote_path);
        let mut list_of_chunks = Vec::new();

        let chunk_size = 1024 * 1024 * 4;

        loop {
            let mut chunk = Vec::with_capacity(chunk_size);
            let n = file
                .by_ref()
                .take(chunk_size as u64)
                .read_to_end(&mut chunk)?;
            if n == 0 {
                break;
            }

            let chunk_md5 = md5::compute(&chunk);

            list_of_chunks.push(Md5Chunks {
                chunk,
                md5: format!("{:x}", chunk_md5),
            });
            if n < chunk_size {
                break;
            }
        }

        let mut md5_arr_str = String::from("");
        list_of_chunks.iter().enumerate().for_each(|(i, x)| {
            match i {
                0 => md5_arr_str.push_str(format!("\"{}\"", x.md5.clone()).as_str()),
                _ => md5_arr_str.push_str(format!(", \"{}\"", x.md5.clone()).as_str()),
            };
        });

        println!("{}", format!("[{}]", md5_arr_str));

        let pre_create_url = format!(
            "https://pan.baidu.com/rest/2.0/xpan/file?method=precreate&access_token={}",
            self.access_token.access_token,
        );

        let pre_create_body = [
            ("path", remote_path.clone()),
            ("isdir", isdir.parse().unwrap()),
            ("size", file_size.to_string()),
            ("block_list", format!("[{}]", md5_arr_str)),
            ("autoinit", String::from("1")),
        ];

        println!("{:#?}", pre_create_body);

        let precreate_client = reqwest::Client::new();
        let precreate_result: PreCreateResponse = precreate_client
            .post(pre_create_url)
            .form(&pre_create_body)
            .send()
            .await?
            .json()
            .await?;

        println!("{:#?}", precreate_result);

        error_panic(precreate_result.errno);

        let return_type = match precreate_result.return_type {
            Some(x) => x,
            None => 0,
        };

        if return_type == 2 {
            // 提前返回， 表示已存在
            let info = precreate_result.info;
            match info {
                None => {}
                Some(info) => {
                    println!("已存在文件： {}", info.path);
                }
            }
            return Ok(());
        }

        let uploadid = match precreate_result.uploadid {
            Some(uploadid) => uploadid,
            None => {
                panic!("uploadid 为空");
            }
        };

        #[derive(Serialize, Deserialize, Debug)]
        struct UploadResponse {
            errno: Option<u64>,
            md5: Option<String>,
            request_id: Option<u64>,
        }

        let mut i = 0_usize;
        let len = list_of_chunks.len();

        async fn upload_chunk(
            list: &Vec<Md5Chunks>,
            i: usize,
            token: &str,
            path: &str,
            uploadid: &str,
            file_name: &str,
            len: usize,
            remote_path: &str,
            isdir: &str,
            file_size: u64,
            md5_arr_str: String,
            uploaded: &mut u32,
        ) {
            let c = list[i].clone();

            type UploadUrl = String;
            let upload_url:UploadUrl = format!(
                "https://d.pcs.baidu.com/rest/2.0/pcs/superfile2?method=upload&access_token={}&type=tmpfile&{}&uploadid={}&partseq={}",
                token,
                path,
                uploadid,
                i,
            );
            println!("{}", upload_url);

            let upload_client = reqwest::Client::new();
            let form_data = multipart::Form::new().part(
                "file",
                multipart::Part::bytes(c.chunk.clone()).file_name(format!("{}{}", i, file_name)),
            );

            let upload_request_builder = upload_client.post(upload_url).multipart(form_data);
            let upload_result = upload_request_builder.send().await.unwrap();
            let upload_result_json = upload_result.json::<UploadResponse>().await.unwrap();

            println!("{:?}", upload_result_json);
            *uploaded += 1;
            if *uploaded >= u32::try_from(len).unwrap() {
                let complete_url = format!(
                    "https://pan.baidu.com/rest/2.0/xpan/file?method=create&access_token={}",
                    token,
                );

                let complete_client = reqwest::Client::new();
                let complete_request_builder = complete_client.post(complete_url);

                let string_block_list = format!("[{}]", md5_arr_str);
                let block_list = string_block_list.as_str();

                let string_size = file_size.to_string();
                let size = string_size.as_str();
                let create_body = [
                    ("path", remote_path.clone()),
                    ("isdir", isdir),
                    ("size", size),
                    ("block_list", block_list),
                    ("uploadid", uploadid),
                ];

                println!("{:#?}", create_body);
                let complete_result = complete_request_builder
                    .form(&create_body)
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap();

                println!("{:#?}", complete_result);
            };
        }

        let mut uploaded = 0_u32;
        loop {
            if i >= len {
                break;
            };

            upload_chunk(
                &list_of_chunks,
                i,
                &self.access_token.access_token,
                &encode_path,
                &uploadid,
                &file_name,
                len,
                &remote_path,
                &isdir,
                file_size.clone(),
                md5_arr_str.clone(),
                &mut uploaded,
            )
            .await;

            i += 1;
        }

        Ok(())
    }

    /**
     * 返回access_token
     */
    pub fn get_token(&self) -> String {
        self.access_token.access_token.clone()
    }

    /**
     * 返回url_encode后的key=value形式的字符串
     */
    pub fn url_encode(&self, key: &str, value: &str) -> String {
        form_urlencoded::Serializer::new(String::new())
            .append_pair(key, value)
            .finish()
    }

    /**
     * 远程目录
     */
    pub fn get_remote_path(&self, file_path: &str) -> (String, String) {
        let split_path = file_path.split("/").collect::<Vec<&str>>();
        let file_name = split_path.last();

        let remote_root = "/apps/xiaok/";
        match file_name {
            Some(file_name) => (
                String::from(*file_name),
                format!("{}{}", remote_root, file_name),
            ),
            _ => (String::from("cache"), format!("{}{}", remote_root, "cache")),
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn json() {
        use serde_json::json;

        // The type of `john` is `serde_json::Value`
        let john = json!({
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        });

        println!("first phone number: {}", john["phones"][0]);

        // Convert to a string of JSON and print it out
        println!("{}", john.to_string());
    }

    #[test]
    fn urlencode() {
        let pan = super::Pan::new();
        let result = pan.url_encode("path", "/BaiduNetdisk.dmg");
        println!("{}", result);
    }

    #[test]
    fn split_file_path() {
        let pan = super::Pan::new();
        let path = pan.get_remote_path("/xixi/xixi/xixi/xixi.png");
        println!("{:?}", path);
    }
}

/**
 * 错误对应信息
 */
fn error_panic(_errno: Option<i32>) {
    match _errno {
        Some(errno) => match errno {
            0 => return,
            2 => panic!("参数错误"),
            111 => panic!("access token 失效"),
            -6 => panic!("身份验证失败"),
            6 => panic!("不允许接入用户数据"),
            31034 => panic!("接口请求过于频繁，注意控制。"),
            _ => {}
        },
        None => {}
    }
}
