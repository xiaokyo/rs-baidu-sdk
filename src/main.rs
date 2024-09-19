mod myth;

use std;

use myth::pan::Pan;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pan = Pan::new();
    // pan.get_device_code().await;

    // loop {
    //     thread::sleep(Duration::from_secs(4));
    //     pan.get_access_token().await;
    //     if pan.get_token().len() > 0 {
    //         break;
    //     }
    // }

    // pan.get_user_info().await;
    // pan.get_file_list().await;
    // pan.get_capacity().await;

    pan.upload_file("小困打卡小程序.xmind").await.unwrap();
    Ok(())
}
