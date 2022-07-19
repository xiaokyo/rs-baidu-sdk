
// mac 终端运行 open 'http://blog.csdn.net/jiezhi2013'

use std::process::Command;

// mac中打开网页
pub fn open_url(url: &str) {
    let mut command = Command::new("open");
    command.arg(url);
    command.output().unwrap();
}