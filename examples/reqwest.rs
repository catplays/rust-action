use std::fs;

fn main() {
    html2md("","")
}

fn html2md(url: &str, output: &str) {
    println!("Fetching url: {}", url);
    println!("Fetching output: {}", output);

    //unwrap() 方法，只关心成功返回的结果，如果出错，整个程序会终止。

    let body = reqwest::blocking::get(url).unwrap().text().unwrap();
    println!("Converting html to markdown...");
    let md = html2md::parse_html(&body);
    // 将文本写入到文件中

    fs::write(output, md.as_bytes()).unwrap();
}