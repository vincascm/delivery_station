
[English](README.md)|[简体中文](README_CN.md)

# delivery\_station

一个简洁的 与 `Gitea` 集成的 `CI/CD` 部署工具


## 配置文件说明


### 逐行说明

```yaml
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
environment: 环境变量
  TARGET: target
repository: git 仓库
-
  name: 仓库名
  branch: git分支(可选, 其值可为'@any', 表示只匹配类型为branch的`ref`)
  tag: git tag(可选, 其值可为'@any', 表示只匹配类型为tag的`ref`))
  environment: 环境变量(可选)
  steps: 执行步骤
    kind: 类型(command or script)
    name: 脚本名称(有字符串和数组两种形式，传递参数用数组形式, kind是script时有效)
    command: 命令(有字符串和数组两种形式，传递参数用数组形式, kind是command时有效)
    host: 执行的目标主机(可选，如果不指定，在本机执行)
    current_dir: 当前目录(可选，在本机执行时有效)
    environment: 环境变量(可选，在本机执行时有效)
```

其中，`steps`中`kind` 有 `script`、`command` 两种类型。

- `script` 在 `work_dir/scripts` 中查找以 `name` 字段命名的脚本，如果存在 `host`，则将此脚本上传至 `host`，并在`host`中执行；如果不存在，则在本机执行。
- `command` 如果存在 `host`，在`host`中执行些命令；如果不存在，在本机执行。

`script`或`command`默认有如下环境变量：

- **TRIGGERED_INFO_REPOSITORY**: 仓库名称，如`com/test`
- **TRIGGERED_INFO_BRANCH**: , 分支名称，可选
- **TRIGGERED_INFO_TAG**: , `tag`名称，可选
- **TRIGGERED_INFO_STEPS_NAME**: , `steps_name`，可选

### config file example

```yaml
http_listen_address: 0.0.0.0:8080
work_dir: ./work_dir
gitea_trigger_secret: SECRET
dingtalk_access_token: TOKEN
dingtalk_secret: SECRET
base_url: http://127.0.0.1:8080
host:
  dev:
    hostname: 192.168.1.1
    port: 22
    user: root
repository:
-
  name: com/test
  steps:
    kind: script
    name: abc.sh
    environment:
      X_ABC: hi
-
  name: com/qq
  steps:
  -
    kind: script
    name: abc.sh
    host: dev
-
  name: com/abc
  steps:
    dev:
        kind: command
        command: [ls, "-lh"]
        current_dir: /tmp
        environment:
          SRC: /tmp/test
    prod:
    -
        kind: script
        name: [abc.sh, a33]
        host: dev
```

## `gitea` 配置

在`gitea`管理后台 -> 默认Web钩子 中添加Gitea Web钩子，

- 目标URL填写形如`http://127.0.0.1:8080/gitea_trigger`(`hostname`根据实际情况替换)
- HTTP方法选择`POST`
- POST Content Type 选择 `application/json`
- 触发条件选择`推送事件`
- 分支过滤填写`*`

添加成功后创建一新仓库，并打开此仓库的`仓库设置` -> `管理Web钩子`，点击列表中一项，找到并点击`测试推送`按钮, 若响应内容是`success`，
在钉钉中查看机器人发送的消息，`status`表示steps执行状态，`logs`对应steps执行输出,stdout对应标准输出,stderr对应标准错误输出。

## 手动触发

除了在`gitea`的事件发生时触发，也可手动触发，例如：

```shell
curl -s -H "Content-Type: application/json" -d '{"repository":"com/abc", "branch":"master", "tag":"1.2.3", "steps_name": "dev"}' http://127.0.0.1:8080/manual_trigger
```

其各参数分别表示：

- *repository* : 仓库名称
- *branch* : git分支, 可选
- *tag* : git tag, 可选
- *steps_name* : steps name, 可选

## TODO

- [x] branch 匹配
- [x] tag 匹配

