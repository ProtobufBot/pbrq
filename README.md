# pbbot-rq
基于Rust的QQ协议接口运行程序

## 报错问题解决

### [1]error[E0554]Cancel changes
```
error[E0554]: `#![feature]` may not be used on the stable release channel
 --> C:\Users\Dongeast\.cargo\git\checkouts\rs-qq-f947ea2807e050e7\e7c6131\rq-engine\src\lib.rs:1:1
  |
1 | #![feature(type_alias_impl_trait)]
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

   Compiling hyper-tls v0.5.0
   Compiling axum v0.4.8
   Compiling reqwest v0.11.10
For more information about this error, try `rustc --explain E0554`.
error: could not compile `rq-engine` due to previous error
warning: build failed, waiting for other jobs to finish...
error: build failed
```
![error: build failed](https://user-images.githubusercontent.com/66114014/164759104-4eb9e7b4-8e9e-4a29-bdac-ca2a14bbd5bb.png)

### [1]解决方法
执行命令：`rustup override set nightly`后再运行build命令编译！

## 接口的访问方法
### 使用PostMan等工具进行访问测试
- 第一步：打开我们的PostMan工具，如果我们想要访问我们已经运行的`pbbot-rq`程序的接口，我们需要在Headers中添加一条配置，设置`Content-type`为`application/json;charset=UTF-8`访问设置如下：
![Header](https://user-images.githubusercontent.com/66114014/164883178-607b4285-2aac-435b-a6ef-67b12600660a.png)
- 第二步：在Body中填写我们的参数列表，选择访问方式为POST/GET
![设置参数列表和访问方式](https://user-images.githubusercontent.com/66114014/164883513-bbe07c62-a54b-423e-81c9-ba776b2e377b.png)
### 接口的参数和访问方式

# 静态文件服务
将编译好的静态文件放到main.exe同目录下，打开main.exe,在浏览器里输入 `127.0.0.1:3000/index`
![1](./assets/QQ%E6%88%AA%E5%9B%BE20220428131935.png)
![1](./assets/QQ%E6%88%AA%E5%9B%BE20220428132804.png)
![1](./assets/QQ%E6%88%AA%E5%9B%BE20220428132940.png)

前端项目地址:[pbbot-rq-ui](https://github.com/dongeast/pbbot-rq-ui)