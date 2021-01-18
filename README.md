# short-link
一个简单的短链接服务

## 开始
#### 编译/运行
`cargo build --relase` /
`cargo run --relase`

## 配置
* [Rocket](https://rocket.rs) 配置请修改 Rocket.toml,
Rocket 配置[文档](https://rocket.rs/master/guide/configuration).
* ApiKey使用环境变量指定: `API_KEY`

## 使用
### `GET /<short>`

短链接, 访问跳转到对应长链接.

如`/ACFfg4`

---

### `POST /`

Body:
```json5
// json
{
    "link": "http://example.com",
    "custom": "custom_short" // option
}
```

生成短链接. 返回对应的6位短标识符,如`ACFfg4`

`custom`字段可选,自定义短标识符.
此功能需要填写`x-api-key`请求头,内容为`API_KEY`环境变量.
