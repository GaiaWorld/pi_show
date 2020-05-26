# GUI源码目录结构及作用

## entity
实体： 在gui中， 每个节点（Node）被看作是一个实体。

## component
组件： 节点上的数据，每节点都有多种数据，每种数据都被看做组件， 一个节点可以有多个组件。 如：一个node上同时存在Zindex，Transfrom， BackGround组件。

gui 用户设置组件：

	Show
	ZIndex
	Transfrom
	TransformWillChange
	Layout
	Opacity
	Overflow
	Hsv
	BackGroundColor
	BorderColor
	BorderImage
	BorderImageClip
	BorderImageSlice
	BorderImageRepeat
	Image
	ImageClip
	ObjectFit
	TextContent
	Text
	ClassName

gui 中间计算组件

	Visibility
	Enable
	Display
	Opacity
	Filter
	ZDepth
	CharBlock
	TransformWillChangeMatrix
	ByOverflow

## single
单例： 包含一定的数据和逻辑， 与组件不同， 单例上的数据并不与每实体（节点）对应， 可以将单例上的数据看作是全局共享的， 如class表， 字体资源表。 单例上还存在一定逻辑， 用于操作单例中的数据

gui中的单例

	ClassSheet  // 全局的class样式缓冲
	FontSheet // 字体资源管理
	DefaultTable // 默认的样式

	Oct // 八叉树，存储每个节点的包围盒， 并可以根据坐标快速命中一个aabb
	ViewMatrix // 渲染视口矩阵
	ProjectionMatrix // 渲染投影矩阵
	ImageWaitSheet // 图片等待列表（图片异步加载， 未加载的图片处于等待状态）
	RenderObjs // 所有的渲染对象（渲染对象不同与节点， 一个节点可能会产生多个渲染对象（同时存在Image，backgroundColor等可显示的组件）， 也可以没有渲染对象（div空节点））
	OverflowClip // overflow属性生成的裁剪平面的集合
	DirtyList // 脏列表， 记录了存在任意属性变动的节点id
	UnitQuad // 单位自已四边形Geometry，由于Image， BackgroundColor等物件的几何形态通常都是一个四边形， 可以有一个全局的单位四边形共享出来， 各自的大小位置可以放在世界矩阵中表示
	NodeRenderMap // 一个节点和渲染对象的映射表（一对多）


## system
系统， 纯粹的逻辑代码， 可以监听、读取和修改组件、单例上的数据。 系统根据现有数据， 得到需要的结果， 可以将结果写入到组件或单例， 共享给其他系统

系统列举 ：

	layout： 监听实体（Node）的创建和销毁， 创建或销毁布局节点
	textlayout: 虽然名字叫textlayout， 但实际上节点的布局也是由它驱动的。 其主要还是监听文本相关组件的变化， 将修改的文本重新布局， 并将布局结果记录到CharBlock组件中。
	overflow： 根据overflow、worldMatrix、TransformWillChangeMatrix计算裁剪平面， 将裁剪平面存储到OverflowClip单例中， 并设置其子节点的ByOverflow属性
	show： 根据计算enable、display、visiblity
	oct： 计算包围盒， 将包围盒放入八叉树
	opacity： 递归计算不透明度， 子节点的不透明度等于自身不透明度与父节点不透明不相乘（效果与css一致， 要做到css的效果，会有更多的性能损失）
	transformwillchange： 根据transformwillchange计算对应的矩阵
	worldmatrix： 计算世界矩阵
	filter： 计算hsv
	style_marke: 监听所有属性的改变并设脏， 同时也负责监听className的设置， 在classSheet中取出对应的class， 将其属性设置到对应的组件上， 如果是图片路径属性， 需要将未加载的图片放入图片等待列表中
	render {
		background_color： 背景颜色渲染对象的属性设置
		image 图片渲染对象的属性设置
		border_color： 边框颜色渲染对象的属性设置
		border_image： 边框渲染对象的属性设置
		box_shadow: 阴影渲染对象的属性设置
		charblock: 文字和文字阴影渲染对象的属性设置
		node_attr: 渲染对象通用属性设置
		render: 渲染， 将渲染对象按照透明与不透明分类， 先渲染不透明物体， 再渲染透明物体， 不透明物体按照渲染管线的顺序渲染， 透明物体按照物体的深度顺序渲染
		res_release： 资源整理
	}
