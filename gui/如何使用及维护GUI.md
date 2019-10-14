# GUI

gui是一个2D渲染库， 支持图片、文字、文字阴影、背景、边框、阴影的渲染。

## GUI的特点

* 简单易用：在gui中， 2D渲染属性的定义尽量遵循了CSS的标准， 对于web开发者来说， 几乎没有学习成本。
* 跨平台： gui不依赖任何与平台相关的库。
* 高性能： 融合了[ECS](https://zhuanlan.zhihu.com/p/54164000)的思想， 使同类数据在内存中尽量连续， 由于CPU的缓存原因在实践中非常高效。

字体支持msdf/canvas
对滚动的优化

下一步计划



## 使用GUI

在使用gui前， 你应该对[CSS](https://developer.mozilla.org/zh-CN/docs/Web/CSS/Reference)有一个基本的了解。

gui提供与CSS属性相对应的接口， 但gui并不支持CSS的所有属性， 在所支持的css属性中，也有部分与css标准有所差异； 同时，gui还支持少量CSS中未定义的属性。

下面是gui支持的全部属性：

| 属性 | 描述 | 与CSS标准相比较 |
| ------ | ------ | ------ |
| background-color | 节点的背景颜色 | 仅支持 rgba、线性渐变 |
| border-color     | 节点的边框颜色 | 仅支持 rgba |
| border-image-source | 边框使用图像的路径 | |
| border-image-clip | 用于剪切边框图片，将被剪切后的图片作用于边框，仅支持百分比设置剪切范围 | <font color=red>CSS不支持</font> |
| border-image-slice | 图像边界向内偏移 | 仅支持百分比设置 |
| border-image-repeat | 用于设置图像边界是否应重复（repeat）、拉伸（stretch）或铺满（round） | |
| image | 节点使用图像的路径，作用于填充和边界（但不包括边距） | 与css中background-image描述一致 |
| image-clip | 用于剪切节点图片，将被剪切后的图片作用于节点，仅支持百分比设置剪切范围 | <font color=red>CSS不支持</font> |
| object-fit | 用于设置图像应该如何适应到其使用的高度和宽度确定的框 | object-fit仅作用于节点的image，并新增了可选择的值，object-fit：fill \| contain \| cover \| none \| scale-down \| repeat \| repeat-x \| repeat-y |
| font-family | 规定文本的字体系列 |  |
| font-size | 规定文本的字体尺寸 |  |
| font-weight | 规定字体的粗细 | 仅支持数字设置：100 \| 200 \| 300 \| 400 \| 500 \| 600 \| 700 \| 800 \| 900 |
| color | 设置文本的颜色 |  |
| letter-spacing | 设置字符间距 |  |
| line-height | 设置行高 |  |
| text-align | 规定文本的水平对齐方式 |  |
| text-indent | 规定文本块首行的缩进 |  |
| white-space | 设置怎样给一元素控件留白 |  |
| word-spacing | 设置单词间距 |  |
| text-shadow | 为文本添加阴影 | 仅支持一重阴影，不能定义多个 |
| text-content | 文字内容 |  |
| pointer-events | 定义元素如何响应点查询 | 仅支持auto \| none \| visible |
| display | 设置一个元素应如何显示 | 仅支持flex \| none |
| visibility | 设置一个元素应如何显示 | 仅支持visible \| hidden |
| z-index | 设置节点在z轴上的顺序 | |
| transfrom | 设置节点的空间变换 | 仅支持scale \| scaleX \| scaleY \| translate \| translateX \| translateY \| rotate \| rotateZ |
| transform-will-change | 用于优化频繁变化的transform，值为 true \| false | <font color=red>CSS不支持</font> |
| opacity | 不透明度 | 当父子节点同时设置opacity， 效果差异较大（可以自己实验） |
| overflow | 设置内容溢出如何显示 | 值为true \| false |
| hsv | 为节点设置滤镜， 与ps中的hsv效果一致 | <font color=red>CSS不支持</font> |
| flex | 这不是一个css属性， 也不对应gui的任何接口，这里只想说明， gui支持的布局属性， gui仅支持flex布局，其属性与css保持一致 |  |
| text-content| 文本内容 |  |

