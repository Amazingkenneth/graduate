<!-- omit in toc -->
# 关于投稿

首先，十分感谢您能抽出宝贵时间来投稿！ ❤️

我们对于所有的投稿都会予以鼓励、加以重视。 您可以参阅 [目录](#table-of-contents) 来了解不同的投稿方式，以及有关此项目对它们的详细信息的处理办法。在投稿之前，您可以通过了解相关信息，使我们的维护更加方便容易、体验更加流畅顺利。我们在社区期待您的投稿！🎉

> 您如果对这个项目十分感兴趣却没有时间投稿，那么您不妨可以试试以下的几种方便快捷的方法来支持我们的项目，我们对此也十分欢迎：
> - 为项目加注星标
> - 在项目的自述文件中引用这个项目
> - 向您身边的朋友、同事介绍该项目

<!-- omit in toc -->
## 目录

- [“援疑质理”](#i-have-a-question)
- [我要投稿](#i-want-to-contribute)
- [反馈错误](#reporting-bugs)
- [建议改进](#suggesting-enhancements)
- [首次代码投稿](#your-first-code-contribution)
- [首次内容投稿](#your-first-content-contribution)
- [改进文档](#improving-the-documentation)
- [加入我们](#join-the-project-team)

# 贡献指南

## 提交说明

各种提交代码之前一定要

```shell
git pull --rebase
```

----

### 占位符

为避免编辑冲突，在完成第一版翻译前，请尽量不要多人修改同一个页面。

正在为某页面编写第一版翻译的参与者，可提交一个空文档作为占位符，并且在其中注明编写者和 deadline，如：
> Reserved by ZgblKylin until 2020-07-31.

超过 deadline 后尚未完成第一版提交的页面，或已经完成第一版提交的页面，均被视作**开放状态**，其它参与者可对其进行修改。


## 完成度追踪

每添加一个页面，需在[完成度追踪表](completeness_tracking.md)中增加相应条目。

当页面完成编辑、完成维护等状态改变时，维护者有责任在[完成度追踪表](completeness_tracking.md)中更新对应信息。

需要新增或修改一篇文档时，请先检索追踪表中是否已存在该文档，和该文档的翻译进度。



## 参考资料
关于如何将英文文档信达雅地翻译为中文技术文档，可参考 [语法小贴士](https://gitee.com/cryfeifei/QtDocumentCN/blob/master/Grammar_Tips.mdGrammar_Tips.md)。

## Markdown 格式规范
这里是 GitHub 的官方说明。

### 注解

当翻译者需要添加额外的资料或吐槽时，需有明确的标注与官方文档区分开。

若为独立段落，建议使用`>`引用语法，并在开头单独一行标识`译者注:`。

若为段内信息，建议使用段内代码(`译者注：xxx`)的方式标注。

### 中英混排

中英混排时，英文内容前后需增加空格分隔，以避免文字过于紧凑：
> 正确写法：
>
> 我要在 X 文件夹下，新建一个 QX11Info 的文件夹。
>
> 错误写法：
>
> 我要在X文件夹下，新建一个QX11Info的文件夹。

文本字段落中的编程关键字，需用段内代码格式包裹，如：
```text
比如我想翻译 `QX11Info` 类。
```
比如我想翻译 `QX11Info` 类。


### 引用链接

#### 页内跳转

Markdown 页内标题跳转较为简便，语法如下：

```markdown
[页内跳转](本文档名#页内跳转标题)
```

页内跳转：[中英混排](CONTRIBUTING.md#中英混排)

省略本文件名称时，通常也可进行跳转，但有的场景会无法正确生成跳转链接，因此不建议省略。

## Markdown 编辑器
### VSCode

若不想下载独立编辑工具，但想直观阅览 Markdown 渲染效果，则可直接使用 VSCode 编辑`.md`文件。

点击 VSCode 的`.md`文件标签页右上角的“打开侧边预览”(Ctrl+K V)，或在命令面板(F1)中搜索“Markdown”并选择打开预览(Markdown: Open Preview)即可。

## 您的首次代码投稿
<!--**@开发组同学们**：
我们非常欢迎大家写脚本（ Python/C++/Javascript/VBS...）来预处理 `data/string.txt` 和 `data/graph.txt`，并生成一个 .json 文件（比如说 config.json）来供后面 Rust 主程序调用。-->

### config.json
我们使用 `config.json` 用于游戏初始化配置：
> 不懂的童鞋们可以看看 [json.org](https://www.json.org/json-zh.html)，学习一下 json 的相关语法。

本项目中，我们用 `/data/src/gen.cpp` 生成一个如下格式的 config.json 配置文件。
```json
{ // json 文件框架
    "idx": [ // Index: Vec<Node>，是储存节点信息的数据结构
        {
            "tp": "Random", // 表示是一个随机跳边的节点
            "zh": "出门", // 对应 `/data/string.txt` 中的节点内容
            "num": 1, // num 表示节点的编号（编号 = 对应行号）
            "ch": [ // 表示有哪些子节点
                {
                    "att": [ // attributes，表示边的属性
                        2, // 表示连向节点 2
                        1 // 表示有 1 份的概率选中这条边
                    ]
                },
                {
                    "att": [
                        3,
                        4
                    ]
                },
                {
                    "att": [
                        4,
                        3
                    ]
                }
            ]
        },
        {
            "tp": "Ending", // 表示是一种结局
            "zh": "坐飞机",
            "num": 2,
            "ch": [] // 结局当然没有子节点啦
        }
    ]
}
```

## Your First Content Contribution
您的第一次内容贡献
### 设计剧情
We place strings in the game in `/data` file. `/data/string.txt` is only for the sentences and words, and `/data/graph.txt` is only for the logic between the contents.
我们将字符串放在“/data”文件中。' /data/string.txt '仅用于句子和单词，' /data/graph.txt '仅用于内容之间的逻辑。
Here is an example graph:
下面是一个示例图:
![example_graph](https://user-images.githubusercontent.com/81886982/198510011-8550b2d0-ba15-468c-a800-db34a189537a.png)

#### string.txt
https://github.com/Amazingkenneth/graduate/blob/e7859c8970a6149890d3f031eb1c695ded3dac06/data/string.txt#L1-L7

We use `|`(ASCII 0x7C) to separate Chinese and English contents.

Each line corresponds to the node number is its line number, which means the example content above is the content of node 1 (Because it is at Line 1).
#### graph.txt

The graph follows this format:
```txt
x,y p
x,y
```
- when the line is in format of "x,y p", it means node `x` has a edge to `y` with a weight of probability of `p`.
- when the line is in format of "x,y", it means node `x` is a question with possible answer for user of node `y`.
### 设计地图
#### 首选项
要求包含以下几个配置
- size
  要求是矩形，给出长和宽，以 `axb` 的形式，如 `128x128`，`720x480`
- 选项别名

## Improving The Documentation
You may help us to translate the documents.
## Join The Project Team
New members are welcome to join us!
