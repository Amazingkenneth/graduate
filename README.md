# graduate &emsp; [![CI](https://github.com/Amazingkenneth/graduate/actions/workflows/ci.yml/badge.svg)](https://github.com/Amazingkenneth/graduate/actions/workflows/ci.yml) [![dependency status](https://deps.rs/repo/github/Amazingkenneth/graduate/status.svg)](https://deps.rs/repo/github/Amazingkenneth/graduate)

***有你，才是一班。***

这是一个纪实类项目，我们打算在毕业那天把它作为一个惊喜送给大家，并希望它能够留住初中生活的点滴美好回忆、友谊和事物。
换言之，我们打算写一部民间的《一史》，并且希望能够得到您的支持。我们将使用程序展现这段历史，如果你愿意帮忙，欢迎加入我们，一起来编写这部史书。

![合照](https://graduate-cdn.netlify.com/image/grade7/开学合照.jpg)

## 我想看看你们的成果！
### 下载可执行文件进行测试
有两种方式，你可以选择在 GitHub Artifacts 中选取任意版本的构建文件下载，或者直接在 nightly.link 上下载最新构建的可执行文件。

#### 在 nightly.link 下载最新版本【推荐】
打开 https://nightly.link/Amazingkenneth/graduate/workflows/ci/main ，如下图找到你自己的系统对应的版本，点下载链接下载可执行文件（可能要稍等一下下）

<img width="979" alt="屏幕截图" src="https://user-images.githubusercontent.com/81886982/215748234-97d14836-8ed9-4d68-b623-d42bceb98606.png">


#### 在 GitHub Actions 的 Artifacts 上下载【需要登录】
在 [GitHub Artifacts](https://github.com/Amazingkenneth/graduate/actions/workflows/ci.yml?query=is%3Asuccess) 上找到最新的一次 CI Action，点进去并划到页面最底端，会看到这些可执行文件供下载。
<img width="918" alt="image" src="https://user-images.githubusercontent.com/81886982/211135228-014a6c72-7047-49e3-b927-f29d70f7a714.png">

在界面中找到你自己的系统对应的可执行文件压缩包（建议选择压缩过的，也就是没有 `-uncompressed` 后缀的），下载后解压 zip 后就可以运行啦。

> 注：Linux 系统上需要安装 `libasound2-dev`（Debian / Ubuntu）或 `alsa-lib-devel`（Fedora）作为声音模块 [RustAudio/cpal](https://github.com/RustAudio/cpal) 的依赖项
>
> Note that on Linux, the ALSA development files are required. These are provided as part of the libasound2-dev package on Debian and Ubuntu distributions and alsa-lib-devel on Fedora.

## 想做点贡献？
如果你只想提供点内容，欢迎加入以下几个小组：
- 图片组（征集 + 分类图片）
- 剧本组（写小剧本）
- 手绘组（完成没有图片的集体 events 的插图）
- 表情组（征集众人的表情包）
- 测试组（吐槽开发哪里做的不好）
- 情节组（整理公众号里面的视频 + 根据图片补充班级events + 录制玩笑梗）

当然，如果你会写代码……
开发组同样欢迎你！

### P.S. 这是班级公众号哟
![Wechat Public Account](https://open.weixin.qq.com/qr/code?username=Sal591526579)

欢迎关注，与我们一起分享班级动态！
