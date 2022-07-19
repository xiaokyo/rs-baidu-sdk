use std::{error::Error, io::Read};

use serde::{Deserialize, Serialize};
use serde_json::json;

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

#[derive(Debug, Deserialize, Serialize)]
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
            return_type: u32,
            errno: u32,
            info: Option<CreateFileInfo>,
            block_list: Option<Vec<u32>>,
            request_id: u64,
            uploadid: Option<String>,
            path: Option<String>,
        }

        let mut md = file.metadata()?;
        let file_size = md.len();
        let file_type = md.file_type();
        let isdir = match file_type.is_dir() {
            true => "1",
            false => "0",
        };
        let encode_path = self.url_encode("path", file_path);
        let mut list_of_chunks = Vec::new();
        // let mut list_of_chunkMd5 = Vec::new();

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

        let md5_str = &list_of_chunks
            .into_iter()
            .map(|x| x.md5.clone())
            .collect::<Vec<String>>();
        let md5_text = serde_json::to_string(md5_str).unwrap();
        println!("{:?}", md5_text);

        let url = format!(
            "https://pan.baidu.com/rest/2.0/xpan/file?method=precreate&access_token={}&{}&size={}&block_list={}&autoinit=1&isdir={}",
            self.access_token.access_token,
            encode_path,
            file_size,
            md5_text,
            isdir
        );
        println!("{}", url);
        let client = reqwest::Client::new();
        let result: PreCreateResponse =
            client.post(url).send().await.unwrap().json().await.unwrap();

        println!("{:#?}", result);

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
}