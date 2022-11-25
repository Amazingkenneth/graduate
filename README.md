# graduate &emsp; [![CI](https://github.com/Amazingkenneth/graduate/actions/workflows/ci.yml/badge.svg)](https://github.com/Amazingkenneth/graduate/actions/workflows/ci.yml) [![Clippy Check](https://github.com/Amazingkenneth/graduate/actions/workflows/clippy-check.yml/badge.svg)](https://github.com/Amazingkenneth/graduate/actions/workflows/clippy-check.yml)

***有你，才是一班。***

这是一个纪实类项目，我们打算在毕业那天把它作为一个惊喜送给大家，我们希望它能够留住初中生活的点滴美好回忆、友谊和事物。
换言之，我们打算写一部民间的《一史》，并且希望能够得到您的支持。我们将使用程序展现这段历史，如果你愿意帮忙，欢迎阅读 [文档](CONTRIBUTING.md) 并同我们一起撰写这部传记。

<meta name="referrer" content="never">
<meta data-draft-node="block" data-draft-type="table" data-size="normal" data-row-style="normal">

![合照](https://mmbiz.qpic.cn/mmbiz_jpg/gkGu2rbIy4EcBYCXhzANZVph9fz6hEFdRNxPoudjiaYEicanTHPW7RLuq1NzKWk4ia5HumLjIeaGibr1h93BzTDMYA/640)

## 我想看看你们的成果！
### 下载可执行文件进行测试
在 [GitHub Artifacts](https://github.com/Amazingkenneth/graduate/actions/workflows/ci.yml?query=is%3Asuccess) 上找到最新的一次 CI Action，点进去并划到页面最底端，会看到如下图所示的不同系统的可执行文件供下载。
[![GitHub Artifacts](https://user-images.githubusercontent.com/81886982/202855238-fbe94bb4-96a0-4f13-9fc5-62cca61d2b77.png)](https://github.com/Amazingkenneth/graduate/actions/workflows/ci.yml?query=is%3Asuccess)

找到你自己的系统对应的版本，下载运行即可（无需安装）。

> 注：Linux 系统上需要安装 `libasound2-dev`（Debian / Ubuntu）或 `alsa-lib-devel`（Fedora）作为声音模块 [RustAudio/cpal](https://github.com/RustAudio/cpal) 的依赖项
>
> Note that on Linux, the ALSA development files are required. These are provided as part of the libasound2-dev package on Debian and Ubuntu distributions and alsa-lib-devel on Fedora.

## 想做点贡献？
**[点此](https://github.com/users/Amazingkenneth/projects/1)** 查看我们当前的进展；

或者，看看 **[这里的说明](https://github.com/Amazingkenneth/graduate/blob/main/CONTRIBUTING.md)**！

## 依赖
- [serde-rs/**json**](https://github.com/serde-rs/json)
- [iced-rs/**iced**](https://github.com/iced-rs/iced)

**[在 Replit 上查看](https://replit.com/@Zykang/graduate#README.md)**

[![Run on Replit](https://replit.com/badge/github/Amazingkenneth/graduate)](https://replit.com/github/Amazingkenneth/graduate)
