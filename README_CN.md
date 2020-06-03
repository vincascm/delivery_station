
[English](README.md)|[简体中文](README_CN.md)

# delivery\_station

一个简洁的 与 `Gitea` 集成的 `CI/CD` 部署工具

配置文件说明:

```
http_listen_address: 监听地址
work_dir: 工作目录
gitea_trigger_secret: gitea secret
dingtalk_access_token: 钉钉 access_token
dingtalk_secret: 钉钉 secret
base_url: 访问前缀
host: 主机
  dev: 主机名
    hostname: 
    port: 
    user: 
repository: git 仓库
-
  name: 仓库名
  steps:
  -
    script:
      name: abc.sh
```

其中，`repository` 中的元素有 `template`、`script`、`command` 3种类型。

- `template` 尚未实现。
- `script` 在 `work_dir/scripts` 中查找以 `name` 字段命名的脚本，如果存在 `host`，则将此脚本上传至 `host`，并在`host`中执行；如果不存在，则在本机执行。
- `command` 如果存在 `host`，在`host`中执行些命令；如果不存在，在本机执行。
