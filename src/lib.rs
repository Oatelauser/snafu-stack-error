#![feature(error_iter)]

use std::any::Any;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

pub use stack_error_derive::stack_trace_debug;

use crate::code::StatusCode;

pub mod code;

pub trait StackError: Error {
    /// 向buf中写入错误信息
    /// layer：当前错误的层数
    /// buf：从外到内搜集所有错误的信息
    fn debug_fmt(&self, layer: usize, buf: &mut Vec<String>);

    /// 获取当前错误指向的下一层的[StackError]
    fn next(&self) -> Option<&dyn StackError>;

    /// 获取最后一条错误对象[StackError]
    fn last(&self) -> &dyn StackError where Self: Sized {
        let mut last_error: &dyn StackError = self;
        while let Some(error) = self.next() {
            last_error = error;
        }
        last_error
    }
}

impl<T: StackError> StackError for Box<T> {
    fn debug_fmt(&self, layer: usize, buf: &mut Vec<String>) {
        self.as_ref().debug_fmt(layer, buf)
    }

    fn next(&self) -> Option<&dyn StackError> {
        self.as_ref().next()
    }
}

impl<T: StackError> StackError for Arc<T> {
    fn debug_fmt(&self, layer: usize, buf: &mut Vec<String>) {
        self.as_ref().debug_fmt(layer, buf)
    }

    fn next(&self) -> Option<&dyn StackError> {
        self.as_ref().next()
    }
}

pub trait ErrorExt: StackError {
    /// 获取当前错误的错误码[StatusCode]
    fn status_code(&self) -> StatusCode {
        StatusCode::Unknown
    }

    /// 当前对象转换为[Any]
    fn as_any(&self) -> &dyn Any;

    /// 对外输出错误信息
    fn output_msg(&self) -> String where Self: Sized {
        match self.status_code() {
            StatusCode::Unknown | StatusCode::Internal => format!("Internal error: {}", self.status_code() as u32),
            _ => {
                let error = self.last();
                if let Some(root_error) = error.source() {
                    let root_error = root_error.sources().last().unwrap();
                    let error_msg = error.to_string();
                    if error_msg.is_empty() {
                        format!("{root_error}")
                    } else {
                        format!("{error_msg}: {root_error}")
                    }
                } else {
                    format!("{error}")
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct PlainError {
    msg: String,
    status_code: StatusCode,
}

impl PlainError {
    pub fn new(msg: impl Into<String>, status_code: StatusCode) -> Self {
        Self { msg: msg.into(), status_code }
    }
}

impl Display for PlainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for PlainError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl StackError for PlainError {
    fn debug_fmt(&self, layer: usize, buf: &mut Vec<String>) {
        buf.push(format!("{}: {}", layer, self.msg))
    }

    fn next(&self) -> Option<&dyn StackError> {
        None
    }
}

impl ErrorExt for PlainError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
