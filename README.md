# pbbot-rq
基于Rust的QQ协议接口运行程序

## 报错问题解决

### error[E0554]
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

### 解决方法
执行命令：`rustup override set nightly`后再运行build命令编译！
