mod lib;

use lib::pan::Pan;
use std::thread;
use std::time::Duration;

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

    pan.upload_file("WeGameMiniLoader.std.3.08.23.1122.exe").await;

    Ok(())
}
