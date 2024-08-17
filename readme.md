# 命令

cargo build --quiet && target/debug/rust-action post httpbin.org/post a=1 b=2

cargo build --quiet && target/debug/rust-action get "http://localhost:8080/image/CgoKCAjYBBCgBiADCgY6BAgUEBQKBDICCAM/https%3A%2F%2Fimages%2Epexels%2Ecom%2Fphotos%2F2470905%2Fpexels%2Dphoto%2D2470905%2Ejpeg%3Fauto%3Dcompress%26cs%3Dtinysrgb%26dpr%3D2%26h%3D750%26w%3D1260"

# 常用 debug

## cargo expand 实现宏的展开

eg: cargo expand --example tokio1

# 库

- web 服务器： axum
- json 解析： jsonxf
- 错误处理： anyhow
- 异步处理： tokio
- 表格处理： polars
- 命令行解析： clap
- http 请求： reqwest
- 多彩显示库： colored
- 高亮库： syntect
- 并发测试框架：loom
