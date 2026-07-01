//! Profile engine dispatch.

mod dispatch;
mod hybrid;

pub(crate) use dispatch::{check_consistency, classify, resolve, sub_object_properties};
