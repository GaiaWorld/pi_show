如果你是一个web前端应用开发者，可能遇到过下面的困扰：

* 小游戏不支持dom，使用一些2d渲染库（如babylon的2d渲染库）来渲染2d界面，不能方便的对界面中的元素布局。
* 将web应用打包为android、ios平台的apk时， apk使用webview对web内容进行展示， 但webview的切换成本很高，导致性能低下

GUI正是为解决这些问题而生。 GUI采用系统级语言-Rust进行编程，充分保证程序执行性能；其实现了一个css子集， web开发者只需要按照css标准开发web界面， 就可以应用在各个平台。

GUI在设计上被分为四个部分：
* hal-core：平台渲染兼容层，定义了渲染接口，目前仅实现了webgl的渲染（GUI-web库实现了webgl的渲染接口）， 接下来还会实现opengl、福尔康，让GUI能在Ios和Android上运行
* GUI-core： gui的核心组件，对节点进行布局，根据节点的样式描述，组装渲染数据
* GUI-interface： GUI的接口层， 主要用于web。 GUI在web上的应用，是将GUI编译为asm.js或wasm来对web曝露其接口
* vdom（虚拟dom）： 根据tpl描述，生成虚拟节点树，再生成gui节点树，对界面的改变， 直接通过混合模板得到新的虚拟节点树， 通过对新旧树的比较，增加、修改、或删除GUI节点。使开发者无需关心修改细节， 仅关心数据显示逻辑