1. ubo, geometry等， 原本使用Arc包装用于共享， 但似乎性能不好， 考虑使用Rc包装
2. ubo原本使用hash表对每个不同的key的值索引， 考虑使用静态结构体定义不同类型的ubo
