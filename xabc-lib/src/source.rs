use std::{
    clone::Clone,
    convert::AsRef,
    ops::{Deref, Index},
    sync::Arc,
};

/// 存放 ABC 文件的原始数据，用于浅拷贝。
pub struct Source<T> {
    // Rc<T> 是一个引用计数的智能指针，用于在多个所有权之间共享不可变访问权。
    // 当你调用 Rc<T> 实例的 clone() 方法时，它不会创建 T 的数据的副本，而是增加引用计数，从而允许新的 Rc<T> 实例共享相同的数据。
    // inner: Rc<T>,
    // 线程安全，Python 接口需要。
    inner: Arc<T>,
}

impl<T> Source<T>
where
    T: AsRef<[u8]>,
{
    /// 创建一个新的 `Source`
    pub(crate) fn new(inner: T) -> Self {
        Self {
            // inner: Rc::new(inner),
            inner: Arc::new(inner),
        }
    }
}

impl<T> Clone for Source<T> {
    /// 浅拷贝
    fn clone(&self) -> Self {
        Self {
            // 不会创建 T 的数据副本
            inner: self.inner.clone(),
        }
    }
}

// scroll::Pread 必须实现这个
// `Deref` 用于重载 * 运算符，可以通过 * 运算符访问 Source 的内部数据。
impl<T: AsRef<[u8]>> Deref for Source<T> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

// `AsRef<T>` 允许将一个类型的引用转换为类型 T 的引用，而不需要进行显式的转换或复制。
// 重载 & 运算符； 将 Source 转换为 &[u8]
impl<T: AsRef<[u8]>> AsRef<[u8]> for Source<T> {
    /// 获取内部数据的引用
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref().as_ref()
    }
}

/// Index 用于重载 [] 运算符
impl<T> Index<usize> for Source<T>
where
    T: AsRef<[u8]>,
{
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_ref()[index]
    }
}

/// Index 用于重载 [m..n] 运算符
impl<T> Index<std::ops::Range<usize>> for Source<T>
where
    T: AsRef<[u8]>,
{
    type Output = [u8];

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        &self.as_ref()[index]
    }
}

/// Index 用于重载 [m..] 运算符
impl<T> Index<std::ops::RangeFrom<usize>> for Source<T>
where
    T: AsRef<[u8]>,
{
    type Output = [u8];

    fn index(&self, index: std::ops::RangeFrom<usize>) -> &Self::Output {
        &self.as_ref()[index]
    }
}
