use std::{fmt, rc::Rc};

use scroll::{ctx, Uleb128};

use crate::error;

#[derive(Debug)]
pub struct ABCString {
    str: Rc<String>,
    /// ABCString 长度，包括 `\0`
    length: usize,
}

impl ABCString {
    pub fn str(&self) -> String {
        self.str.clone().to_string()
    }

    pub fn length(&self) -> usize {
        self.length
    }
}

impl Clone for ABCString {
    fn clone(&self) -> Self {
        ABCString {
            str: self.str.clone(),
            length: self.length,
        }
    }
}

impl fmt::Display for ABCString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.str.clone())
    }
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for ABCString {
    type Error = error::Error;
    fn try_from_ctx(source: &'a [u8], _: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let off = &mut 0;
        let utf16_length = Uleb128::read(source, off).unwrap();

        // 字符串的长度
        let count = (utf16_length >> 1) as usize;
        let bytes = &source[*off..*off + count];
        let str = std::str::from_utf8(bytes).unwrap();
        let mut len = *off + count;

        // 还有`\0`
        len += 1;

        Ok((
            ABCString {
                str: Rc::new(str.to_string()),
                length: len,
            },
            source.len(),
        ))
    }
}
